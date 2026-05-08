/// Configuration management for `~/.config/bitmex/config.toml`.
///
/// Credentials are stored in the OS keychain via `keychain.rs`.
/// The TOML file holds only non-sensitive data: settings, active profile name,
/// and per-profile metadata (name, testnet flag).
///
/// On first run after an upgrade, if the legacy `[auth]` section with plaintext
/// credentials is detected it is automatically migrated to the keychain and
/// removed from the file.

pub(crate) mod keychain;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use figment::{providers::{Format, Toml}, Figment};
use serde::{Deserialize, Serialize};

use crate::errors::{BitmexError, Result};

// ---------------------------------------------------------------------------
// On-disk structs
// ---------------------------------------------------------------------------

/// Top-level config file schema.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct BitmexConfig {
    /// Name of the currently active profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) active_profile: Option<String>,

    #[serde(default)]
    pub(crate) settings: SettingsConfig,

    /// Named credential profiles (non-sensitive metadata only when keychain available).
    #[serde(default)]
    pub(crate) profiles: Vec<ProfileConfig>,

    /// Legacy `[auth]` section — deserialised for migration, never re-serialised.
    #[serde(default, skip_serializing)]
    pub(crate) auth: LegacyAuthConfig,
}

/// Per-profile metadata stored in TOML.
///
/// `api_key` and `api_secret` are present only when the OS keychain is
/// unavailable (headless/CI fallback). They are omitted from the file when
/// keychain storage succeeds.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub(crate) struct ProfileConfig {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) testnet: bool,
    /// Plaintext fallback — written only when keychain is unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) api_key: Option<String>,
    /// Plaintext fallback — written only when keychain is unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) api_secret: Option<String>,
}

impl std::fmt::Display for ProfileConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// General settings (non-credential).
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct SettingsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) default_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) output: Option<String>,
}

/// Legacy `[auth]` section — only used for one-time migration from old format.
/// Uses `Deserialize` only; never written back.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct LegacyAuthConfig {
    #[serde(default)]
    pub(crate) api_key: Option<String>,
    #[serde(default)]
    pub(crate) api_secret: Option<String>,
}

// ---------------------------------------------------------------------------
// Credential types
// ---------------------------------------------------------------------------

/// Wrapper that keeps secrets out of Debug, Display, and Serialize output.
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(val: String) -> Self { Self(val) }
    /// Provides read-only access to the secret. Callers must not log the result.
    pub fn expose(&self) -> &str { &self.0 }
}

impl std::fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl std::fmt::Display for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

/// Resolved credentials for BitMEX API calls.
#[derive(Debug)]
pub struct Credentials {
    pub api_key: String,
    pub api_secret: SecretValue,
    pub source: CredentialSource,
}

/// Where the credentials were resolved from.
#[derive(Debug, Clone)]
pub enum CredentialSource {
    Flag,
    Env,
    /// Found in OS keychain for the named profile.
    Keychain(String),
    /// Keychain unavailable; read from plaintext TOML fallback for the named profile.
    KeychainFallback(String),
}

impl std::fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flag => write!(f, "command-line flag"),
            Self::Env => write!(f, "environment variable"),
            Self::Keychain(p) => write!(f, "OS keychain (profile: {p})"),
            Self::KeychainFallback(p) => {
                write!(f, "config file fallback (profile: {p}, keychain unavailable)")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

/// Returns the config directory path: `~/.config/bitmex/`.
pub(crate) fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| BitmexError::Config { message: "Cannot determine config directory".into() })?;
    Ok(base.join("bitmex"))
}

/// Returns the full path to the config file.
pub(crate) fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

// ---------------------------------------------------------------------------
// Load / save
// ---------------------------------------------------------------------------

/// Load configuration from `~/.config/bitmex/config.toml`.
///
/// Runs auto-migration if a legacy `[auth]` section is found.
pub(crate) fn load() -> Result<BitmexConfig> {
    let path = config_path()?;
    let mut cfg: BitmexConfig = Figment::new()
        .merge(Toml::file(&path))
        .extract()
        .map_err(|e| BitmexError::Config { message: format!("Config load error: {e}") })?;

    maybe_migrate_legacy_auth(&mut cfg, &path)?;
    Ok(cfg)
}

