/// Public market data commands — no auth required.
use clap::Subcommand;

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::build_query;
use crate::errors::Result;
use crate::cli::output::CommandOutput;

#[derive(Debug, Subcommand)]
pub(crate) enum MarketCommand {
    /// Get instrument/contract information.
    Instrument {
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long, default_value = "100")]
        count: u32,
        #[arg(long)]
        reverse: bool,
        #[arg(long)]
        start_time: Option<String>,
        #[arg(long)]
        end_time: Option<String>,
        /// Show only active instruments.
        #[arg(long)]
        active: bool,
        /// Show only index instruments.
        #[arg(long)]
        indices: bool,
    },
    /// Get best bid/ask quotes.
    Quote {
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
        /// Bucketed OHLCV quotes.
        #[arg(long)]
        bucketed: bool,
        /// Bin size for bucketed: 1m, 5m, 1h, 1d.
        #[arg(long, default_value = "1h")]
        bin_size: String,
    },
    /// Get recent trades.
    Trades {
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
        /// Bucketed (OHLCV) trade bins.
        #[arg(long)]
        bucketed: bool,
        #[arg(long, default_value = "1h")]
        bin_size: String,
    },
    /// Get L2 order book.
    Orderbook {
        symbol: String,
        #[arg(long, default_value = "25")]
        depth: u32,
    },
    /// Get perpetual funding rate history.
    Funding {
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
    },
    /// Get liquidation history.
    Liquidation {
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
    },
    /// Get settlement history.
    Settlement {
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
    },
    /// Get insurance fund history.
    Insurance {
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long, default_value = "100")]
        count: u32,
        #[arg(long)]
        start_time: Option<String>,
        #[arg(long)]
        end_time: Option<String>,
    },
    /// Get site-wide trading statistics.
    Stats,
    /// Get stats history.
    StatsHistory {
        #[arg(long)]
        currency: Option<String>,
    },
    /// Get leaderboard.
    Leaderboard {
        /// Method: notional (default) or roe.
        #[arg(long, default_value = "notional")]
        method: String,
    },
    /// Get composite index data.
    CompositeIndex {
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long, default_value = "100")]
        count: u32,
        #[arg(long)]
        start_time: Option<String>,
        #[arg(long)]
        end_time: Option<String>,
    },
    /// Get USD volume for instruments.
    UsdVolume {
        #[arg(long)]
        symbol: Option<String>,
    },
}

pub(crate) async fn run(
    cmd: MarketCommand,
    client: &impl ExchangeClient,
) -> Result<CommandOutput> {
    match cmd {
        MarketCommand::Instrument {
            symbol,
            filter,
            count,
            reverse,
            start_time,
            end_time,
            active,
            indices,
        } => {
            let path = if active {
                "/instrument/active"
            } else if indices {
                "/instrument/indices"
            } else {
                "/instrument"
            };
            let q = build_query(&[
                ("symbol", symbol),
                ("filter", filter),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("startTime", start_time),
                ("endTime", end_time),
            ]);
            let val = client.get(path, &q).await?;
            Ok(CommandOutput::builder()
                .data(val)
                .columns(&[
                    "symbol", "underlying", "state", "typ", "lastPrice",
                    "bidPrice", "askPrice", "volume24h", "openInterest",
                    "fundingRate", "markPrice",
                ])
                .build())
        }

        MarketCommand::Quote {
            symbol,
            count,
            reverse,
            start_time,
            end_time,
            bucketed,
            bin_size,
        } => {
            let path = if bucketed { "/quote/bucketed" } else { "/quote" };
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("startTime", start_time),
                ("endTime", end_time),
                ("binSize", if bucketed { Some(bin_size) } else { None }),
            ]);
            let val = client.get(path, &q).await?;
            Ok(CommandOutput::builder()
                .data(val)
                .columns(&["symbol", "bidPrice", "bidSize", "askPrice", "askSize", "timestamp"])
                .build())
        }

        MarketCommand::Trades {
            symbol,
            count,
            reverse,
            start_time,
            end_time,
            bucketed,
            bin_size,
        } => {
            let path = if bucketed { "/trade/bucketed" } else { "/trade" };
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("startTime", start_time),
                ("endTime", end_time),
                ("binSize", if bucketed { Some(bin_size) } else { None }),
            ]);
            let val = client.get(path, &q).await?;
            Ok(CommandOutput::builder()
                .data(val)
                .columns(&["timestamp", "symbol", "side", "size", "price", "tickDirection"])
                .build())
        }

        MarketCommand::Orderbook { symbol, depth } => {
            let q = build_query(&[
                ("symbol", Some(symbol)),
                ("depth", Some(depth.to_string())),
            ]);
            let val = client.get("/orderBook/L2", &q).await?;
            Ok(CommandOutput::from_json(val))
        }

        MarketCommand::Funding {
            symbol,
            count,
            reverse,
            start_time,
            end_time,
        } => {
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("startTime", start_time),
                ("endTime", end_time),
            ]);
            let val = client.get("/funding", &q).await?;
            Ok(CommandOutput::builder()
                .data(val)
                .columns(&["timestamp", "symbol", "fundingRate", "fundingRateDaily"])
                .build())
        }

        MarketCommand::Liquidation {
            symbol,
            count,
            reverse,
            start_time,
            end_time,
        } => {
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("startTime", start_time),
                ("endTime", end_time),
            ]);
            let val = client.get("/liquidation", &q).await?;
            Ok(CommandOutput::from_json(val))
        }

        MarketCommand::Settlement {
            symbol,
            count,
            reverse,
            start_time,
            end_time,
        } => {
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("startTime", start_time),
                ("endTime", end_time),
            ]);
            let val = client.get("/settlement", &q).await?;
            Ok(CommandOutput::from_json(val))
        }

        MarketCommand::Insurance {
            symbol,
            count,
            start_time,
            end_time,
        } => {
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("startTime", start_time),
                ("endTime", end_time),
            ]);
            let val = client.get("/insurance", &q).await?;
            Ok(CommandOutput::from_json(val))
        }

        MarketCommand::Stats => {
            let val = client.get("/stats", "").await?;
            Ok(CommandOutput::from_json(val))
        }

        MarketCommand::StatsHistory { currency } => {
            let q = build_query(&[("currency", currency)]);
            let val = client.get("/stats/history", &q).await?;
            Ok(CommandOutput::from_json(val))
        }

        MarketCommand::Leaderboard { method } => {
            let q = build_query(&[("method", Some(method))]);
            let val = client.get("/leaderboard", &q).await?;
            Ok(CommandOutput::from_json(val))
        }

        MarketCommand::CompositeIndex {
            symbol,
            count,
            start_time,
            end_time,
        } => {
            let q = build_query(&[
                ("symbol", symbol),
                ("count", Some(count.to_string())),
                ("startTime", start_time),
                ("endTime", end_time),
            ]);
            let val = client.get("/instrument/compositeIndex", &q).await?;
            Ok(CommandOutput::from_json(val))
        }

        MarketCommand::UsdVolume { symbol } => {
            let q = build_query(&[("symbol", symbol)]);
            let val = client.get("/instrument/usdVolume", &q).await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
