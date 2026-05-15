use clap::Subcommand;
use serde_json::json;

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::build_query;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum ChatCommand {
    /// Read chat messages.
    Read {
        #[arg(long, default_value = "1")]
        channel_id: f64,
        #[arg(long, default_value = "100")]
        count: u32,
        #[arg(long)]
        reverse: bool,
        #[arg(long)]
        start: Option<f64>,
    },
    /// Post a message to chat.
    Post {
        message: String,
        #[arg(long, default_value = "1")]
        channel_id: f64,
    },
    /// Show the pinned message for a channel.
    Pinned {
        #[arg(long, default_value = "1")]
        channel_id: f64,
    },
    /// List available channels.
    Channels,
    /// List connected users.
    Connected,
}

pub(crate) async fn run(
    cmd: ChatCommand,
    client: &impl ExchangeClient,
    creds: Option<&Credentials>,
) -> Result<CommandOutput> {
    match cmd {
        ChatCommand::Read {
            channel_id,
            count,
            reverse,
            start,
        } => {
            let q = build_query(&[
                ("channelID", Some(channel_id.to_string())),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("start", start.map(|s| s.to_string())),
            ]);
            let val = client.get("/chat", &q).await?;
            Ok(CommandOutput::from_json(val))
        }

        ChatCommand::Post {
            message,
            channel_id,
        } => {
            let body = json!({ "message": message, "channelID": channel_id });
            let creds = creds.ok_or_else(|| {
                crate::errors::BitmexError::Auth { message: "Authentication required to post chat messages".into() }
            })?;
            let val = client.post("/chat", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        ChatCommand::Pinned { channel_id } => {
            let q = build_query(&[("channelID", Some(channel_id.to_string()))]);
            let val = client.get("/chat/pinned", &q).await?;
            Ok(CommandOutput::from_json(val))
        }

        ChatCommand::Channels => {
            let val = client.get("/chat/channels", "").await?;
            Ok(CommandOutput::from_json(val))
        }

        ChatCommand::Connected => {
            let val = client.get("/chat/connected", "").await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