/// Save configuration to disk atomically with 0600 permissions.
pub(crate) fn save(cfg: &BitmexConfig) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join("config.toml");
    let contents = toml::to_string_pretty(cfg)
        .map_err(|e| BitmexError::Config { message: format!("TOML serialize error: {e}") })?;
    atomic_write_restricted(&path, contents.as_bytes())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Auto-migration
// ---------------------------------------------------------------------------

fn maybe_migrate_legacy_auth(cfg: &mut BitmexConfig, path: &Path) -> Result<()> {
    let (Some(key), Some(secret)) = (
        cfg.auth.api_key.take(),
        cfg.auth.api_secret.take(),
    ) else {
        return Ok(());
    };

    let stored_in_keychain = keychain::store_profile("default", &key, &secret, false)
        .unwrap_or(false);

    // Ensure "default" profile entry exists in [[profiles]]
    upsert_profile(cfg, "default", false);

    if !stored_in_keychain {
        // Keychain unavailable — keep credentials in plaintext profile fallback
        if let Some(p) = cfg.profiles.iter_mut().find(|p| p.name == "default") {
            p.api_key = Some(key);
            p.api_secret = Some(secret);
        }
    }

    if cfg.active_profile.is_none() {
        cfg.active_profile = Some("default".into());
    }

    // Save without [auth] (LegacyAuthConfig is skip_serializing)
    save(cfg)?;

    let location = if stored_in_keychain {
        "OS keychain (profile: \"default\")"
    } else {
        "config.toml profile section (keychain unavailable)"
    };
    eprintln!("Notice: plaintext credentials migrated to {location}. [auth] section removed.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Profile helpers
// ---------------------------------------------------------------------------

/// Insert or update a profile entry (non-credential fields only).
pub(crate) fn upsert_profile(cfg: &mut BitmexConfig, name: &str, testnet: bool) {
    if let Some(p) = cfg.profiles.iter_mut().find(|p| p.name == name) {
        p.testnet = testnet;
    } else {
        cfg.profiles.push(ProfileConfig {
            name: name.to_string(),
            testnet,
            api_key: None,
            api_secret: None,
        });
    }
}

/// Find a profile by name.
pub(crate) fn get_profile<'a>(cfg: &'a BitmexConfig, name: &str) -> Option<&'a ProfileConfig> {
    cfg.profiles.iter().find(|p| p.name == name)
}

/// Returns the effective testnet flag for a profile, ORed with the CLI flag.
pub fn effective_testnet(profile: Option<&str>, cli_testnet: bool) -> bool {
    if cli_testnet {
        return true;
    }
    let Ok(cfg) = load() else { return false };
    let effective = effective_profile_name(profile, &cfg);
    get_profile(&cfg, effective)
        .map(|p| p.testnet)
        .unwrap_or(false)
}

fn effective_profile_name<'a>(profile: Option<&'a str>, cfg: &'a BitmexConfig) -> &'a str {
    profile
        .or_else(|| cfg.active_profile.as_deref())
        .unwrap_or("default")
}

// ---------------------------------------------------------------------------
// Credential resolution
// ---------------------------------------------------------------------------

