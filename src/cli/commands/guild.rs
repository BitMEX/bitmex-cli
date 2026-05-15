use clap::Subcommand;
use serde_json::{json, Value};

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::confirm_destructive;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;
use crate::AppContext;

#[derive(Debug, Subcommand)]
pub(crate) enum GuildCommand {
    /// Show your guild info.
    Info,
    /// Create a new guild.
    Create {
        name: String,
        #[arg(long)]
        emoji: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        twitter: Option<String>,
        #[arg(long)]
        discord: Option<String>,
        #[arg(long)]
        telegram: Option<String>,
        #[arg(long)]
        img_url: Option<String>,
        #[arg(long)]
        is_private: bool,
    },
    /// Update guild settings.
    Update {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        emoji: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        is_private: Option<bool>,
    },
    /// Join a guild by share code.
    Join {
        code: String,
    },
    /// Leave your current guild.
    Leave,
    /// Kick a member from the guild.
    Kick {
        member_user_id: i64,
    },
    /// Archive the guild.
    Archive,
    /// Toggle trade sharing for the guild.
    ShareTrades {
        share: bool,
    },
}

pub(crate) async fn run(
    cmd: GuildCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
    ctx: &AppContext,
) -> Result<CommandOutput> {
    match cmd {
        GuildCommand::Info => {
            let val = client.get_auth("/guild", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        GuildCommand::Create {
            name,
            emoji,
            description,
            twitter,
            discord,
            telegram,
            img_url,
            is_private,
        } => {
            let mut body = json!({ "name": name, "isPrivate": is_private });
            if let Some(v) = emoji { body["emoji"] = Value::String(v); }
            if let Some(v) = description { body["description"] = Value::String(v); }
            if let Some(v) = twitter { body["twitter"] = Value::String(v); }
            if let Some(v) = discord { body["discord"] = Value::String(v); }
            if let Some(v) = telegram { body["telegram"] = Value::String(v); }
            if let Some(v) = img_url { body["imgUrl"] = Value::String(v); }
            let val = client.post("/guild", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        GuildCommand::Update {
            name,
            emoji,
            description,
            is_private,
        } => {
            let mut body = json!({});
            if let Some(v) = name { body["name"] = Value::String(v); }
            if let Some(v) = emoji { body["emoji"] = Value::String(v); }
            if let Some(v) = description { body["description"] = Value::String(v); }
            if let Some(v) = is_private { body["isPrivate"] = Value::Bool(v); }
            let val = client.put("/guild", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        GuildCommand::Join { code } => {
            let body = json!({ "code": code });
            let val = client.post("/guild/join", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        GuildCommand::Leave => {
            if !ctx.force {
                confirm_destructive("Leave your current guild?")?;
            }
            let val = client.post("/guild/leave", &json!({}), creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        GuildCommand::Kick { member_user_id } => {
            if !ctx.force {
                confirm_destructive(&format!("Kick user ID {} from the guild?", member_user_id))?;
            }
            let body = json!({ "memberUserId": member_user_id });
            let val = client.post("/guild/kick", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        GuildCommand::Archive => {
            if !ctx.force {
                confirm_destructive("Archive the guild? This cannot be undone.")?;
            }
            let val = client.post("/guild/archive", &json!({}), creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        GuildCommand::ShareTrades { share } => {
            let body = json!({ "shareTrades": share });
            let val = client.post("/guild/shareTrades", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
