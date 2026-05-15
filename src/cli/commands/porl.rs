use clap::Subcommand;

use crate::exchange::client::ExchangeClient;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum PorlCommand {
    /// Get a PoRL nonce for proof of reserves verification.
    Nonce,
    /// List PoRL snapshots.
    Snapshots,
}

pub(crate) async fn run(
    cmd: PorlCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
) -> Result<CommandOutput> {
    match cmd {
        PorlCommand::Nonce => {
            let val = client.get_auth("/porl/nonce", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        PorlCommand::Snapshots => {
            let val = client.get_auth("/porl/snapshots", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
