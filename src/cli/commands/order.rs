/// Order management commands: buy, sell, amend, cancel, cancel-all, cancel-after, close.
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

/// Side of a position-closing order. `sell` closes a long, `buy` closes a short.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum CloseSide {
    Buy,
    Sell,
}

impl CloseSide {
    /// The exact string the BitMEX API expects for the `side` field.
    fn as_api(self) -> &'static str {
        match self {
            CloseSide::Buy => "Buy",
            CloseSide::Sell => "Sell",
        }
    }
}

/// Trigger price type for stop / take-profit orders.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum TriggerType {
    Last,
    Mark,
    Index,
}

impl TriggerType {
    /// The exact `execInst` trigger token the BitMEX API expects.
    fn as_api(self) -> &'static str {
        match self {
            TriggerType::Last => "LastPrice",
            TriggerType::Mark => "MarkPrice",
            TriggerType::Index => "IndexPrice",
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
    /// Place a 100% position-closing order: Stop-Loss, Take-Profit, an OCO
    /// bracket (both, linked), or an immediate market close.
    ///
    /// `orderQty` is always omitted, so the order tracks the *entire* position
    /// dynamically (BitMEX renders it as "SL (100%)" / "TP (100%)") — no resync.
    /// Pass both `--stop-px` and `--tp-px` to place an OCO bracket where filling
    /// one leg cancels the other.
    Close {
        symbol: String,
        /// `sell` closes a long, `buy` closes a short.
        #[arg(long, value_enum)]
        side: CloseSide,
        /// Stop-Loss trigger price. Sets ordType=Stop (or StopLimit with --stop-limit-px).
        #[arg(long)]
        stop_px: Option<f64>,
        /// Take-Profit trigger price. Sets ordType=MarketIfTouched (or LimitIfTouched with --tp-limit-px).
        #[arg(long)]
        tp_px: Option<f64>,
        /// Trigger price type. Required when --stop-px or --tp-px is set.
        #[arg(long, value_enum)]
        trigger: Option<TriggerType>,
        /// Limit price for the Stop-Loss leg (upgrades Stop -> StopLimit).
        #[arg(long)]
        stop_limit_px: Option<f64>,
        /// Limit price for the Take-Profit leg (upgrades MarketIfTouched -> LimitIfTouched).
        #[arg(long)]
        tp_limit_px: Option<f64>,
        /// OCO link id (clOrdLinkID). Auto-generated for brackets when omitted.
        #[arg(long)]
        link_id: Option<String>,
        /// Position strategy for Hedge Mode: `long` or `short`.
        #[arg(long, value_enum)]
        strategy: Option<OrderStrategy>,
        /// Client order id (single-order placements only).
        #[arg(long)]
        cl_ord_id: Option<String>,
        #[arg(long)]
        text: Option<String>,
        /// Print the request body without submitting.
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
    qty: Option<f64>,
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
        "ordType": order_type,
    });
    // Omitting orderQty (None) is what makes a Close order track 100% of the
    // position dynamically — see `OrderCommand::Close`.
    if let Some(q) = qty { body["orderQty"] = json!(q); }
    if let Some(p) = price { body["price"] = json!(p); }
    if let Some(p) = stop_px { body["stopPx"] = json!(p); }
    if let Some(t) = tif { body["timeInForce"] = Value::String(t); }
    if let Some(e) = exec_inst { body["execInst"] = Value::String(e); }
    if let Some(s) = strategy { body["strategy"] = Value::String(s); }
    if let Some(c) = cl_ord_id { body["clOrdID"] = Value::String(c); }
    body["text"] = Value::String(text.unwrap_or_else(|| "Submitted via CLI.".to_string()));
    body
}

/// Build the `execInst` for a close order: the trigger price type (if any)
/// followed by `Close`. `Close` implies ReduceOnly and, with `orderQty`
/// omitted, closes the entire position.
fn close_exec_inst(trigger: Option<TriggerType>) -> String {
    match trigger {
        Some(t) => format!("{},Close", t.as_api()),
        None => "Close".to_string(),
    }
}

/// A resolved close request: the endpoint to POST to and the JSON body.
struct ClosePlan {
    path: &'static str,
    body: Value,
}

