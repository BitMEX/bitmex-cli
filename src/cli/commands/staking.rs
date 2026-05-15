use clap::Subcommand;
use serde_json::json;

use crate::exchange::client::ExchangeClient;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum StakingCommand {
    /// Show staking positions.
    Status,
    /// List stakeable instruments.
    Instruments,
    /// List pending unstaking requests.
    PendingUnstake,
    /// Request to unstake.
    Unstake {
        symbol: String,
        amount: i64,
    },
    /// Cancel an unstaking request.
    CancelUnstake {
        request_id: String,
    },
}

pub(crate) async fn run(
    cmd: StakingCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
) -> Result<CommandOutput> {
    match cmd {
        StakingCommand::Status => {
            let val = client.get_auth("/user/staking", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        StakingCommand::Instruments => {
            let val = client
                .get_auth("/user/staking/instruments", "", creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        StakingCommand::PendingUnstake => {
            let val = client
                .get_auth("/user/unstakingRequests", "", creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        StakingCommand::Unstake { symbol, amount } => {
            let body = json!({ "symbol": symbol, "amount": amount });
            let val = client
                .post("/user/unstakingRequests", &body, creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        StakingCommand::CancelUnstake { request_id } => {
            let body = json!({ "requestID": request_id });
            let val = client
                .delete("/user/unstakingRequests", "", Some(&body), creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
