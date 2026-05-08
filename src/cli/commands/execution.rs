use clap::Subcommand;

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::build_query;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum ExecutionCommand {
    /// Raw execution (fill) history.
    List {
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long, default_value = "100")]
        count: u32,
        #[arg(long)]
        reverse: bool,
        #[arg(long)]
        start_time: Option<String>,
        #[arg(long)]
        end_time: Option<String>,
        #[arg(long)]
        filter: Option<String>,
    },
    /// Trade history with realised PnL.
    TradeHistory {
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long, default_value = "100")]
        count: u32,
        #[arg(long)]
        reverse: bool,
        #[arg(long)]
        start_time: Option<String>,
        #[arg(long)]
        end_time: Option<String>,
        #[arg(long)]
        filter: Option<String>,
    },
}

pub(crate) async fn run(
    cmd: ExecutionCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
) -> Result<CommandOutput> {
    match cmd {
        ExecutionCommand::List {
            symbol,
            count,
            reverse,
            start_time,
            end_time,
            filter,
        } => {
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("startTime", start_time),
                ("endTime", end_time),
                ("filter", filter),
            ]);
            let val = client.get_auth("/execution", &q, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        ExecutionCommand::TradeHistory {
            symbol,
            count,
            reverse,
            start_time,
            end_time,
            filter,
        } => {
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("startTime", start_time),
                ("endTime", end_time),
                ("filter", filter),
            ]);
            let val = client
                .get_auth("/execution/tradeHistory", &q, creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
