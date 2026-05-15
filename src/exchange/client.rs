/// HTTP client for the BitMEX REST API.
///
/// `BitmexClient` enforces TLS-only transport (via rustls), handles request
/// signing with HMAC-SHA256, and retries transient network errors with
/// exponential backoff.
///
/// Rate limiting is server-authoritative: the client surfaces `x-ratelimit-*`
/// headers in error responses so agents can adapt their behaviour.
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

use super::auth;
use crate::config::Credentials;
use crate::errors::{BitmexError, Result};

pub(crate) const MAINNET_URL: &str = "https://www.bitmex.com/api/v1";
pub(crate) const TESTNET_URL: &str = "https://testnet.bitmex.com/api/v1";
pub(crate) const WS_MAINNET_URL: &str = "wss://ws.bitmex.com/realtime";
pub(crate) const WS_TESTNET_URL: &str = "wss://ws.testnet.bitmex.com/realtime";

/// Trait abstraction over the BitMEX REST API surface.
///
/// All command modules accept `&impl ExchangeClient`, enabling mock clients
/// for unit testing without live HTTP.
pub trait ExchangeClient: Send + Sync {
    fn get(&self, path: &str, query: &str) -> impl Future<Output = Result<Value>> + Send;
    fn get_auth(
        &self,
        path: &str,
        query: &str,
        creds: &Credentials,
    ) -> impl Future<Output = Result<Value>> + Send;
    fn post(
        &self,
        path: &str,
        body: &Value,
        creds: &Credentials,
    ) -> impl Future<Output = Result<Value>> + Send;
    fn put(
        &self,
        path: &str,
        body: &Value,
        creds: &Credentials,
    ) -> impl Future<Output = Result<Value>> + Send;
    fn delete(
        &self,
        path: &str,
        query: &str,
        body: Option<&Value>,
        creds: &Credentials,
    ) -> impl Future<Output = Result<Value>> + Send;
}

use std::future::Future;

/// BitMEX REST API client with retry and server-authoritative rate limiting.
pub struct BitmexClient {
    http: reqwest::Client,
    pub(crate) base_url: String,
    verbose: bool,
    /// When Some, timing phases are collected and printed for every request.
    timing: Option<super::timing::TimingHandle>,
}

fn build_http_client(
    timing: Option<&super::timing::TimingHandle>,
) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .use_rustls_tls()
        .timeout(Duration::from_secs(30))
        .user_agent(concat!("bitmex-cli/", env!("CARGO_PKG_VERSION")));

    if let Some(handle) = timing {
        builder = builder
            .dns_resolver(Arc::new(
                super::timing::TimingDnsResolver::new(handle.clone())
            ))
            .connector_layer(
                super::timing::TimingConnectorLayer::new(handle.clone())
            );
    }

    if let Ok(val) = env::var("BITMEX_DANGER_ACCEPT_INVALID_CERTS") {
        if matches!(val.as_str(), "1" | "true" | "yes") {
            eprintln!(
                "WARNING: TLS certificate verification is DISABLED via \
                 BITMEX_DANGER_ACCEPT_INVALID_CERTS. Do NOT use this in production."
            );
            builder = builder.danger_accept_invalid_certs(true);
        }
    }

    builder
        .build()
        .map_err(|e| BitmexError::Network { message: format!("Failed to build HTTP client: {e}") })
}

impl BitmexClient {
    /// Create a new client targeting mainnet or testnet.
    pub fn new(testnet: bool, verbose: bool, timing: bool) -> Result<Self> {
        let base_url = if testnet {
            TESTNET_URL.to_string()
        } else {
            MAINNET_URL.to_string()
        };
        let timing_handle = timing.then(super::timing::new_handle);
        Ok(Self {
            http: build_http_client(timing_handle.as_ref())?,
            base_url,
            verbose,
            timing: timing_handle,
        })
    }

    /// Create a new client with an explicit base URL override.
    pub fn new_with_url(base_url: String, verbose: bool, timing: bool) -> Result<Self> {
        let timing_handle = timing.then(super::timing::new_handle);
        Ok(Self {
            http: build_http_client(timing_handle.as_ref())?,
            base_url,
            verbose,
            timing: timing_handle,
        })
    }

    /// Public GET — no authentication headers.
    pub async fn get(&self, path: &str, query: &str) -> Result<Value> {
        let url = self.url(path, query);
        if self.verbose { crate::cli::output::verbose(&format!("GET {url}")); }
        self.begin_request();
        let resp = super::middleware::execute_with_retry(self.verbose, || async {
            self.http.get(&url).send().await
                .map_err(|e| BitmexError::Network { message: e.to_string() })
        }).await?;
        self.parse_response(resp, "GET", &url).await
    }

