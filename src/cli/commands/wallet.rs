/// Wallet commands: balance, history, deposit, withdrawal, transfer.
use clap::Subcommand;
use serde_json::{json, Value};

use crate::exchange::client::ExchangeClient;
use crate::cli::commands::helpers::{build_query, confirm_destructive};
use crate::config::Credentials;
use crate::errors::Result;
use crate::cli::output::CommandOutput;
use crate::AppContext;

#[derive(Debug, Subcommand)]
pub(crate) enum WalletCommand {
    /// Show wallet balance.
    Balance {
        #[arg(long, default_value = "all")]
        currency: Option<String>,
    },
    /// Show wallet transaction history.
    History {
        #[arg(long)]
        currency: Option<String>,
        #[arg(long, default_value = "100")]
        count: u32,
        #[arg(long)]
        start: Option<i64>,
    },
    /// Show wallet summary.
    Summary {
        #[arg(long)]
        currency: Option<String>,
    },
    /// Get deposit address.
    Deposit {
        #[arg(long, default_value = "XBt")]
        currency: String,
        #[arg(long)]
        network: Option<String>,
    },
    /// Get detailed deposit address information.
    DepositInfo {
        #[arg(long, default_value = "XBt")]
        currency: String,
        #[arg(long)]
        network: Option<String>,
    },
    /// Request a withdrawal.
    Withdraw {
        #[arg(long)]
        currency: String,
        #[arg(long)]
        network: String,
        #[arg(long)]
        address: String,
        #[arg(long)]
        amount: i64,
        #[arg(long)]
        fee: Option<i64>,
        #[arg(long)]
        text: Option<String>,
        /// One-time password (2FA code).
        #[arg(long)]
        otp: Option<String>,
    },
    /// Cancel a pending withdrawal.
    CancelWithdraw {
        token: String,
    },
    /// Confirm a withdrawal.
    ConfirmWithdraw {
        token: String,
    },
    /// Transfer between accounts.
    Transfer {
        #[arg(long)]
        currency: String,
        #[arg(long)]
        amount: i64,
        #[arg(long)]
        to_user_id: Option<i64>,
    },
    /// List accounts available for transfer.
    TransferAccounts,
    /// List supported wallet assets.
    Assets,
    /// List supported wallet networks.
    Networks,
    /// List haircut rates.
    Haircuts,
}

pub(crate) async fn run(
    cmd: WalletCommand,
    client: &impl ExchangeClient,
    creds: &Credentials,
    ctx: &AppContext,
) -> Result<CommandOutput> {
    match cmd {
        WalletCommand::Balance { currency } => {
            let q = build_query(&[("currency", currency)]);
            let val = client.get_auth("/user/wallet", &q, creds).await?;
            Ok(CommandOutput::builder()
                .data(val)
                .columns(&["currency", "amount", "prevAmount", "addr", "deposited", "withdrawn"])
                .build())
        }

        WalletCommand::History {
            currency,
            count,
            start,
        } => {
            let q = build_query(&[
                ("currency", currency),
                ("count", Some(count.to_string())),
                ("start", start.map(|s| s.to_string())),
            ]);
            let val = client.get_auth("/user/walletHistory", &q, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::Summary { currency } => {
            let q = build_query(&[("currency", currency)]);
            let val = client.get_auth("/user/walletSummary", &q, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::Deposit { currency, network } => {
            let q = build_query(&[
                ("currency", Some(currency)),
                ("network", network),
            ]);
            let val = client.get_auth("/user/depositAddress", &q, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::DepositInfo { currency, network } => {
            let q = build_query(&[
                ("currency", Some(currency)),
                ("network", network),
            ]);
            let val = client
                .get_auth("/user/depositAddressInformation", &q, creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::Withdraw {
            currency,
            network,
            address,
            amount,
            fee,
            text,
            otp,
        } => {
            if !ctx.force {
                confirm_destructive(&format!(
                    "Withdraw {} {} to {}?",
                    amount, currency, address
                ))?;
            }
            let mut body = json!({
                "currency": currency,
                "network": network,
                "address": address,
                "amount": amount,
            });
            if let Some(v) = fee { body["fee"] = json!(v); }
            if let Some(v) = text { body["text"] = Value::String(v); }
            if let Some(v) = otp { body["otpToken"] = Value::String(v); }
            let val = client.post("/user/requestWithdrawal", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::CancelWithdraw { token } => {
            if !ctx.force {
                confirm_destructive("Cancel this withdrawal request?")?;
            }
            let body = json!({ "token": token });
            let val = client.post("/user/cancelWithdrawal", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::ConfirmWithdraw { token } => {
            let body = json!({ "token": token });
            let val = client.post("/user/confirmWithdrawal", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::Transfer {
            currency,
            amount,
            to_user_id,
        } => {
            if !ctx.force {
                confirm_destructive(&format!(
                    "Transfer {} {} to account {:?}?",
                    amount, currency, to_user_id
                ))?;
            }
            let mut body = json!({ "currency": currency, "amount": amount });
            if let Some(v) = to_user_id { body["toUserId"] = json!(v); }
            let val = client.post("/user/walletTransfer", &body, creds).await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::TransferAccounts => {
            let val = client
                .get_auth("/user/getWalletTransferAccounts", "", creds)
                .await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::Assets => {
            let val = client.get("/wallet/assets", "").await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::Networks => {
            let val = client.get("/wallet/networks", "").await?;
            Ok(CommandOutput::from_json(val))
        }

        WalletCommand::Haircuts => {
            let val = client.get("/wallet/haircuts", "").await?;
            Ok(CommandOutput::from_json(val))
        }
    }
}
