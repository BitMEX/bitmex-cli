/// Account/user commands: profile, margin, commission, preferences, etc.
use clap::Subcommand;
use serde_json::{json, Value};

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::build_query;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum AccountCommand {
    /// Show your user profile.
    Me,
    /// Show margin state for an asset.
    Margin {
        #[arg(long)]
        currency: Option<String>,
    },
    /// Show commission rates.
    Commission,
    /// Show affiliate status.
    Affiliate,
    /// Show 30-day trading volume.
    Volume,
    /// Show quote fill ratio.
    QuoteFillRatio {
        #[arg(long)]
        target_account_id: Option<i64>,
    },
    /// Show CSA (crypto-settled account) info.
    Csa,
    /// Show recent execution history (user-scoped).
    ExecutionHistory {
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long, default_value = "100")]
        count: u32,
        #[arg(long)]
        reverse: bool,
    },
    /// Update account preferences.
    Preferences {
        /// JSON string of preferences to set, e.g. '{"locale":"en-US"}'.
        #[arg(long)]
        prefs: String,
    },
    /// Set margining mode.
    MarginingMode {
        /// REGULAR_MARGIN or ISOLATED_MARGIN.
        mode: String,
        #[arg(long)]
        currency: Option<String>,
    },
}

pub(crate) async fn run(
    cmd: AccountCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
) -> Result<CommandOutput> {
    match cmd {
        AccountCommand::Me => {
            let val = client.get_auth("/user", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        AccountCommand::Margin { currency } => {
            let q = build_query(&[("currency", currency)]);
            let val = client.get_auth("/user/margin", &q, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        AccountCommand::Commission => {
            let val = client.get_auth("/user/commission", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        AccountCommand::Affiliate => {
            let val = client.get_auth("/user/affiliateStatus", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        AccountCommand::Volume => {
            let val = client.get_auth("/user/tradingVolume", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        AccountCommand::QuoteFillRatio { target_account_id } => {
            let q = build_query(&[(
                "targetAccountId",
                target_account_id.map(|v| v.to_string()),
            )]);
            let val = client.get_auth("/user/quoteFillRatio", &q, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        AccountCommand::Csa => {
            let val = client.get_auth("/user/csa", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        AccountCommand::ExecutionHistory {
            symbol,
            count,
            reverse,
        } => {
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
            ]);
            let val = client.get_auth("/user/executionHistory", &q, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        AccountCommand::Preferences { prefs } => {
            let prefs_val: Value = serde_json::from_str(&prefs).map_err(|e| {
                crate::errors::BitmexError::Validation { message: format!("Invalid prefs JSON: {e}") }
            })?;
            let body = json!({ "prefs": prefs_val });
            let val = client.post("/user/preferences", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        AccountCommand::MarginingMode { mode, currency } => {
            let mut body = json!({ "selectedMarginingMode": mode });
            if let Some(c) = currency {
                body["currency"] = Value::String(c);
            }
            let val = client.post("/user/marginingMode", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
