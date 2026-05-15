use clap::Subcommand;

use crate::exchange::client::ExchangeClient;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum ApiKeyCommand {
    /// List all API keys for the account.
    List,
    /// Show info about the currently authenticated API key.
    Me,
}

pub(crate) async fn run(
    cmd: ApiKeyCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
) -> Result<CommandOutput> {
    match cmd {
        ApiKeyCommand::List => {
            let val = client.get_auth("/apiKey", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
        ApiKeyCommand::Me => {
            let val = client.get_auth("/apiKey/self", "", creds).await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
