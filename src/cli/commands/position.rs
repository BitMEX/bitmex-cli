use clap::Subcommand;
use serde_json::json;

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::{build_query, confirm_destructive};
use crate::cli::commands::position_mode::{self, PositionModeArg};
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;
use crate::AppContext;

#[derive(Debug, Subcommand)]
pub(crate) enum PositionCommand {
    /// List open positions.
    List {
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long, default_value = "100")]
        count: u32,
    },
    /// Set leverage for isolated margin (0 = cross margin).
    Leverage {
        symbol: String,
        leverage: f64,
    },
    /// Set cross leverage.
    CrossLeverage {
        symbol: String,
        cross_leverage: f64,
    },
    /// Toggle isolated/cross margin mode.
    Isolate {
        symbol: String,
        /// true = isolated margin, false = cross margin.
        #[arg(long)]
        enabled: bool,
    },
    /// Set risk limit (in XBT satoshis).
    RiskLimit {
        symbol: String,
        risk_limit: i64,
    },
    /// Transfer margin in (positive) or out (negative) of a position.
    TransferMargin {
        symbol: String,
        /// Amount in satoshis.
        amount: i64,
    },
    /// Switch position mode: oneway (netting) or multiway (Hedge Mode).
    ///
    /// Alias for `account position-mode`. Hedge Mode lets you hold independent
    /// Long and Short positions on the same contract.
    Mode {
        /// Target mode: `oneway` or `multiway` (alias `hedge`).
        #[arg(value_enum)]
        mode: PositionModeArg,
        /// Paired/sub-account to switch (defaults to the calling account).
        #[arg(long)]
        target_account_id: Option<i64>,
    },
}

pub(crate) async fn run(
    cmd: PositionCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
    ctx: &AppContext,
) -> Result<CommandOutput> {
    match cmd {
        PositionCommand::List {
            symbol,
            filter,
            count,
        } => {
            let q = build_query(&[
                ("symbol", symbol),
                ("filter", filter),
                ("count", Some(count.to_string())),
            ]);
            let val = client.get_auth("/position", &q, creds).await?;
            Ok(CommandOutput::builder()
                .data(val)
                .columns(&[
                    "symbol", "strategy", "currentQty", "avgEntryPrice", "markPrice",
                    "liquidationPrice", "unrealisedPnl", "realisedPnl",
                    "leverage", "crossMargin", "marginCallPrice",
                ])
                .build())
        }

        PositionCommand::Leverage { symbol, leverage } => {
            let body = json!({ "symbol": symbol, "leverage": leverage });
            let val = client.post("/position/leverage", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        PositionCommand::CrossLeverage {
            symbol,
            cross_leverage,
        } => {
            let body = json!({ "symbol": symbol, "crossLeverage": cross_leverage });
            let val = client
                .post("/position/crossLeverage", &body, creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        PositionCommand::Isolate { symbol, enabled } => {
            if !ctx.force {
                confirm_destructive(&format!(
                    "Set {} margin to {} for {}?",
                    if enabled { "isolated" } else { "cross" },
                    if enabled { "isolated" } else { "cross" },
                    symbol
                ))?;
            }
            let body = json!({ "symbol": symbol, "enabled": enabled });
            let val = client.post("/position/isolate", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        PositionCommand::RiskLimit { symbol, risk_limit } => {
            let body = json!({ "symbol": symbol, "riskLimit": risk_limit });
            let val = client.post("/position/riskLimit", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        PositionCommand::TransferMargin { symbol, amount } => {
            if !ctx.force {
                confirm_destructive(&format!(
                    "Transfer {} satoshis {} position {}?",
                    amount.abs(),
                    if amount >= 0 { "into" } else { "out of" },
                    symbol
                ))?;
            }
            let body = json!({ "symbol": symbol, "amount": amount });
            let val = client
                .post("/position/transferMargin", &body, creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        PositionCommand::Mode {
            mode,
            target_account_id,
        } => position_mode::run(client, creds, ctx, mode, target_account_id).await,
    }
}
