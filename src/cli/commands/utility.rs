/// Utility commands: `setup` wizard.
///
/// The setup wizard is a thin wrapper that delegates to `bitmex auth add`
/// and additionally captures the default symbol setting.
use crate::cli::commands::auth::{run as auth_run, AuthCommand};
use crate::config::{self, SettingsConfig};
use crate::errors::Result;
use crate::cli::output::CommandOutput;
use crate::AppContext;

/// Run the interactive setup wizard.
pub(crate) async fn setup(ctx: &AppContext) -> Result<CommandOutput> {
    if ctx.verbose {
        crate::cli::output::verbose("Starting setup wizard");
    }

    println!("BitMEX CLI Setup");
    println!("================");
    println!();

    let storage_note = if crate::config::keychain::is_available() && !ctx.no_keychain {
        "OS keychain (secure)".to_string()
    } else {
        format!(
            "{} (mode 0600)",
            config::config_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "config file".into())
        )
    };
    println!("Credentials will be stored in: {storage_note}");
    println!();

    // Delegate credential collection to `auth add`
    let add_result = auth_run(
        AuthCommand::Add { profile: None, testnet: false },
        ctx,
    )
    .await?;

    // Prompt for default symbol
    let default_symbol: String = inquire::Text::new("Default symbol")
        .with_default("XBTUSD")
        .with_help_message("Used when --symbol is not specified")
        .prompt()
        .map_err(|e| crate::errors::BitmexError::Config { message: format!("Input error: {e}") })?;

    let mut cfg = config::load()?;
    cfg.settings = SettingsConfig {
        default_symbol: Some(default_symbol),
        output: Some("table".to_string()),
    };
    config::save(&cfg)?;

    let _ = add_result; // result already printed by auth add
    let path = config::config_path()?;
    Ok(CommandOutput::message(&format!(
        "Setup complete! Config saved to {}",
        path.display()
    )))
}