/// Build the request for a 100% position-closing order.
///
/// - both `stop_px` and `tp_px` -> OCO bracket of two legs sharing a
///   `clOrdLinkID` with `contingencyType=OneCancelsTheOther`, posted to
///   `/order/bulk`;
/// - only `stop_px` -> single Stop / StopLimit leg;
/// - only `tp_px` -> single MarketIfTouched / LimitIfTouched leg;
/// - neither -> single Market close.
///
/// Every leg omits `orderQty` so it closes the full position. Pure (the OCO
/// link id is resolved by the caller) so it is unit-testable without I/O.
#[allow(clippy::too_many_arguments)]
fn build_close_plan(
    symbol: &str,
    side: &str,
    stop_px: Option<f64>,
    tp_px: Option<f64>,
    exec_inst: &str,
    stop_limit_px: Option<f64>,
    tp_limit_px: Option<f64>,
    link_id: Option<String>,
    strategy: Option<String>,
    cl_ord_id: Option<String>,
    text: Option<String>,
) -> ClosePlan {
    let sl_leg = |cl_ord_id: Option<String>| {
        let ord_type = if stop_limit_px.is_some() { "StopLimit" } else { "Stop" };
        build_order_body(
            symbol, side, None, ord_type, stop_limit_px, stop_px, None,
            Some(exec_inst.to_string()), strategy.clone(), cl_ord_id, text.clone(),
        )
    };
    let tp_leg = |cl_ord_id: Option<String>| {
        let ord_type = if tp_limit_px.is_some() { "LimitIfTouched" } else { "MarketIfTouched" };
        build_order_body(
            symbol, side, None, ord_type, tp_limit_px, tp_px, None,
            Some(exec_inst.to_string()), strategy.clone(), cl_ord_id, text.clone(),
        )
    };

    match (stop_px, tp_px) {
        (Some(_), Some(_)) => {
            // OCO bracket: link the two legs so filling one cancels the other.
            let link = link_id.unwrap_or_else(gen_link_id);
            let mut sl = sl_leg(None);
            let mut tp = tp_leg(None);
            for leg in [&mut sl, &mut tp] {
                leg["clOrdLinkID"] = Value::String(link.clone());
                leg["contingencyType"] = Value::String("OneCancelsTheOther".to_string());
            }
            ClosePlan { path: "/order/bulk", body: json!({ "orders": [sl, tp] }) }
        }
        (Some(_), None) => ClosePlan { path: "/order", body: sl_leg(cl_ord_id) },
        (None, Some(_)) => ClosePlan { path: "/order", body: tp_leg(cl_ord_id) },
        (None, None) => {
            // Immediate market close of the full position.
            let body = build_order_body(
                symbol, side, None, "Market", None, None, None,
                Some(exec_inst.to_string()), strategy, cl_ord_id, text,
            );
            ClosePlan { path: "/order", body }
        }
    }
}