/// Resolve credentials using precedence: flag > env > profile (keychain → TOML fallback).
///
/// `profile`     — value of `--profile` flag (overrides active_profile from config)
/// `flag_key`    — value of `--api-key` flag
/// `flag_secret` — value of `--api-secret` flag
/// `no_keychain` — skip keychain, force TOML fallback (`--no-keychain` / `BITMEX_NO_KEYCHAIN`)
pub(crate) fn resolve_credentials(
    profile: Option<&str>,
    flag_key: Option<&str>,
    flag_secret: Option<&str>,
    no_keychain: bool,
) -> Result<Credentials> {
    // 1. Explicit flags — highest precedence
    match (flag_key, flag_secret) {
        (Some(k), Some(s)) => {
            return Ok(Credentials {
                api_key: k.to_string(),
                api_secret: SecretValue::new(s.to_string()),
                source: CredentialSource::Flag,
            });
        }
        (Some(_), None) => crate::cli::output::warn(
            "--api-key provided without --api-secret. \
             Flag credentials ignored, falling back to env/profile.",
        ),
        (None, Some(_)) => crate::cli::output::warn(
            "--api-secret provided without --api-key. \
             Flag credentials ignored, falling back to env/profile.",
        ),
        (None, None) => {}
    }

    // 2. Environment variables
    let env_key = env::var("BITMEX_API_KEY").ok();
    let env_secret = env::var("BITMEX_API_SECRET").ok();
    if let (Some(k), Some(s)) = (env_key, env_secret) {
        return Ok(Credentials {
            api_key: k,
            api_secret: SecretValue::new(s),
            source: CredentialSource::Env,
        });
    }

    // 3. Profile lookup — api_key from TOML, api_secret from keychain (one prompt)
    let cfg = load()?;
    let effective = effective_profile_name(profile, &cfg).to_string();

    // Try secret from keychain + key from TOML config.
    if let Some(secret) = keychain::load_secret(&effective, no_keychain)? {
        if let Some(p) = get_profile(&cfg, &effective) {
            if let Some(k) = p.api_key.clone() {
                return Ok(Credentials {
                    api_key: k,
                    api_secret: SecretValue::new(secret),
                    source: CredentialSource::Keychain(effective),
                });
            }
        }
    }

    // Fall back to plaintext in ProfileConfig
    if let Some(p) = get_profile(&cfg, &effective) {
        if let (Some(k), Some(s)) = (p.api_key.clone(), p.api_secret.clone()) {
            return Ok(Credentials {
                api_key: k,
                api_secret: SecretValue::new(s),
                source: CredentialSource::KeychainFallback(effective),
            });
        }
    }

    Err(BitmexError::Auth {
        message: format!(
            "No credentials found for profile \"{effective}\". \
             Run `bitmex auth add` or set BITMEX_API_KEY / BITMEX_API_SECRET."
        ),
    })
}

/// Remove credentials for the "default" profile (backward-compat `auth reset`).
pub(crate) fn reset_auth() -> Result<()> {
    delete_profile_creds("default", false)
}

