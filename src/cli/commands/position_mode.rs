/// Hedge Mode (MultiWay position mode) switching, shared by the canonical
/// `account position-mode` command and the `position mode` alias.
///
/// Hedge Mode is an account-level setting: when enabled (MultiWay), derivative
/// positions split into independent Long and Short buckets on the same
/// instrument instead of netting into a single One-Way position. The exchange
/// rejects the switch if the account has open orders or isolated-margin
/// positions, and only uncapped derivatives support per-leg strategies.
use clap::ValueEnum;
use serde_json::{json, Value};

use crate::cli::commands::helpers::confirm_destructive;
use crate::cli::output::CommandOutput;
use crate::config::Credentials;
use crate::errors::Result;
use crate::exchange::client::ExchangeClient;
use crate::AppContext;

/// Account position mode: One-Way (netting) vs MultiWay (Hedge Mode).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum PositionModeArg {
    /// One-Way / netting: a single net position per contract.
    #[value(name = "oneway", alias = "one-way")]
    OneWay,
    /// MultiWay / Hedge Mode: independent Long and Short positions per contract.
    #[value(name = "multiway", aliases = ["multi-way", "hedge"])]
    MultiWay,
}

impl PositionModeArg {
    fn is_multiway(self) -> bool {
        matches!(self, PositionModeArg::MultiWay)
    }
}

/// Build the `POST /user/positionMode` request body.
///
/// MultiWay sends `{"positionMode":"MultiWay"}`; One-Way omits the field
/// entirely (the API treats an absent `positionMode` as One-Way). An optional
/// `targetAccountId` targets a paired/sub-account.
pub(crate) fn build_position_mode_body(
    mode: PositionModeArg,
    target_account_id: Option<i64>,
) -> Value {
    let mut body = json!({});
    if mode.is_multiway() {
        body["positionMode"] = Value::String("MultiWay".to_string());
    }
    if let Some(id) = target_account_id {
        body["targetAccountId"] = json!(id);
    }
    body
}

/// Switch the account between One-Way and Hedge (MultiWay) position mode.
pub(crate) async fn run(
    client: &impl ExchangeClient,
    creds: &Credentials,
    ctx: &AppContext,
    mode: PositionModeArg,
    target_account_id: Option<i64>,
) -> Result<CommandOutput> {
    if !ctx.force {
        let label = if mode.is_multiway() {
            "MultiWay (Hedge Mode)"
        } else {
            "One-Way"
        };
        confirm_destructive(&format!(
            "Switch account position mode to {label}? This requires no open orders \
             and no isolated-margin positions."
        ))?;
    }
    let body = build_position_mode_body(mode, target_account_id);
    let val = client.post("/user/positionMode", &body, creds).await?;
    Ok(CommandOutput::from_json(val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiway_body_sets_position_mode() {
        let body = build_position_mode_body(PositionModeArg::MultiWay, None);
        assert_eq!(body["positionMode"], "MultiWay");
        assert!(body.get("targetAccountId").is_none());
    }

    #[test]
    fn oneway_body_omits_position_mode() {
        let body = build_position_mode_body(PositionModeArg::OneWay, None);
        // One-Way is signalled by the absence of the field, per the API contract.
        assert!(body.get("positionMode").is_none());
        assert_eq!(body, json!({}));
    }

    #[test]
    fn target_account_id_threaded() {
        let body = build_position_mode_body(PositionModeArg::MultiWay, Some(12345));
        assert_eq!(body["positionMode"], "MultiWay");
        assert_eq!(body["targetAccountId"], 12345);
    }

    #[test]
    fn oneway_body_keeps_target_account_id() {
        let body = build_position_mode_body(PositionModeArg::OneWay, Some(777));
        assert!(body.get("positionMode").is_none());
        assert_eq!(body["targetAccountId"], 777);
    }
}
