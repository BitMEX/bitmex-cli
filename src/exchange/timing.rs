/// curl-style per-request timing: namelookup, connect (TCP+TLS), TTFB, transfer, total.
///
/// Architecture:
/// - `TimingHandle` = `Arc<Mutex<Phases>>` shared between the DNS resolver,
///   connector layer, and the request call site.
/// - `TimingDnsResolver` wraps tokio's system resolver and records namelookup time.
/// - `TimingConnectorLayer` is a Tower Layer applied via `ClientBuilder::connector_layer`.
///   It fires connect_start before the inner connector runs (DNS+TCP+TLS) and
///   connect_end when the stream is ready, giving appconnect time.
/// - The call site records request_start, ttfb, and transfer_end.
///
/// Designed for sequential CLI use. Concurrent requests would interleave timings.
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

// ---------------------------------------------------------------------------
// Phase data
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
pub(crate) struct Phases {
    pub request_start: Option<Instant>,
    pub dns_start: Option<Instant>,
    pub dns_end: Option<Instant>,
    pub connect_start: Option<Instant>,
    pub connect_end: Option<Instant>, // TCP + TLS complete
    pub ttfb: Option<Instant>,        // first byte (response headers received)
    pub transfer_end: Option<Instant>,
}

impl Phases {
    /// DNS resolution time.
    pub fn namelookup(&self) -> Option<Duration> {
        Some(self.dns_end? - self.dns_start?)
    }
    /// TCP + TLS handshake time (after DNS).
    pub fn appconnect(&self) -> Option<Duration> {
        match (self.connect_end, self.dns_end) {
            (Some(end), Some(dns_end)) => Some(end - dns_end),
            (Some(end), None) => self.connect_start.map(|s| end - s),
            _ => None,
        }
    }
    /// Time to first byte from request start.
    pub fn starttransfer(&self) -> Option<Duration> {
        Some(self.ttfb? - self.request_start?)
    }
    /// Body transfer time after first byte.
    pub fn transfer(&self) -> Option<Duration> {
        Some(self.transfer_end? - self.ttfb?)
    }
    /// Total end-to-end time.
    pub fn total(&self) -> Option<Duration> {
        Some(self.transfer_end? - self.request_start?)
    }
}

pub(crate) type TimingHandle = Arc<Mutex<Phases>>;

pub(crate) fn new_handle() -> TimingHandle {
    Arc::new(Mutex::new(Phases::default()))
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

pub(crate) fn print_timing(
    phases: &Phases,
    method: &str,
    url: &str,
    server_ms: Option<&str>,
    request_id: Option<&str>,
) {
    let ms = |d: Option<Duration>| -> String {
        d.map(|d| format!("{:.0}ms", d.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "-".to_string())
    };

    let mut output = format!(
        "\n── Timing: {method} {url}\n\
         {:>22}  {}\n\
         {:>22}  {}\n\
         {:>22}  {}\n\
         {:>22}  {}\n\
         {:>22}  {}",
        "namelookup:", ms(phases.namelookup()),
        "connect (TCP+TLS):", ms(phases.appconnect()),
        "starttransfer:", ms(phases.starttransfer()),
        "transfer:", ms(phases.transfer()),
        "total:", ms(phases.total()),
    );

    if let Some(s) = server_ms {
        output.push_str(&format!("\n{:>22}  {}ms", "server processing:", s));
    }
    if let Some(id) = request_id {
        output.push_str(&format!("\n{:>22}  {}", "x-request-id:", id));
    }

    eprintln!("{output}");
}

// ---------------------------------------------------------------------------
// DNS resolver wrapper
// ---------------------------------------------------------------------------

pub(crate) struct TimingDnsResolver {
    handle: TimingHandle,
}

impl TimingDnsResolver {
    pub(crate) fn new(handle: TimingHandle) -> Self {
        Self { handle }
    }
}

impl Resolve for TimingDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let handle = self.handle.clone();
        let host = name.as_str().to_string();

        Box::pin(async move {
            if let Ok(mut phases) = handle.lock() {
                phases.dns_start = Some(Instant::now());
            }

            let lookup = tokio::net::lookup_host(format!("{}:0", host)).await;

            let addrs: Result<Vec<SocketAddr>, Box<dyn std::error::Error + Send + Sync>> =
                lookup
                    .map(|iter| {
                        // Sort IPv4 before IPv6 so hyper connects without waiting
                        // for an IPv6 timeout when IPv6 is unreachable. This matches
                        // what reqwest's default resolver does on most platforms.
                        let all: Vec<SocketAddr> = iter.collect();
                        let mut v4: Vec<SocketAddr> = all.iter().filter(|a| a.is_ipv4()).cloned().collect();
                        let v6: Vec<SocketAddr> = all.iter().filter(|a| a.is_ipv6()).cloned().collect();
                        v4.extend(v6);
                        v4
                    })
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>);

            if let Ok(mut phases) = handle.lock() {
                phases.dns_end = Some(Instant::now());
            }

            let addrs = addrs?;
            let boxed: Addrs = Box::new(addrs.into_iter());
            Ok(boxed)
        })
    }
}

// ---------------------------------------------------------------------------
// Connector layer (times the full DNS+TCP+TLS connection setup)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct TimingConnectorLayer {
    handle: TimingHandle,
}

impl TimingConnectorLayer {
    pub(crate) fn new(handle: TimingHandle) -> Self {
        Self { handle }
    }
}

impl<S: Clone> tower::Layer<S> for TimingConnectorLayer {
    type Service = TimingConnector<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TimingConnector {
            inner,
            handle: self.handle.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct TimingConnector<S> {
    inner: S,
    handle: TimingHandle,
}

impl<S, Req> tower::Service<Req> for TimingConnector<S>
where
    S: tower::Service<Req> + Clone + Send + 'static,
    S::Future: Send + 'static,
    Req: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TimingConnectorFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let handle = self.handle.clone();
        if let Ok(mut phases) = handle.lock() {
            phases.connect_start = Some(Instant::now());
        }
        TimingConnectorFuture {
            inner: self.inner.call(req),
            handle,
        }
    }
}

pin_project_lite::pin_project! {
    pub(crate) struct TimingConnectorFuture<F> {
        #[pin]
        inner: F,
        handle: TimingHandle,
    }
}

impl<F, T, E> Future for TimingConnectorFuture<F>
where
    F: Future<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(result) => {
                if result.is_ok() {
                    if let Ok(mut phases) = this.handle.lock() {
                        phases.connect_end = Some(Instant::now());
                    }
                }
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
