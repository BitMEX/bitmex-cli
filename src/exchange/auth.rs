/// Authentication and signing for the BitMEX REST and WebSocket APIs.
///
/// Signature: hex(HMAC-SHA256(secret, verb + path_with_query + expires + body))
/// Headers:   api-key, api-expires, api-signature
///
/// The secret is used as raw UTF-8 bytes (not base64-decoded).
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::config::SecretValue;
use crate::errors::{BitmexError, Result};

type HmacSha256 = Hmac<Sha256>;

/// Returns `now + 5` seconds as a Unix timestamp.
/// BitMEX rejects requests where `api-expires` is in the past.
pub(crate) fn generate_expires() -> Result<u64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| BitmexError::Auth { message: "system clock is before Unix epoch".into() })?
        .as_secs();
    Ok(now + 5)
}

/// Compute the BitMEX `api-signature` header value.
///
/// - `verb`    — HTTP method in uppercase, e.g. "GET"
/// - `path`    — full path including query string, e.g. "/api/v1/order?symbol=XBTUSD"
/// - `expires` — Unix seconds (from `generate_expires()`)
/// - `body`    — raw request body string (empty string for GET/DELETE with no body)
/// - `secret`  — raw API secret (not base64-encoded)
pub(crate) fn sign(
    verb: &str,
    path: &str,
    expires: u64,
    body: &str,
    secret: &SecretValue,
) -> Result<String> {
    let message = format!("{}{}{}{}", verb.to_uppercase(), path, expires, body);
    let mut mac = HmacSha256::new_from_slice(secret.expose().as_bytes())
        .map_err(|e| BitmexError::Auth { message: format!("HMAC key error: {e}") })?;
    mac.update(message.as_bytes());
    let result = mac.finalize().into_bytes();
    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-good vectors verified with:
    //   echo -n "GET/api/v1/order1518064236" | openssl dgst -sha256 -hmac "secret"
    //   echo -n 'POST/api/v1/order1518064238{"symbol":"XBTUSD"}' | openssl dgst -sha256 -hmac "secret"
    #[test]
    fn sign_get_no_body() {
        let secret = crate::config::SecretValue::new("secret".to_string());
        let sig = sign("GET", "/api/v1/order", 1518064236, "", &secret).unwrap();
        assert_eq!(
            sig,
            "05b241833763473e08eff9bc87f8850ae402c50daeccc7f3afdc998b4e937122"
        );
    }

    #[test]
    fn sign_post_with_body() {
        let secret = crate::config::SecretValue::new("secret".to_string());
        let sig = sign(
            "POST",
            "/api/v1/order",
            1518064238,
            r#"{"symbol":"XBTUSD"}"#,
            &secret,
        )
        .unwrap();
        assert_eq!(
            sig,
            "37a90d9a89f8dc9d6e2edf88dc8609e8ee7810b9e3f42e66d2d41f83ab64b01e"
        );
    }

    #[test]
    fn sign_verb_is_uppercased() {
        let secret = crate::config::SecretValue::new("secret".to_string());
        let sig_lower = sign("get", "/api/v1/order", 1518064236, "", &secret).unwrap();
        let sig_upper = sign("GET", "/api/v1/order", 1518064236, "", &secret).unwrap();
        assert_eq!(sig_lower, sig_upper);
    }

    #[test]
    fn expires_is_future() {
        let exp = generate_expires().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(exp > now);
        assert!(exp <= now + 10);
    }

    #[test]
    fn signature_is_64_hex_chars() {
        let secret = crate::config::SecretValue::new("any_secret".to_string());
        let sig = sign("GET", "/api/v1/position", 9999999999, "", &secret).unwrap();
        assert_eq!(sig.len(), 64);
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
