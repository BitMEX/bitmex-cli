use clap::Subcommand;
use serde_json::{json, Value};

use crate::exchange::client::ExchangeClient;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum BotsCommand {
    /// List available bot strategies.
    Strategies,
    /// Show a specific strategy.
    Strategy {
        strategy_id: String,
    },
    /// List your bot instances.
    Instances,
    /// Create a new bot instance.
    Create {
        strategy_id: String,
        /// JSON params string, e.g. '{"symbol":"XBTUSD"}'.
        #[arg(long)]
        params: Option<String>,
    },
    /// Preview a bot instance without creating it.
    Preview {
        strategy_id: String,
        #[arg(long)]
        params: Option<String>,
    },
    /// Show a specific bot instance.
    Instance {
        instance_id: String,
    },
    /// Pause a running bot instance.
    Pause {
        instance_id: String,
    },
    /// Resume a paused bot instance.
    Resume {
        instance_id: String,
    },
    /// Stop a bot instance permanently.
    Stop {
        instance_id: String,
    },
    /// Browse the bot marketplace.
    Marketplace,
    /// Show a marketplace listing.
    MarketplaceListing {
        instance_id: String,
    },
}

pub(crate) async fn run(
    cmd: BotsCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
) -> Result<CommandOutput> {
    match cmd {
        BotsCommand::Strategies => {
            let val = client
                .get_auth("/trading-bots/strategies", "", creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::Strategy { strategy_id } => {
            let path = format!("/trading-bots/strategies/{strategy_id}");
            let val = client.get_auth(&path, "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::Instances => {
            let val = client
                .get_auth("/trading-bots/instances", "", creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::Create {
            strategy_id,
            params,
        } => {
            let mut body = json!({ "strategyId": strategy_id });
            if let Some(p) = params {
                let parsed: Value = serde_json::from_str(&p)
                    .map_err(|e| crate::errors::BitmexError::Validation { message: format!("Invalid params JSON: {e}") })?;
                body["params"] = parsed;
            }
            let val = client
                .post("/trading-bots/instances", &body, creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::Preview {
            strategy_id,
            params,
        } => {
            let mut body = json!({ "strategyId": strategy_id });
            if let Some(p) = params {
                let parsed: Value = serde_json::from_str(&p)
                    .map_err(|e| crate::errors::BitmexError::Validation { message: format!("Invalid params JSON: {e}") })?;
                body["params"] = parsed;
            }
            let val = client
                .post("/trading-bots/instances/preview", &body, creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::Instance { instance_id } => {
            let path = format!("/trading-bots/instances/{instance_id}");
            let val = client.get_auth(&path, "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::Pause { instance_id } => {
            let path = format!("/trading-bots/instances/{instance_id}/pause");
            let val = client.post(&path, &json!({}), creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::Resume { instance_id } => {
            let path = format!("/trading-bots/instances/{instance_id}/resume");
            let val = client.post(&path, &json!({}), creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::Stop { instance_id } => {
            let path = format!("/trading-bots/instances/{instance_id}/stop");
            let val = client.post(&path, &json!({}), creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::Marketplace => {
            let val = client
                .get_auth("/trading-bots/marketplace", "", creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        BotsCommand::MarketplaceListing { instance_id } => {
            let path = format!("/trading-bots/marketplace/{instance_id}");
            let val = client.get_auth(&path, "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
