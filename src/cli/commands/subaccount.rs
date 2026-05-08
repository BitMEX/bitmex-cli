use clap::Subcommand;
use serde_json::{json, Value};

use crate::exchange::client::ExchangeClient;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum SubaccountCommand {
    /// Add a subaccount.
    Add {
        account_name: String,
    },
    /// Create an independent subaccount.
    CreateIndependent {
        account_name: String,
    },
    /// Update a subaccount.
    Update {
        account_id: i64,
        #[arg(long)]
        account_name: Option<String>,
    },
    /// List accounts eligible for wallet transfer.
    TransferAccounts,
}

pub(crate) async fn run(
    cmd: SubaccountCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
) -> Result<CommandOutput> {
    match cmd {
        SubaccountCommand::Add { account_name } => {
            let body = json!({ "accountName": account_name });
            let val = client.post("/user/addSubaccount", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        SubaccountCommand::CreateIndependent { account_name } => {
            let body = json!({ "accountName": account_name });
            let val = client
                .post("/user/createIndependentSubaccount", &body, creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }
        SubaccountCommand::Update {
            account_id,
            account_name,
        } => {
            let mut body = json!({ "accountId": account_id });
            if let Some(v) = account_name {
                body["accountName"] = Value::String(v);
            }
            let val = client.post("/user/updateSubaccount", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        SubaccountCommand::TransferAccounts => {
            let val = client
                .get_auth("/user/getWalletTransferAccounts", "", creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
