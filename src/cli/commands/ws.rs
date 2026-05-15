/// WebSocket streaming for BitMEX real-time market and account data.
///
/// Connects to `wss://ws.bitmex.com/realtime` (or testnet), optionally
/// authenticates, subscribes to the requested topics, and streams NDJSON
/// to stdout until Ctrl-C.
use clap::Args;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::signal;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::exchange::auth;
use crate::config::Credentials;
use crate::errors::{BitmexError, Result};
use crate::cli::output::CommandOutput;

/// Arguments for the `bitmex ws` command.
#[derive(Debug, Args)]
pub(crate) struct WsArgs {
    /// Topics to subscribe to.
    /// Examples: "trade:XBTUSD"  "orderBookL2_25:XBTUSD"  "position"  "execution"
    #[arg(required = true)]
    pub topics: Vec<String>,

    /// Authenticate the WebSocket connection (required for private topics).
    #[arg(long)]
    pub auth: bool,
}

pub(crate) async fn run(
    args: WsArgs,
    ws_url: &str,
    creds: Option<&Credentials>,
) -> Result<CommandOutput> {
    let url = ws_url.to_string();

    let (mut ws, _) = connect_async(&url)
        .await
        .map_err(|e| BitmexError::WebSocket { message: format!("Connect failed: {e}") })?;

    // Authenticate if requested or if any private topic is detected
    let needs_auth = args.auth || args.topics.iter().any(|t| is_private_topic(t));
    if needs_auth {
        let creds = creds.ok_or_else(|| {
            BitmexError::Auth {
                message: "Authentication required. Set BITMEX_API_KEY / BITMEX_API_SECRET or use --api-key / --api-secret."
                    .into(),
            }
        })?;
        let expires = auth::generate_expires()?;
        let sig = auth::sign("GET", "/realtime", expires, "", &creds.api_secret)?;
        let auth_msg = json!({
            "op": "authKeyExpires",
            "args": [creds.api_key, expires, sig]
        });
        ws.send(Message::Text(auth_msg.to_string().into()))
            .await
            .map_err(|e| BitmexError::WebSocket { message: format!("Auth send failed: {e}") })?;
    }

    // Subscribe
    let sub_msg = json!({
        "op": "subscribe",
        "args": args.topics,
    });
    ws.send(Message::Text(sub_msg.to_string().into()))
        .await
        .map_err(|e| BitmexError::WebSocket { message: format!("Subscribe send failed: {e}") })?;

    // Stream messages until Ctrl-C
    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            msg = ws.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        println!("{}", text);
                    }
                    Some(Ok(Message::Binary(bin))) => {
                        if let Ok(text) = std::str::from_utf8(&bin) {
                            println!("{}", text);
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = ws.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        eprintln!("WebSocket closed.");
                        break;
                    }
                    Some(Err(e)) => {
                        return Err(BitmexError::WebSocket { message: format!("Stream error: {e}") });
                    }
                    _ => {}
                }
            }
            _ = &mut ctrl_c => {
                let _ = ws.send(Message::Close(None)).await;
                break;
            }
        }
    }

    Ok(CommandOutput::empty())
}

/// Account-locked topics that require WebSocket authentication.
pub(crate) fn is_private_topic(topic: &str) -> bool {
    // Strip symbol suffix (e.g. "position:XBTUSD" → "position")
    let base = topic.split(':').next().unwrap_or(topic);
    matches!(
        base,
        "affiliate"
            | "csastate"
            | "execution"
            | "isolation"
            | "leverage"
            | "mamAllocation"
            | "margin"
            | "order"
            | "position"
            | "privateNotification"
            | "transact"
            | "voucher"
            | "wallet"
    )
}