/// Generate a unique OCO link id. Uses wall-clock nanos — sufficient to keep
/// each bracket's two legs grouped and distinct from other brackets.
fn gen_link_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("cli-oco-{nanos}")
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
            let body = build_order_body(&symbol, "Buy", Some(qty), &order_type, price, stop_px, tif, exec_inst, strategy.map(|s| s.as_api().to_string()), cl_ord_id, text);
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
            let body = build_order_body(&symbol, "Sell", Some(qty), &order_type, price, stop_px, tif, exec_inst, strategy.map(|s| s.as_api().to_string()), cl_ord_id, text);
            if validate {
                return Ok(CommandOutput::from_json(body));
            }
            if !ctx.force {
                confirm_destructive(&format!("Place SELL order: {} {} @ {:?}?", qty, symbol, price))?;
            }
            let val = client.post("/order", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        OrderCommand::Close {
            symbol, side, stop_px, tp_px, trigger, stop_limit_px, tp_limit_px,
            link_id, strategy, cl_ord_id, text, validate,
        } => {
            // A trigger price type is mandatory whenever a stop/TP trigger is set.
            if (stop_px.is_some() || tp_px.is_some()) && trigger.is_none() {
                return Err(crate::errors::BitmexError::Validation {
                    message: "stop/take-profit close orders require --trigger last|mark|index".into(),
                });
            }
            let exec_inst = close_exec_inst(trigger);
            // Pre-resolve the OCO link id so build_close_plan stays pure/testable.
            let link_id = match (stop_px, tp_px, link_id) {
                (Some(_), Some(_), None) => Some(gen_link_id()),
                (_, _, given) => given,
            };
            let plan = build_close_plan(
                &symbol, side.as_api(), stop_px, tp_px, &exec_inst,
                stop_limit_px, tp_limit_px, link_id,
                strategy.map(|s| s.as_api().to_string()), cl_ord_id, text,
            );
            if validate {
                return Ok(CommandOutput::from_json(plan.body));
            }
            if !ctx.force {
                let desc = match (stop_px, tp_px) {
                    (Some(s), Some(t)) => format!("OCO bracket on {symbol} (SL @ {s} / TP @ {t})"),
                    (Some(s), None) => format!("Stop-Loss on {symbol} @ {s}"),
                    (None, Some(t)) => format!("Take-Profit on {symbol} @ {t}"),
                    (None, None) => format!("MARKET close on {symbol}"),
                };
                confirm_destructive(&format!("Place 100% {desc}?"))?;
            }
            let val = client.post(plan.path, &plan.body, creds).await?;
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
            "XBTUSD", "Buy", Some(100.0), "Limit", Some(50000.0), None, None, None, strategy, None, None,
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

    #[test]
    fn build_order_body_includes_qty_when_set() {
        let body = sample_body(None);
        assert_eq!(body["orderQty"], 100.0);
    }

    #[test]
    fn build_order_body_omits_qty_when_none() {
        let body = build_order_body(
            "XBTUSD", "Sell", None, "Stop", None, Some(50000.0), None,
            Some("MarkPrice,Close".to_string()), None, None, None,
        );
        assert!(body.get("orderQty").is_none());
    }

    #[test]
    fn close_side_and_trigger_map_to_exact_api_strings() {
        assert_eq!(CloseSide::Buy.as_api(), "Buy");
        assert_eq!(CloseSide::Sell.as_api(), "Sell");
        assert_eq!(TriggerType::Last.as_api(), "LastPrice");
        assert_eq!(TriggerType::Mark.as_api(), "MarkPrice");
        assert_eq!(TriggerType::Index.as_api(), "IndexPrice");
    }

    #[test]
    fn close_exec_inst_prefixes_trigger_then_close() {
        assert_eq!(close_exec_inst(Some(TriggerType::Mark)), "MarkPrice,Close");
        assert_eq!(close_exec_inst(Some(TriggerType::Last)), "LastPrice,Close");
        assert_eq!(close_exec_inst(None), "Close");
    }

    /// Helper to build a close plan with the common args defaulted.
    fn close_plan(
        stop_px: Option<f64>,
        tp_px: Option<f64>,
        stop_limit_px: Option<f64>,
        tp_limit_px: Option<f64>,
    ) -> ClosePlan {
        build_close_plan(
            "XBTUSD", "Sell", stop_px, tp_px, "MarkPrice,Close",
            stop_limit_px, tp_limit_px, Some("link-1".to_string()), None, None, None,
        )
    }

    #[test]
    fn close_plan_single_stop_loss() {
        let plan = close_plan(Some(50000.0), None, None, None);
        assert_eq!(plan.path, "/order");
        assert_eq!(plan.body["ordType"], "Stop");
        assert_eq!(plan.body["stopPx"], 50000.0);
        assert_eq!(plan.body["execInst"], "MarkPrice,Close");
        assert_eq!(plan.body["side"], "Sell");
        assert!(plan.body.get("orderQty").is_none());
        assert!(plan.body.get("price").is_none());
    }

    #[test]
    fn close_plan_stop_limit_when_limit_px_set() {
        let plan = close_plan(Some(50000.0), None, Some(49900.0), None);
        assert_eq!(plan.body["ordType"], "StopLimit");
        assert_eq!(plan.body["stopPx"], 50000.0);
        assert_eq!(plan.body["price"], 49900.0);
    }

    #[test]
    fn close_plan_single_take_profit() {
        let plan = close_plan(None, Some(60000.0), None, None);
        assert_eq!(plan.path, "/order");
        assert_eq!(plan.body["ordType"], "MarketIfTouched");
        assert_eq!(plan.body["stopPx"], 60000.0);
        assert!(plan.body.get("orderQty").is_none());
    }

    #[test]
    fn close_plan_limit_if_touched_when_limit_px_set() {
        let plan = close_plan(None, Some(60000.0), None, Some(60100.0));
        assert_eq!(plan.body["ordType"], "LimitIfTouched");
        assert_eq!(plan.body["price"], 60100.0);
    }

    #[test]
    fn close_plan_market_close_when_no_trigger() {
        let plan = build_close_plan(
            "XBTUSD", "Sell", None, None, "Close", None, None, None, None, None, None,
        );
        assert_eq!(plan.path, "/order");
        assert_eq!(plan.body["ordType"], "Market");
        assert_eq!(plan.body["execInst"], "Close");
        assert!(plan.body.get("orderQty").is_none());
        assert!(plan.body.get("stopPx").is_none());
    }

    #[test]
    fn close_plan_oco_bracket_links_both_legs() {
        let plan = close_plan(Some(50000.0), Some(60000.0), None, None);
        assert_eq!(plan.path, "/order/bulk");
        let orders = plan.body["orders"].as_array().expect("orders array");
        assert_eq!(orders.len(), 2);

        let (sl, tp) = (&orders[0], &orders[1]);
        assert_eq!(sl["ordType"], "Stop");
        assert_eq!(sl["stopPx"], 50000.0);
        assert_eq!(tp["ordType"], "MarketIfTouched");
        assert_eq!(tp["stopPx"], 60000.0);

        for leg in orders {
            assert_eq!(leg["clOrdLinkID"], "link-1");
            assert_eq!(leg["contingencyType"], "OneCancelsTheOther");
            assert!(leg.get("orderQty").is_none());
        }
    }
}
