/// Order management commands: buy, sell, amend, cancel, cancel-all, cancel-after, close-position.
use clap::{Subcommand, ValueEnum};
use serde_json::{json, Value};

/// Position strategy for an order leg under Hedge (MultiWay) mode.
///
/// In One-Way mode leave this unset (or `oneway`); in Hedge Mode pass `long`
/// or `short` to target the corresponding position bucket.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum OrderStrategy {
    #[value(name = "oneway", alias = "one-way")]
    OneWay,
    Long,
    Short,
}

impl OrderStrategy {
    /// The exact string the BitMEX API expects for the `strategy` field.
    fn as_api(self) -> &'static str {
        match self {
            OrderStrategy::OneWay => "OneWay",
            OrderStrategy::Long => "Long",
            OrderStrategy::Short => "Short",
        }
    }
}

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::confirm_destructive;
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;
use crate::AppContext;

#[derive(Debug, Subcommand)]
pub(crate) enum OrderCommand {
    /// Place a buy order.
    Buy {
        symbol: String,
        qty: f64,
        #[arg(long, default_value = "Limit")]
        order_type: String,
        #[arg(long)]
        price: Option<f64>,
        #[arg(long)]
        stop_px: Option<f64>,
        #[arg(long)]
        tif: Option<String>,
        /// Execution instructions (e.g. ParticipateDoNotInitiate, ReduceOnly).
        #[arg(long)]
        exec_inst: Option<String>,
        /// Position strategy for Hedge Mode: `long` or `short`. Requires hedge
        /// mode enabled; omit (or `oneway`) for One-Way accounts.
        #[arg(long, value_enum)]
        strategy: Option<OrderStrategy>,
        #[arg(long)]
        cl_ord_id: Option<String>,
        #[arg(long)]
        text: Option<String>,
        /// Print the request body without submitting.
        #[arg(long)]
        validate: bool,
    },
    /// Place a sell order.
    Sell {
        symbol: String,
        qty: f64,
        #[arg(long, default_value = "Limit")]
        order_type: String,
        #[arg(long)]
        price: Option<f64>,
        #[arg(long)]
        stop_px: Option<f64>,
        #[arg(long)]
        tif: Option<String>,
        #[arg(long)]
        exec_inst: Option<String>,
        /// Position strategy for Hedge Mode: `long` or `short`. Requires hedge
        /// mode enabled; omit (or `oneway`) for One-Way accounts.
        #[arg(long, value_enum)]
        strategy: Option<OrderStrategy>,
        #[arg(long)]
        cl_ord_id: Option<String>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        validate: bool,
    },
    /// Amend an existing order.
    Amend {
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long)]
        cl_ord_id: Option<String>,
        #[arg(long)]
        qty: Option<f64>,
        #[arg(long)]
        price: Option<f64>,
        #[arg(long)]
        stop_px: Option<f64>,
        #[arg(long)]
        text: Option<String>,
    },
    /// Cancel one or more orders.
    Cancel {
        /// Comma-separated order IDs to cancel.
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long)]
        cl_ord_id: Option<String>,
        #[arg(long)]
        text: Option<String>,
    },
    /// Cancel all open orders.
    CancelAll {
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        text: Option<String>,
    },
    /// Dead Man's Switch: cancel all orders after timeout milliseconds.
    CancelAfter {
        timeout: u64,
    },
    /// List open (or all) orders.
    List {
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
    },
}

#[allow(clippy::too_many_arguments)]
fn build_order_body(
    symbol: &str,
    side: &str,
    qty: f64,
    order_type: &str,
    price: Option<f64>,
    stop_px: Option<f64>,
    tif: Option<String>,
    exec_inst: Option<String>,
    strategy: Option<String>,
    cl_ord_id: Option<String>,
    text: Option<String>,
) -> Value {
    let mut body = json!({
        "symbol": symbol,
        "side": side,
        "orderQty": qty,
        "ordType": order_type,
    });
    if let Some(p) = price { body["price"] = json!(p); }
    if let Some(p) = stop_px { body["stopPx"] = json!(p); }
    if let Some(t) = tif { body["timeInForce"] = Value::String(t); }
    if let Some(e) = exec_inst { body["execInst"] = Value::String(e); }
    if let Some(s) = strategy { body["strategy"] = Value::String(s); }
    if let Some(c) = cl_ord_id { body["clOrdID"] = Value::String(c); }
    body["text"] = Value::String(text.unwrap_or_else(|| "Submitted via CLI.".to_string()));
    body
}

