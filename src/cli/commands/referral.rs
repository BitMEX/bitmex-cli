use clap::Subcommand;
use serde_json::{json, Value};

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::confirm_destructive;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;
use crate::AppContext;

#[derive(Debug, Subcommand)]
pub(crate) enum ReferralCommand {
    /// List your referral codes.
    List,
    /// Create a new referral code.
    Create {
        #[arg(long)]
        referral_code: Option<String>,
    },
    /// Check if a referral code is valid.
    Check {
        code: String,
    },
    /// Show details of a referral code by ID.
    Get {
        id: String,
    },
    /// Look up a referral code by its code string.
    Lookup {
        code: String,
    },
    /// Update a referral code.
    Update {
        id: String,
        #[arg(long)]
        referral_code: Option<String>,
    },
    /// Delete a referral code.
    Delete {
        id: String,
    },
}

pub(crate) async fn run(
    cmd: ReferralCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
    ctx: &AppContext,
) -> Result<CommandOutput> {
    match cmd {
        ReferralCommand::List => {
            let val = client.get_auth("/referralCode", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        ReferralCommand::Create { referral_code } => {
            let mut body = json!({});
            if let Some(v) = referral_code { body["referralCode"] = Value::String(v); }
            let val = client.post("/referralCode", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        ReferralCommand::Check { code } => {
            let path = format!("/referralCode/check/{code}");
            let val = client.get_auth(&path, "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        ReferralCommand::Get { id } => {
            let path = format!("/referralCode/{id}");
            let val = client.get_auth(&path, "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        ReferralCommand::Lookup { code } => {
            let path = format!("/referralCode/code/{code}");
            let val = client.get(&path, "").await?;
            Ok(CommandOutput::from_json(val))
        }

        ReferralCommand::Update { id, referral_code } => {
            let mut body = json!({});
            if let Some(v) = referral_code { body["referralCode"] = Value::String(v); }
            let path = format!("/referralCode/{id}");
            let val = client.put(&path, &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        ReferralCommand::Delete { id } => {
            if !ctx.force {
                confirm_destructive(&format!("Delete referral code {}?", id))?;
            }
            let path = format!("/referralCode/{id}");
            let val = client.delete(&path, "", None, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