/// Delete all credential data for a named profile (keychain + TOML fallback fields).
pub(crate) fn delete_profile_creds(name: &str, no_keychain: bool) -> Result<()> {
    keychain::delete_profile(name, no_keychain)?;
    let path = config_path()?;
    if !path.exists() {
        return Ok(());
    }
    let mut cfg = load()?;
    cfg.profiles.retain(|p| p.name != name);
    if cfg.active_profile.as_deref() == Some(name) {
        cfg.active_profile = None;
    }
    save(&cfg)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Stdin / file secret readers
// ---------------------------------------------------------------------------

/// Read a secret from stdin (one line, trimmed). Returns an error on EOF/empty input.
pub fn read_secret_from_stdin() -> Result<SecretValue> {
    let mut buf = String::new();
    let n = std::io::stdin().read_line(&mut buf).map_err(BitmexError::from)?;
    let trimmed = buf.trim().to_string();
    if n == 0 || trimmed.is_empty() {
        return Err(BitmexError::Auth {
            message: "Empty secret received from stdin — did you forget to pipe input?".into(),
        });
    }
    Ok(SecretValue::new(trimmed))
}

/// Read a secret from a file path. Returns an error if the file is empty.
pub fn read_secret_from_file(path: &Path) -> Result<SecretValue> {
    let contents = fs::read_to_string(path)?;
    let trimmed = contents.trim().to_string();
    if trimmed.is_empty() {
        return Err(BitmexError::Auth {
            message: format!("Empty secret read from file: {}", path.display()),
        });
    }
    Ok(SecretValue::new(trimmed))
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Mask a string for display, showing only the first 4 and last 4 characters.
pub(crate) fn mask_string(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= 8 {
        return "****".to_string();
    }
    let prefix: String = chars[..4].iter().collect();
    let suffix: String = chars[chars.len() - 4..].iter().collect();
    format!("{prefix}...{suffix}")
}

#[cfg(unix)]
fn atomic_write_restricted(path: &Path, data: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let dir = path
        .parent()
        .ok_or_else(|| BitmexError::Config { message: "config path has no parent directory".into() })?;
    let tmp_path = dir.join(".config.tmp");

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&tmp_path)?;
    file.write_all(data)?;
    file.sync_all()?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

#[cfg(not(unix))]
fn atomic_write_restricted(path: &Path, data: &[u8]) -> Result<()> {
    fs::write(path, data)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_value_debug_is_redacted() {
        let secret = SecretValue::new("my_actual_secret".to_string());
        assert_eq!(format!("{:?}", secret), "[REDACTED]");
        assert!(!format!("{:?}", secret).contains("my_actual_secret"));
    }

    #[test]
    fn secret_value_display_is_redacted() {
        let secret = SecretValue::new("my_actual_secret".to_string());
        assert_eq!(format!("{}", secret), "[REDACTED]");
    }

    #[test]
    fn mask_string_short() {
        assert_eq!(mask_string("abc"), "****");
    }

    #[test]
    fn mask_string_long() {
        assert_eq!(mask_string("abcdefghij"), "abcd...ghij");
    }

    #[test]
    fn config_path_resolves_to_config_toml() {
        let path = config_path().expect("config_path() should succeed on any desktop OS");
        assert!(path.ends_with("config.toml"));
        assert!(path.parent().unwrap().ends_with("bitmex"));
    }

    #[test]
    fn config_roundtrip_new_schema() {
        let mut cfg = BitmexConfig {
            active_profile: Some("default".into()),
            settings: SettingsConfig {
                default_symbol: Some("XBTUSD".into()),
                ..Default::default()
            },
            profiles: vec![
                ProfileConfig { name: "default".into(), testnet: false, api_key: None, api_secret: None },
                ProfileConfig { name: "testnet".into(), testnet: true, api_key: None, api_secret: None },
            ],
            auth: LegacyAuthConfig::default(),
        };
        let serialized = toml::to_string_pretty(&cfg).unwrap();
        // [auth] must not appear (skip_serializing)
        assert!(!serialized.contains("[auth]"), "legacy [auth] should not be serialized");
        assert!(serialized.contains("active_profile"));
        assert!(serialized.contains("XBTUSD"));

        let deserialized: BitmexConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.active_profile.as_deref(), Some("default"));
        assert_eq!(deserialized.settings.default_symbol.as_deref(), Some("XBTUSD"));
        assert_eq!(deserialized.profiles.len(), 2);
        assert!(deserialized.profiles[1].testnet);

        // Legacy [auth] roundtrip — can be read but is not written
        let legacy_toml = r#"
[auth]
api_key = "test_key"
api_secret = "test_secret"
[settings]
default_symbol = "XBTUSD"
"#;
        let legacy: BitmexConfig = toml::from_str(legacy_toml).unwrap();
        assert_eq!(legacy.auth.api_key.as_deref(), Some("test_key"));
        // Re-serializing should NOT include [auth]
        let reserialized = toml::to_string_pretty(&legacy).unwrap();
        assert!(!reserialized.contains("test_key"), "migration should remove plaintext key");

        // upsert_profile
        upsert_profile(&mut cfg, "ci", true);
        assert_eq!(cfg.profiles.len(), 3);
        assert!(cfg.profiles.iter().any(|p| p.name == "ci" && p.testnet));
        upsert_profile(&mut cfg, "ci", false); // update
        assert_eq!(cfg.profiles.iter().find(|p| p.name == "ci").unwrap().testnet, false);
    }

    #[test]
    fn profile_config_no_plaintext_when_none() {
        let cfg = BitmexConfig {
            profiles: vec![ProfileConfig {
                name: "default".into(),
                testnet: false,
                api_key: None,
                api_secret: None,
            }],
            ..Default::default()
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        assert!(!s.contains("api_key"), "api_key should be omitted when None");
        assert!(!s.contains("api_secret"), "api_secret should be omitted when None");
    }

    #[test]
    fn credential_source_display() {
        assert_eq!(CredentialSource::Flag.to_string(), "command-line flag");
        assert_eq!(CredentialSource::Env.to_string(), "environment variable");
        assert_eq!(
            CredentialSource::Keychain("default".into()).to_string(),
            "OS keychain (profile: default)"
        );
        assert_eq!(
            CredentialSource::KeychainFallback("ci".into()).to_string(),
            "config file fallback (profile: ci, keychain unavailable)"
        );
    }
}
