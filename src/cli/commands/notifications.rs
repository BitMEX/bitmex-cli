use clap::Subcommand;
use serde_json::json;

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::{build_query, confirm_destructive};
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;
use crate::AppContext;

#[derive(Debug, Subcommand)]
pub(crate) enum NotificationsCommand {
    /// Show global site notifications.
    Global,
    /// Show user events.
    Events {
        #[arg(long, default_value = "100")]
        count: u32,
    },
    /// List price alerts.
    Alerts,
    /// Create a price alert.
    AddAlert {
        symbol: String,
        price: f64,
        /// Alert when price goes above (default: below).
        #[arg(long)]
        above: bool,
    },
    /// Delete a price alert by ID.
    DeleteAlert {
        id: i64,
    },
    /// Delete all price alerts.
    DeleteAllAlerts,
}

pub(crate) async fn run(
    cmd: NotificationsCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
    ctx: &AppContext,
) -> Result<CommandOutput> {
    match cmd {
        NotificationsCommand::Global => {
            let val = client
                .get_auth("/globalNotification", "", creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        NotificationsCommand::Events { count } => {
            let q = build_query(&[("count", Some(count.to_string()))]);
            let val = client.get_auth("/userEvent", &q, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        NotificationsCommand::Alerts => {
            let val = client.get_auth("/userPriceAlert", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        NotificationsCommand::AddAlert {
            symbol,
            price,
            above,
        } => {
            let trigger = if above { "above" } else { "below" };
            let body = json!({ "symbol": symbol, "price": price, "trigger": trigger });
            let val = client.post("/userPriceAlert", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        NotificationsCommand::DeleteAlert { id } => {
            if !ctx.force {
                confirm_destructive(&format!("Delete price alert ID {}?", id))?;
            }
            let path = format!("/userPriceAlert/{id}");
            let val = client.delete(&path, "", None, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        NotificationsCommand::DeleteAllAlerts => {
            if !ctx.force {
                confirm_destructive("Delete ALL price alerts?")?;
            }
            let val = client.delete("/userPriceAlert", "", None, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