    /// Authenticated GET.
    pub async fn get_auth(&self, path: &str, query: &str, creds: &Credentials) -> Result<Value> {
        let full_path = if query.is_empty() {
            format!("/api/v1{path}")
        } else {
            format!("/api/v1{path}?{query}")
        };
        let url = self.url(path, query);
        if self.verbose { crate::cli::output::verbose(&format!("GET {url} (auth)")); }
        self.begin_request();
        let resp = super::middleware::execute_with_retry(self.verbose, || async {
            let expires = auth::generate_expires()?;
            let sig = auth::sign("GET", &full_path, expires, "", &creds.api_secret)?;
            let headers = self.auth_headers(&creds.api_key, expires, &sig)?;
            self.http.get(&url).headers(headers).send().await
                .map_err(|e| BitmexError::Network { message: e.to_string() })
        }).await?;
        self.parse_response(resp, "GET", &url).await
    }

    /// Authenticated POST with JSON body.
    pub async fn post(&self, path: &str, body: &Value, creds: &Credentials) -> Result<Value> {
        let full_path = format!("/api/v1{path}");
        let body_str = serde_json::to_string(body)?;
        let url = self.url(path, "");
        if self.verbose { crate::cli::output::verbose(&format!("POST {url}")); }
        self.begin_request();
        let resp = super::middleware::execute_with_retry(self.verbose, || async {
            let expires = auth::generate_expires()?;
            let sig = auth::sign("POST", &full_path, expires, &body_str, &creds.api_secret)?;
            let headers = self.auth_headers(&creds.api_key, expires, &sig)?;
            self.http.post(&url).headers(headers)
                .header("Content-Type", "application/json")
                .body(body_str.clone()).send().await
                .map_err(|e| BitmexError::Network { message: e.to_string() })
        }).await?;
        self.parse_response(resp, "POST", &url).await
    }

    /// Authenticated PUT with JSON body.
    pub async fn put(&self, path: &str, body: &Value, creds: &Credentials) -> Result<Value> {
        let full_path = format!("/api/v1{path}");
        let body_str = serde_json::to_string(body)?;
        let url = self.url(path, "");
        if self.verbose { crate::cli::output::verbose(&format!("PUT {url}")); }
        self.begin_request();
        let resp = super::middleware::execute_with_retry(self.verbose, || async {
            let expires = auth::generate_expires()?;
            let sig = auth::sign("PUT", &full_path, expires, &body_str, &creds.api_secret)?;
            let headers = self.auth_headers(&creds.api_key, expires, &sig)?;
            self.http.put(&url).headers(headers)
                .header("Content-Type", "application/json")
                .body(body_str.clone()).send().await
                .map_err(|e| BitmexError::Network { message: e.to_string() })
        }).await?;
        self.parse_response(resp, "PUT", &url).await
    }

    /// Authenticated DELETE, optional JSON body.
    pub async fn delete(
        &self,
        path: &str,
        query: &str,
        body: Option<&Value>,
        creds: &Credentials,
    ) -> Result<Value> {
        let full_path = if query.is_empty() {
            format!("/api/v1{path}")
        } else {
            format!("/api/v1{path}?{query}")
        };
        let body_str = body.map(|b| serde_json::to_string(b)).transpose()?.unwrap_or_default();
        let url = self.url(path, query);
        if self.verbose { crate::cli::output::verbose(&format!("DELETE {url}")); }
        self.begin_request();
        let resp = super::middleware::execute_with_retry(self.verbose, || async {
            let expires = auth::generate_expires()?;
            let sig = auth::sign("DELETE", &full_path, expires, &body_str, &creds.api_secret)?;
            let headers = self.auth_headers(&creds.api_key, expires, &sig)?;
            let mut req = self.http.delete(&url).headers(headers);
            if !body_str.is_empty() {
                req = req.header("Content-Type", "application/json").body(body_str.clone());
            }
            req.send().await.map_err(|e| BitmexError::Network { message: e.to_string() })
        }).await?;
        self.parse_response(resp, "DELETE", &url).await
    }

