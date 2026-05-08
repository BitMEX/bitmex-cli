/// Request execution pipeline with consistent retry across all HTTP methods.
///
/// Previously, `get` and `get_auth` had inline retry loops while `post`, `put`,
/// `patch`, and `delete` had none. This module centralises the logic so every
/// HTTP method gets the same retry and backoff behaviour.
///
/// `build_and_send` is called on **every** attempt so that auth headers —
/// which embed a fresh `api-expires` timestamp — are re-signed on each retry.
use std::future::Future;
use std::time::Duration;

use crate::errors::{BitmexError, Result};

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 500;

/// Execute an HTTP request with exponential-backoff retry for transient failures.
///
/// The closure `build_and_send` is responsible for building **and sending** the
/// request, including auth header generation when needed. It is invoked on every
/// attempt so that timestamp-sensitive auth headers stay fresh.
///
/// Retries on:
/// - 5xx server errors
/// - transient network errors (timeout, connection refused)
///
/// Does **not** retry on auth errors, validation errors, or 4xx responses.
pub(crate) async fn execute_with_retry<F, Fut>(verbose: bool, build_and_send: F) -> Result<reqwest::Response>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<reqwest::Response>>,
{
    let mut attempt = 0u32;
    loop {
        match build_and_send().await {
            Ok(r) if r.status().is_server_error() && attempt < MAX_RETRIES => {
                attempt += 1;
                backoff(verbose, attempt).await;
            }
            Ok(r) => return Ok(r),
            Err(BitmexError::Network { .. }) if attempt < MAX_RETRIES => {
                attempt += 1;
                backoff(verbose, attempt).await;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn backoff(verbose: bool, attempt: u32) {
    let ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
    if verbose {
        crate::cli::output::verbose(&format!(
            "Retry {attempt}/{MAX_RETRIES} after {ms}ms"
        ));
    }
    tokio::time::sleep(Duration::from_millis(ms)).await;
}
