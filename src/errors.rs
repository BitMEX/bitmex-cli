/// Unified error types for the BitMEX CLI.
///
/// Error categories cover API responses, authentication failures, network
/// issues, rate-limiting, validation, configuration, WebSocket errors,
/// I/O, and response parsing problems.
///
/// Uses snafu for explicit context at every error site, preserving the full
/// source chain rather than collapsing it to a string via `.to_string()`.
use std::fmt;

use snafu::Snafu;

/// Top-level error categories used in JSON error envelopes and exit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Api,
    Auth,
    Network,
    RateLimit,
    Validation,
    Config,
    WebSocket,
    Io,
    Parse,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Api => "api",
            Self::Auth => "auth",
            Self::Network => "network",
            Self::RateLimit => "rate_limit",
            Self::Validation => "validation",
            Self::Config => "config",
            Self::WebSocket => "websocket",
            Self::Io => "io",
            Self::Parse => "parse",
        };
        f.write_str(s)
    }
}

/// The primary error type for all CLI operations.
///
/// Snafu generates context selector structs for each variant:
/// `ApiSnafu`, `AuthSnafu`, `NetworkSnafu`, etc.
/// Use them with `.context(AuthSnafu { message: "..." })` on `Result` values,
/// or construct variants directly as `BitmexError::Auth { message: "...".into() }`.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum BitmexError {
    #[snafu(display("{message}"))]
    Api {
        category: ErrorCategory,
        message: String,
    },

    #[snafu(display("Authentication failed: {message}"))]
    Auth { message: String },

    #[snafu(display("Network error: {message}"))]
    Network { message: String },

    #[snafu(display("{message}"))]
    RateLimit {
        message: String,
        suggestion: String,
        retryable: bool,
        docs_url: String,
    },

    #[snafu(display("Validation error: {message}"))]
    Validation { message: String },

    #[snafu(display("Configuration error: {message}"))]
    Config { message: String },

    #[snafu(display("WebSocket error: {message}"))]
    WebSocket { message: String },

    #[snafu(display("I/O error: {source}"))]
    Io { source: std::io::Error },

    #[snafu(display("Parse error: {message}"))]
    Parse { message: String },
}

impl BitmexError {
    /// Returns the error category for JSON envelope output.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Api { category, .. } => *category,
            Self::Auth { .. } => ErrorCategory::Auth,
            Self::Network { .. } => ErrorCategory::Network,
            Self::RateLimit { .. } => ErrorCategory::RateLimit,
            Self::Validation { .. } => ErrorCategory::Validation,
            Self::Config { .. } => ErrorCategory::Config,
            Self::WebSocket { .. } => ErrorCategory::WebSocket,
            Self::Io { .. } => ErrorCategory::Io,
            Self::Parse { .. } => ErrorCategory::Parse,
        }
    }

    /// Constructs an error from a BitMEX API error message and optional HTTP status code.
    ///
    /// BitMEX returns errors as JSON: `{"error": {"message": "...", "name": "HTTPError"}}`
    /// or sometimes as a plain string in the `error` field.
    pub(crate) fn from_bitmex_error(message: &str, status: Option<u16>) -> Self {
        // HTTP 429 is always rate limited
        if status == Some(429) {
            return Self::RateLimit {
                message: format!("BitMEX rate limit exceeded: {message}"),
                suggestion: "BitMEX allows 300 requests per 5 minutes. \
                    Check x-ratelimit-remaining and x-ratelimit-reset headers. \
                    Use WebSocket streaming for real-time data instead of REST polling. \
                    Back off and retry after x-ratelimit-reset (Unix seconds)."
                    .to_string(),
                retryable: true,
                docs_url: "https://www.bitmex.com/app/restAPI#Rate-Limits".to_string(),
            };
        }

        // Auth errors
        if message.contains("Invalid API Key")
            || message.contains("Signature not valid")
            || message.contains("Access Denied")
            || message.contains("not signed in")
            || message.contains("invalid api_key")
        {
            return Self::Auth {
                message: message.to_string(),
            };
        }

        // Overloaded / service unavailable
        if message.contains("overloaded")
            || message.contains("Service Unavailable")
            || status == Some(503)
        {
            return Self::RateLimit {
                message: format!("BitMEX service temporarily unavailable: {message}"),
                suggestion: "The exchange is under high load. Wait 5-30 seconds before retrying."
                    .to_string(),
                retryable: true,
                docs_url: "https://www.bitmex.com/app/restAPI".to_string(),
            };
        }

        Self::Api {
            category: ErrorCategory::Api,
            message: message.to_string(),
        }
    }

    /// Builds the JSON error envelope.
    ///
    /// Rate limit errors include additional fields that LLM agents can use
    /// to adapt their strategy: `suggestion`, `retryable`, and `docs_url`.
    pub(crate) fn to_json_envelope(&self) -> serde_json::Value {
        match self {
            Self::RateLimit {
                message,
                suggestion,
                retryable,
                docs_url,
            } => serde_json::json!({
                "error": "rate_limit",
                "message": message,
                "suggestion": suggestion,
                "retryable": retryable,
                "docs_url": docs_url,
            }),
            _ => serde_json::json!({
                "error": self.category().to_string(),
                "message": self.to_string(),
            }),
        }
    }
}

