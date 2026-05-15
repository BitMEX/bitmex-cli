/// `bitmex auth` subcommands: add, set, list, show, use, delete, reset.
use clap::Subcommand;

use crate::config::{self, keychain, mask_string};
use crate::errors::{BitmexError, Result};
use crate::cli::output::CommandOutput;
use crate::AppContext;

#[derive(Debug, Subcommand)]
pub(crate) enum AuthCommand {
    /// Interactively guided wizard to add a new credential profile.
    Add {
        /// Profile name (prompted if not supplied).
        #[arg(long)]
        profile: Option<String>,
        /// Mark this profile as targeting testnet.
        #[arg(long)]
        testnet: bool,
    },
    /// Save credentials to a profile (non-interactive; use flags or prompts).
    Set {
        /// Profile name [default: "default"].
        #[arg(long)]
        profile: Option<String>,
        /// Mark this profile as targeting testnet.
        #[arg(long)]
        testnet: bool,
    },
    /// List all credential profiles.
    List,
    /// Show masked credentials for a profile.
    Show {
        /// Profile name [defaults to active profile].
        #[arg(long)]
        profile: Option<String>,
    },
    /// Set the active default profile.
    Use {
        /// Profile name to activate.
        name: String,
    },
    /// Delete a credential profile from the keychain and config.
    Delete {
        /// Profile name to delete.
        #[arg(long)]
        profile: String,
    },
    /// Remove all stored credentials (clears the "default" profile).
    Reset,
}

