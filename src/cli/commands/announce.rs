use clap::Subcommand;

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::build_query;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum AnnounceCommand {
    /// List site announcements.
    List {
        /// Comma-separated list of columns to return.
        #[arg(long)]
        columns: Option<String>,
    },
    /// List urgent announcements.
    Urgent,
}

pub(crate) async fn run(
    cmd: AnnounceCommand,
    client: &impl ExchangeClient,
) -> Result<CommandOutput> {
    match cmd {
        AnnounceCommand::List { columns } => {
            let q = build_query(&[("columns", columns)]);
            let val = client.get("/announcement", &q).await?;
            Ok(CommandOutput::from_json(val))
        }
        AnnounceCommand::Urgent => {
            let val = client.get("/announcement/urgent", "").await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