impl From<reqwest::Error> for BitmexError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for BitmexError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parse {
            message: err.to_string(),
        }
    }
}

impl From<url::ParseError> for BitmexError {
    fn from(err: url::ParseError) -> Self {
        Self::Validation {
            message: format!("Invalid URL: {err}"),
        }
    }
}

impl From<toml::de::Error> for BitmexError {
    fn from(err: toml::de::Error) -> Self {
        Self::Config {
            message: format!("TOML parse error: {err}"),
        }
    }
}

impl From<std::io::Error> for BitmexError {
    fn from(err: std::io::Error) -> Self {
        Self::Io { source: err }
    }
}

pub type Result<T> = std::result::Result<T, BitmexError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bitmex_error_rate_limit_429() {
        let err = BitmexError::from_bitmex_error("Rate limit exceeded", Some(429));
        assert_eq!(err.category(), ErrorCategory::RateLimit);
        if let BitmexError::RateLimit {
            retryable,
            docs_url,
            suggestion,
            ..
        } = &err
        {
            assert!(retryable);
            assert!(docs_url.contains("Rate-Limits"));
            assert!(suggestion.contains("300 requests"));
        } else {
            panic!("Expected RateLimit variant, got {err:?}");
        }
    }

    #[test]
    fn from_bitmex_error_invalid_api_key() {
        let err = BitmexError::from_bitmex_error("Invalid API Key.", None);
        assert_eq!(err.category(), ErrorCategory::Auth);
    }

    #[test]
    fn from_bitmex_error_signature_not_valid() {
        let err = BitmexError::from_bitmex_error("Signature not valid.", None);
        assert_eq!(err.category(), ErrorCategory::Auth);
    }

    #[test]
    fn from_bitmex_error_503_overloaded() {
        let err = BitmexError::from_bitmex_error("overloaded", Some(503));
        assert_eq!(err.category(), ErrorCategory::RateLimit);
        if let BitmexError::RateLimit { retryable, .. } = &err {
            assert!(retryable);
        }
    }

    #[test]
    fn from_bitmex_error_unknown_falls_to_api() {
        let err = BitmexError::from_bitmex_error("Unknown order", None);
        assert_eq!(err.category(), ErrorCategory::Api);
    }

    #[test]
    fn rate_limit_envelope_has_all_fields() {
        let err = BitmexError::RateLimit {
            message: "test".to_string(),
            suggestion: "wait".to_string(),
            retryable: true,
            docs_url: "https://example.com".to_string(),
        };
        let envelope = err.to_json_envelope();
        assert_eq!(envelope["error"], "rate_limit");
        assert_eq!(envelope["message"], "test");
        assert_eq!(envelope["suggestion"], "wait");
        assert_eq!(envelope["retryable"], true);
        assert_eq!(envelope["docs_url"], "https://example.com");
    }

    #[test]
    fn non_rate_limit_envelope_has_error_and_message() {
        let err = BitmexError::Auth {
            message: "bad key".to_string(),
        };
        let envelope = err.to_json_envelope();
        assert_eq!(envelope["error"], "auth");
        assert!(envelope["message"].as_str().unwrap().contains("bad key"));
        assert!(envelope.get("suggestion").is_none());
    }
}