pub(crate) async fn run(cmd: AuthCommand, ctx: &AppContext) -> Result<CommandOutput> {
    match cmd {
        // ------------------------------------------------------------------
        // Add — interactive guided wizard
        // ------------------------------------------------------------------
        AuthCommand::Add { profile, testnet } => {
            println!("BitMEX credential wizard");
            println!("========================");
            println!();

            // Profile name
            let profile_name = if let Some(p) = profile.or_else(|| ctx.profile.clone()) {
                p
            } else {
                inquire::Text::new("Profile name")
                    .with_default("default")
                    .with_help_message("A short label, e.g. \"default\", \"testnet\", \"trading-bot\"")
                    .prompt()
                    .map_err(|e| BitmexError::Config { message: format!("Prompt error: {e}") })?
            };

            // Testnet?
            let name_implies_testnet = profile_name.to_lowercase().contains("testnet");
            let is_testnet = if testnet || ctx.testnet || name_implies_testnet {
                true
            } else {
                inquire::Confirm::new("Is this a testnet profile?")
                    .with_default(false)
                    .prompt()
                    .map_err(|e| BitmexError::Config { message: format!("Prompt error: {e}") })?
            };

            // API key
            let key_url = if is_testnet {
                "https://testnet.bitmex.com/app/apiKeys"
            } else {
                "https://www.bitmex.com/app/apiKeys"
            };
            println!();
            println!("Create API keys at: {key_url}");
            println!();

            let api_key = if let Some(k) = ctx.api_key.clone() {
                k
            } else {
                inquire::Text::new("API Key")
                    .with_help_message("Paste your BitMEX API key")
                    .prompt()
                    .map_err(|e| BitmexError::Config { message: format!("Prompt error: {e}") })?
            };

            let api_secret = if let Some(s) = ctx.api_secret.clone() {
                s
            } else {
                inquire::Password::new("API Secret")
                    .without_confirmation()
                    .with_help_message("Paste your BitMEX API secret (hidden)")
                    .prompt()
                    .map_err(|e| BitmexError::Config { message: format!("Prompt error: {e}") })?
            };

            // Summary + confirmation
            println!();
            println!("  Profile : {profile_name}");
            println!("  Testnet : {is_testnet}");
            println!("  API Key : {}", mask_string(&api_key));
            println!(
                "  Storage : {}",
                if keychain::is_available() && !ctx.no_keychain {
                    "OS keychain"
                } else {
                    "config file (keychain unavailable)"
                }
            );
            println!();

            let confirmed = inquire::Confirm::new("Save these credentials?")
                .with_default(true)
                .prompt()
                .map_err(|e| BitmexError::Config { message: format!("Prompt error: {e}") })?;

            if !confirmed {
                return Ok(CommandOutput::message("Cancelled — no credentials saved."));
            }

            save_profile_creds(ctx, &profile_name, &api_key, &api_secret, is_testnet)?;

            // Offer to set as active profile
            let mut cfg = config::load()?;
            let set_active = if cfg.active_profile.is_none() {
                true // auto-activate when nothing is set
            } else if cfg.active_profile.as_deref() != Some(&profile_name) {
                inquire::Confirm::new(&format!(
                    "Set \"{profile_name}\" as the active profile?"
                ))
                .with_default(true)
                .prompt()
                .map_err(|e| BitmexError::Config { message: format!("Prompt error: {e}") })?
            } else {
                false
            };

            if set_active {
                cfg.active_profile = Some(profile_name.clone());
                config::save(&cfg)?;
            }

            let path = config::config_path()?;
            Ok(CommandOutput::from_json(serde_json::json!({
                "status": "saved",
                "profile": profile_name,
                "testnet": is_testnet,
                "api_key": mask_string(&api_key),
                "active": cfg.active_profile.as_deref() == Some(&profile_name),
                "config_file": path.display().to_string(),
            })))
        }

        // ------------------------------------------------------------------
        // Set — non-interactive upsert
        // ------------------------------------------------------------------
        AuthCommand::Set { profile, testnet } => {
            let profile_name = profile
                .or_else(|| ctx.profile.clone())
                .unwrap_or_else(|| "default".to_string());
            let is_testnet = testnet || ctx.testnet;

            let api_key = if let Some(k) = ctx.api_key.clone() {
                k
            } else {
                inquire::Text::new("API Key")
                    .prompt()
                    .map_err(|e| BitmexError::Config { message: format!("Prompt error: {e}") })?
            };

            let api_secret = if let Some(s) = ctx.api_secret.clone() {
                s
            } else {
                inquire::Password::new("API Secret")
                    .without_confirmation()
                    .prompt()
                    .map_err(|e| BitmexError::Config { message: format!("Prompt error: {e}") })?
            };

            save_profile_creds(ctx, &profile_name, &api_key, &api_secret, is_testnet)?;

            let path = config::config_path()?;
            Ok(CommandOutput::from_json(serde_json::json!({
                "status": "saved",
                "profile": profile_name,
                "testnet": is_testnet,
                "api_key": mask_string(&api_key),
                "config_file": path.display().to_string(),
            })))
        }

        // ------------------------------------------------------------------
        // List
        // ------------------------------------------------------------------
        AuthCommand::List => {
            let cfg = config::load()?;
            let active = cfg.active_profile.as_deref().unwrap_or("default");

            if cfg.profiles.is_empty() {
                return Ok(CommandOutput::message(
                    "No profiles configured. Run `bitmex auth add` to create one.",
                ));
            }

            // Read keychain once for all profiles instead of once per profile.
            let keychain_secrets = keychain::load_all_secrets(ctx.no_keychain)
                .unwrap_or(None)
                .unwrap_or_default();

            let rows: Vec<serde_json::Value> = cfg
                .profiles
                .iter()
                .map(|p| {
                    let has_keychain = keychain_secrets.contains_key(&p.name);
                    let storage = if has_keychain {
                        "keychain"
                    } else if p.api_key.is_some() {
                        "config file"
                    } else {
                        "no credentials"
                    };
                    serde_json::json!({
                        "profile": p.name,
                        "active": p.name == active,
                        "testnet": p.testnet,
                        "credentials": storage,
                    })
                })
                .collect();

            Ok(CommandOutput::from_json(serde_json::Value::Array(rows)))
        }

        // ------------------------------------------------------------------
        // Show
        // ------------------------------------------------------------------
        AuthCommand::Show { profile } => {
            let cfg = config::load()?;
            let profile_name = profile
                .or_else(|| ctx.profile.clone())
                .or_else(|| cfg.active_profile.clone())
                .unwrap_or_else(|| "default".to_string());
            let path = config::config_path()?;

            let has_keychain_secret =
                keychain::load_secret(&profile_name, ctx.no_keychain)?.is_some();
            let (key_display, source) =
                match config::get_profile(&cfg, &profile_name).and_then(|p| p.api_key.as_deref())
                {
                    Some(k) => (
                        mask_string(k),
                        if has_keychain_secret { "keychain" } else { "config file" },
                    ),
                    None => ("not set".to_string(), "none"),
                };

            let testnet = config::get_profile(&cfg, &profile_name)
                .map(|p| p.testnet)
                .unwrap_or(false);

            Ok(CommandOutput::from_json(serde_json::json!({
                "profile": profile_name,
                "api_key": key_display,
                "api_secret": "[REDACTED]",
                "testnet": testnet,
                "credentials_source": source,
                "config_file": path.display().to_string(),
            })))
        }

        // ------------------------------------------------------------------
        // Use — set active profile
        // ------------------------------------------------------------------
        AuthCommand::Use { name } => {
            let mut cfg = config::load()?;
            if !cfg.profiles.iter().any(|p| p.name == name) {
                return Err(BitmexError::Validation {
                    message: format!(
                        "Profile \"{name}\" not found. \
                         Run `bitmex auth list` to see available profiles."
                    ),
                });
            }
            cfg.active_profile = Some(name.clone());
            config::save(&cfg)?;
            Ok(CommandOutput::from_json(serde_json::json!({
                "active_profile": name,
            })))
        }

        // ------------------------------------------------------------------
        // Delete — remove profile from keychain + config
        // ------------------------------------------------------------------
        AuthCommand::Delete { profile } => {
            if !ctx.force {
                crate::cli::commands::helpers::confirm_destructive(&format!(
                    "Delete profile \"{profile}\" and its credentials?"
                ))?;
            }
            config::delete_profile_creds(&profile, ctx.no_keychain)?;
            Ok(CommandOutput::from_json(serde_json::json!({
                "status": "deleted",
                "profile": profile,
            })))
        }

        // ------------------------------------------------------------------
        // Reset — backward-compat: clears "default" profile
        // ------------------------------------------------------------------
        AuthCommand::Reset => {
            config::reset_auth()?;
            Ok(CommandOutput::from_json(serde_json::json!({
                "status": "credentials cleared",
            })))
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helper
// ---------------------------------------------------------------------------

/// Store credentials for a profile in the keychain (or TOML fallback),
/// upsert the `ProfileConfig` entry, and save.
fn save_profile_creds(
    ctx: &AppContext,
    profile_name: &str,
    api_key: &str,
    api_secret: &str,
    testnet: bool,
) -> Result<()> {
    let stored_in_keychain =
        keychain::store_profile(profile_name, api_key, api_secret, ctx.no_keychain)?;

    let mut cfg = config::load()?;
    config::upsert_profile(&mut cfg, profile_name, testnet);

    // Always write api_key to TOML — it is not sensitive and avoids a second
    // keychain prompt when loading credentials.
    if let Some(p) = cfg.profiles.iter_mut().find(|p| p.name == profile_name) {
        p.api_key = Some(api_key.to_string());
        if !stored_in_keychain {
            // Keychain unavailable — also store the secret as plaintext fallback.
            p.api_secret = Some(api_secret.to_string());
        }
    }

    if !stored_in_keychain {
        crate::cli::output::warn(
            "Keychain unavailable — credentials stored in config.toml (plaintext fallback).",
        );
    }

    config::save(&cfg)?;
    Ok(())
}