pub(crate) async fn run(
    cmd: OrderCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
    ctx: &AppContext,
) -> Result<CommandOutput> {
    match cmd {
        OrderCommand::Buy {
            symbol, qty, order_type, price, stop_px, tif, exec_inst, strategy, cl_ord_id, text, validate,
        } => {
            let body = build_order_body(&symbol, "Buy", qty, &order_type, price, stop_px, tif, exec_inst, strategy.map(|s| s.as_api().to_string()), cl_ord_id, text);
            if validate {
                return Ok(CommandOutput::from_json(body));
            }
            if !ctx.force {
                confirm_destructive(&format!("Place BUY order: {} {} @ {:?}?", qty, symbol, price))?;
            }
            let val = client.post("/order", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        OrderCommand::Sell {
            symbol, qty, order_type, price, stop_px, tif, exec_inst, strategy, cl_ord_id, text, validate,
        } => {
            let body = build_order_body(&symbol, "Sell", qty, &order_type, price, stop_px, tif, exec_inst, strategy.map(|s| s.as_api().to_string()), cl_ord_id, text);
            if validate {
                return Ok(CommandOutput::from_json(body));
            }
            if !ctx.force {
                confirm_destructive(&format!("Place SELL order: {} {} @ {:?}?", qty, symbol, price))?;
            }
            let val = client.post("/order", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        OrderCommand::Amend {
            order_id, cl_ord_id, qty, price, stop_px, text,
        } => {
            let mut body = json!({});
            if let Some(v) = order_id { body["orderID"] = Value::String(v); }
            if let Some(v) = cl_ord_id { body["clOrdID"] = Value::String(v); }
            if let Some(v) = qty { body["orderQty"] = json!(v); }
            if let Some(v) = price { body["price"] = json!(v); }
            if let Some(v) = stop_px { body["stopPx"] = json!(v); }
            body["text"] = Value::String(text.unwrap_or_else(|| "Amended via CLI.".to_string()));
            let val = client.put("/order", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        OrderCommand::Cancel { order_id, cl_ord_id, text } => {
            if !ctx.force {
                confirm_destructive("Cancel the specified order(s)?")?;
            }
            let mut body = json!({});
            if let Some(v) = order_id { body["orderID"] = Value::String(v); }
            if let Some(v) = cl_ord_id { body["clOrdID"] = Value::String(v); }
            body["text"] = Value::String(text.unwrap_or_else(|| "Canceled via CLI.".to_string()));
            let val = client.delete("/order", "", Some(&body), creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        OrderCommand::CancelAll { symbol, text } => {
            if !ctx.force {
                confirm_destructive("Cancel ALL open orders?")?;
            }
            let mut body = json!({});
            if let Some(v) = symbol { body["symbol"] = Value::String(v); }
            body["text"] = Value::String(text.unwrap_or_else(|| "Canceled via CLI.".to_string()));
            let val = client.delete("/order/all", "", Some(&body), creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        OrderCommand::CancelAfter { timeout } => {
            let body = json!({ "timeout": timeout });
            let val = client.post("/order/cancelAllAfter", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        OrderCommand::List {
            symbol, filter, count, reverse, start_time, end_time,
        } => {
            use crate::cli::commands::helpers::build_query;
            let q = build_query(&[
                ("symbol", symbol),
                ("filter", filter),
                ("count", Some(count.to_string())),
                ("reverse", if reverse { Some("true".into()) } else { None }),
                ("startTime", start_time),
                ("endTime", end_time),
            ]);
            let val = client.get_auth("/order", &q, creds).await?;
            Ok(CommandOutput::builder()
                .data(val)
                .columns(&[
                    "orderID", "symbol", "side", "strategy", "orderQty", "price",
                    "ordType", "ordStatus", "avgPx", "leavesQty", "timestamp",
                ])
                .build())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_body(strategy: Option<String>) -> Value {
        build_order_body(
            "XBTUSD", "Buy", 100.0, "Limit", Some(50000.0), None, None, None, strategy, None, None,
        )
    }

    #[test]
    fn order_strategy_maps_to_exact_api_strings() {
        assert_eq!(OrderStrategy::OneWay.as_api(), "OneWay");
        assert_eq!(OrderStrategy::Long.as_api(), "Long");
        assert_eq!(OrderStrategy::Short.as_api(), "Short");
    }

    #[test]
    fn build_order_body_includes_strategy_when_set() {
        let body = sample_body(Some(OrderStrategy::Long.as_api().to_string()));
        assert_eq!(body["strategy"], "Long");
    }

    #[test]
    fn build_order_body_omits_strategy_when_unset() {
        let body = sample_body(None);
        assert!(body.get("strategy").is_none());
    }
}
