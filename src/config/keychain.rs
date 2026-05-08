/// OS keychain integration for secure credential storage.
///
/// Uses the `keyring` crate which delegates to the platform-native backend:
/// - macOS: Keychain Services (Security.framework)
/// - Linux: libsecret / Secret Service D-Bus API
/// - Windows: Windows Credential Manager
///
/// All api_secrets for all profiles are stored together as a single JSON entry:
///
///   service: "bitmex-cli"  account: "secrets"
///   value:   {"default":"secret1","testnet-trading":"secret2"}
///
/// This means a single keychain prompt regardless of how many profiles exist.
/// api_keys are not sensitive and are stored as plaintext in the TOML config.
///
/// Every public function accepts `no_keychain: bool`. When `true` the keychain
/// is bypassed entirely and the fallback plaintext TOML path is used — useful
/// for CI, Docker, and headless environments (`--no-keychain` / `BITMEX_NO_KEYCHAIN=1`).
use std::collections::HashMap;
use std::sync::Mutex;

use keyring::{Entry, Error as KeyringError};

use crate::errors::{BitmexError, Result};

const SERVICE: &str = "bitmex-cli";
const SECRETS_ACCOUNT: &str = "secrets";

fn secrets_entry() -> std::result::Result<Entry, KeyringError> {
    Entry::new(SERVICE, SECRETS_ACCOUNT)
}

pub(crate) type SecretsMap = HashMap<String, String>;

/// Process-level cache: the keychain is read at most once per process.
/// Writes update both the cache and the keychain.
static CACHE: Mutex<Option<Option<SecretsMap>>> = Mutex::new(None);

fn read_secrets_map(no_keychain: bool) -> Result<Option<SecretsMap>> {
    if no_keychain {
        return Ok(None);
    }

    {
        let guard = CACHE.lock().unwrap();
        if let Some(cached) = &*guard {
            return Ok(cached.clone());
        }
    }

    let entry = secrets_entry()
        .map_err(|e| BitmexError::Config { message: format!("Keychain entry error: {e}") })?;
    let result = match entry.get_password() {
        Ok(json) => {
            let map: SecretsMap = serde_json::from_str(&json).unwrap_or_default();
            Ok(Some(map))
        }
        Err(KeyringError::NoEntry) => Ok(Some(SecretsMap::new())),
        Err(KeyringError::PlatformFailure(_) | KeyringError::NoStorageAccess(_)) => {
            crate::cli::output::warn(
                "Keychain unavailable — falling back to config file credentials.",
            );
            Ok(None)
        }
        Err(e) => Err(BitmexError::Config {
            message: format!("Failed to read secrets from keychain: {e}"),
        }),
    };

    if let Ok(ref value) = result {
        *CACHE.lock().unwrap() = Some(value.clone());
    }

    result
}

fn write_secrets_map(map: &SecretsMap) -> Result<()> {
    let entry = secrets_entry()
        .map_err(|e| BitmexError::Config { message: format!("Keychain entry error: {e}") })?;
    let json = serde_json::to_string(map).expect("serialization is infallible");
    entry.set_password(&json).map_err(|e| BitmexError::Config {
        message: format!("Failed to write secrets to keychain: {e}"),
    })?;
    // Update cache to reflect the write.
    *CACHE.lock().unwrap() = Some(Some(map.clone()));
    Ok(())
}

/// Store `api_secret` for `profile` in the shared keychain entry (one prompt).
/// The `api_key` is intentionally ignored here — callers must persist it in
/// the TOML config.
///
/// Returns:
/// - `Ok(true)`  — stored successfully
/// - `Ok(false)` — keychain unavailable; caller should fall back to plaintext
pub(crate) fn store_profile(
    profile: &str,
    _api_key: &str,
    api_secret: &str,
    no_keychain: bool,
) -> Result<bool> {
    if no_keychain {
        return Ok(false);
    }

    let mut map = match read_secrets_map(no_keychain)? {
        Some(m) => m,
        None => return Ok(false),
    };

    map.insert(profile.to_string(), api_secret.to_string());

    match write_secrets_map(&map) {
        Ok(()) => {}
        Err(_) => return Ok(false),
    }

    Ok(true)
}

/// Retrieve the `api_secret` for `profile` from the shared keychain entry.
///
/// Returns:
/// - `Ok(Some(secret))` — found
/// - `Ok(None)`         — not found or keychain unavailable
pub(crate) fn load_secret(profile: &str, no_keychain: bool) -> Result<Option<String>> {
    Ok(load_all_secrets(no_keychain)?.and_then(|m| m.get(profile).cloned()))
}

/// Read the full secrets map in one keychain access. Use this when checking
/// multiple profiles at once to avoid repeated prompts.
pub(crate) fn load_all_secrets(no_keychain: bool) -> Result<Option<SecretsMap>> {
    read_secrets_map(no_keychain)
}

/// Delete the `profile` entry from the shared keychain secrets map.
pub(crate) fn delete_profile(profile: &str, no_keychain: bool) -> Result<()> {
    if no_keychain {
        return Ok(());
    }

    if let Some(mut map) = read_secrets_map(no_keychain)? {
        if map.remove(profile).is_some() {
            let _ = write_secrets_map(&map);
        }
    }

    Ok(())
}

/// Returns `true` if the OS keychain backend is accessible on this platform.
pub(crate) fn is_available() -> bool {
    match Entry::new(SERVICE, "__probe__") {
        Err(_) => false,
        Ok(entry) => match entry.get_password() {
            Ok(_) | Err(KeyringError::NoEntry) => true,
            Err(_) => false,
        },
    }
}
