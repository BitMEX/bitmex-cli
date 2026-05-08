use clap::Subcommand;
use serde_json::{json, Value};

use crate::exchange::client::ExchangeClient;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;
use crate::AppContext;

#[derive(Debug, Subcommand)]
pub(crate) enum AddressCommand {
    /// List saved withdrawal addresses.
    List,
    /// Add a withdrawal address to the address book.
    Add {
        currency: String,
        network: String,
        address: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        note: Option<String>,
        #[arg(long)]
        memo: Option<String>,
        /// One-time password (2FA code).
        #[arg(long)]
        otp: Option<String>,
    },
    /// Update an existing address book entry.
    Update {
        id: i64,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// Show address book config (whitelist status, etc).
    Config,
}

pub(crate) async fn run(
    cmd: AddressCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
    _ctx: &AppContext,
) -> Result<CommandOutput> {
    match cmd {
        AddressCommand::List => {
            let val = client.get_auth("/address", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        AddressCommand::Add {
            currency,
            network,
            address,
            name,
            note,
            memo,
            otp,
        } => {
            let mut body = json!({
                "currency": currency,
                "network": network,
                "address": address,
            });
            if let Some(v) = name { body["name"] = Value::String(v); }
            if let Some(v) = note { body["note"] = Value::String(v); }
            if let Some(v) = memo { body["memo"] = Value::String(v); }
            if let Some(v) = otp { body["otpToken"] = Value::String(v); }
            let val = client.post("/address", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        AddressCommand::Update { id, name, note } => {
            let mut body = json!({ "id": id });
            if let Some(v) = name { body["name"] = Value::String(v); }
            if let Some(v) = note { body["note"] = Value::String(v); }
            let val = client.put("/address", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        AddressCommand::Config => {
            let val = client.get_auth("/addressConfig", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