    fn url(&self, path: &str, query: &str) -> String {
        if query.is_empty() {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}{}?{}", self.base_url, path, query)
        }
    }

    fn auth_headers(&self, api_key: &str, expires: u64, sig: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "api-key",
            HeaderValue::from_str(api_key)
                .map_err(|e| BitmexError::Validation { message: format!("Invalid api-key header: {e}") })?,
        );
        headers.insert(
            "api-expires",
            HeaderValue::from_str(&expires.to_string())
                .map_err(|e| BitmexError::Validation { message: format!("Invalid api-expires header: {e}") })?,
        );
        headers.insert(
            "api-signature",
            HeaderValue::from_str(sig)
                .map_err(|e| BitmexError::Validation { message: format!("Invalid api-signature header: {e}") })?,
        );
        Ok(headers)
    }

    /// Reset timing state and record request_start. Call before each request.
    fn begin_request(&self) {
        if let Some(ref handle) = self.timing {
            if let Ok(mut phases) = handle.lock() {
                *phases = super::timing::Phases {
                    request_start: Some(Instant::now()),
                    ..Default::default()
                };
            }
        }
    }

    /// Print timing summary to stderr. Call after body is fully read.
    fn end_request(
        &self,
        method: &str,
        url: &str,
        server_ms: Option<&str>,
        request_id: Option<&str>,
    ) {
        if let Some(ref handle) = self.timing {
            if let Ok(phases) = handle.lock() {
                super::timing::print_timing(&phases, method, url, server_ms, request_id);
            }
        }
    }

    async fn parse_response(&self, resp: reqwest::Response, method: &str, url: &str) -> Result<Value> {
        let status = resp.status();
        let remaining = resp
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok());
        let reset = resp
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let server_ms = resp
            .headers()
            .get("x-envoy-upstream-service-time")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let request_id = resp
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);

        // Record TTFB — response headers have arrived.
        if let Some(ref handle) = self.timing {
            if let Ok(mut phases) = handle.lock() {
                phases.ttfb = Some(Instant::now());
            }
        }

        let body = resp.text().await.map_err(BitmexError::from)?;

        // Record transfer end — body fully received. Print timing now so it
        // appears regardless of whether the response is a success or an error.
        if let Some(ref handle) = self.timing {
            if let Ok(mut phases) = handle.lock() {
                phases.transfer_end = Some(Instant::now());
            }
        }
        self.end_request(method, url, server_ms.as_deref(), request_id.as_deref());

        if self.verbose {
            crate::cli::output::verbose(&format!("Status: {status}"));
            if let Some(r) = remaining {
                crate::cli::output::verbose(&format!("Rate limit remaining: {r}"));
            }
        }

        // Rate limited
        if status.as_u16() == 429 {
            let msg = extract_error_message(&body);
            let mut err = BitmexError::from_bitmex_error(&msg, Some(429));
            // Enrich with reset time if available
            if let (BitmexError::RateLimit { suggestion, .. }, Some(rst)) =
                (&mut err, reset)
            {
                suggestion.push_str(&format!(" Retry after Unix timestamp {rst}."));
            }
            return Err(err);
        }

        // Surface rate limit warning even on success
        if remaining == Some(0) {
            if self.verbose {
                crate::cli::output::verbose("Warning: rate limit budget exhausted.");
            }
        }

        if !status.is_success() {
            let msg = extract_error_message(&body);
            return Err(BitmexError::from_bitmex_error(&msg, Some(status.as_u16())));
        }

        let value: Value = serde_json::from_str(&body)
            .map_err(|e| BitmexError::Parse { message: format!("Failed to parse response JSON: {e}") })?;

        Ok(value)
    }

}

impl ExchangeClient for BitmexClient {
    async fn get(&self, path: &str, query: &str) -> Result<Value> {
        BitmexClient::get(self, path, query).await
    }

    async fn get_auth(&self, path: &str, query: &str, creds: &Credentials) -> Result<Value> {
        BitmexClient::get_auth(self, path, query, creds).await
    }

    async fn post(&self, path: &str, body: &Value, creds: &Credentials) -> Result<Value> {
        BitmexClient::post(self, path, body, creds).await
    }

    async fn put(&self, path: &str, body: &Value, creds: &Credentials) -> Result<Value> {
        BitmexClient::put(self, path, body, creds).await
    }

    async fn delete(
        &self,
        path: &str,
        query: &str,
        body: Option<&Value>,
        creds: &Credentials,
    ) -> Result<Value> {
        BitmexClient::delete(self, path, query, body, creds).await
    }
}

/// Extract a human-readable error message from a BitMEX error response body.
///
/// BitMEX errors typically look like:
///   `{"error":{"message":"Invalid API Key.","name":"HTTPError"}}`
///   or `{"error":"some string"}`
fn extract_error_message(body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(body) {
        if let Some(err) = v.get("error") {
            if let Some(msg) = err.get("message").and_then(|m| m.as_str()) {
                return msg.to_string();
            }
            if let Some(s) = err.as_str() {
                return s.to_string();
            }
        }
    }
    body.to_string()
}

/// Validate and normalise a URL override from a CLI flag or environment variable.
///
/// Accepts `http://` (for local test servers) and `https://` schemes only.
/// Returns `None` if `cli_flag` and `env_val` are both `None`, so callers fall
/// back to their compiled-in default.
pub fn resolve_url_override(
    cli_flag: Option<&str>,
    env_val: Option<&str>,
) -> Result<Option<String>> {
    let raw = match cli_flag.or(env_val) {
        Some(v) => v,
        None => return Ok(None),
    };

    let url = url::Url::parse(raw)?;
    let scheme = url.scheme();

    if scheme != "https" && scheme != "http" {
        return Err(BitmexError::Validation { message: format!(
            "URL must use https:// or http://, got: {raw}"
        ) });
    }

    // Block arbitrary internet hosts unless the danger env var is set.
    if scheme == "http" {
        let host = url.host_str().unwrap_or("");
        let is_local = host == "localhost"
            || host == "127.0.0.1"
            || host == "[::1]"
            || host.ends_with(".local");
        let danger_allowed =
            env::var("BITMEX_DANGER_ALLOW_ANY_URL_HOST").is_ok_and(|v| v == "1");
        if !is_local && !danger_allowed {
            return Err(BitmexError::Validation { message: format!(
                "http:// is only allowed for localhost/127.0.0.1 or when \
                 BITMEX_DANGER_ALLOW_ANY_URL_HOST=1 is set. Got: {raw}"
            ) });
        }
    }

    Ok(Some(raw.trim_end_matches('/').to_string()))
}
