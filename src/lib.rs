/// BitMEX CLI library crate.
///
/// Provides `AppContext`, the shared dispatcher, and all modules.
pub(crate) mod cli;
pub mod config;
pub mod errors;
pub(crate) mod exchange;
pub(crate) mod mcp;

// Re-export the public surface used by the binary crate.
pub mod output {
    pub use crate::cli::output::{
        render_error, warn, OutputFormat,
    };
}
pub mod client {
    pub use crate::exchange::client::resolve_url_override;
}

use clap::{Parser, Subcommand};

use exchange::client::BitmexClient;
use cli::commands::account::{self as account, AccountCommand};
use cli::commands::address::{self as address, AddressCommand};
use cli::commands::announce::{self as announce, AnnounceCommand};
use cli::commands::apikey::{self as apikey, ApiKeyCommand};
use cli::commands::auth::{self as auth_cmds, AuthCommand};
use cli::commands::bots::{self as bots, BotsCommand};
use cli::commands::chat::{self as chat, ChatCommand};
use cli::commands::execution::{self as execution, ExecutionCommand};
use cli::commands::guild::{self as guild, GuildCommand};
use cli::commands::market::{self as market, MarketCommand};
use cli::commands::notifications::{self as notifications, NotificationsCommand};
use cli::commands::order::{self as order, OrderCommand};
use cli::commands::porl::{self as porl, PorlCommand};
use cli::commands::position::{self as position, PositionCommand};
use cli::commands::referral::{self as referral, ReferralCommand};
use cli::commands::staking::{self as staking, StakingCommand};
use cli::commands::subaccount::{self as subaccount, SubaccountCommand};
use cli::commands::utility;
use cli::commands::wallet::{self as wallet, WalletCommand};
use cli::commands::ws::{self as ws, WsArgs};
use errors::Result;
use cli::output::{render, CommandOutput, OutputFormat};

/// Runtime context assembled from global CLI flags and config.
#[derive(Clone)]
pub struct AppContext {
    pub format: OutputFormat,
    pub verbose: bool,
    pub testnet: bool,
    pub api_url: Option<String>,
    pub ws_url: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub force: bool,
    /// True when the secret came from the raw `--api-secret` CLI flag (argv-visible).
    pub secret_from_flag: bool,
    /// True when running in MCP server mode. Disables all interactive prompts.
    pub mcp_mode: bool,
    /// Named credential profile to use (`--profile` flag).
    pub profile: Option<String>,
    /// Skip OS keychain; force plaintext TOML fallback (`--no-keychain` / `BITMEX_NO_KEYCHAIN`).
    pub no_keychain: bool,
    /// Print curl-style per-request timing breakdown to stderr.
    pub timing: bool,
}

/// BitMEX CLI — trade, query, and manage your BitMEX account from the terminal.
#[derive(Parser)]
#[command(name = "bitmex", version, about, long_about = None)]
pub struct Cli {
    /// Output format: table (default) or json.
    #[arg(short, long, value_enum, global = true)]
    pub output: Option<OutputFormat>,

    /// Show request/response details on stderr.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Print curl-style timing breakdown (namelookup, connect, TTFB, transfer, total) to stderr.
    #[arg(long, global = true)]
    pub timing: bool,

    /// Use BitMEX testnet (https://testnet.bitmex.com) instead of mainnet.
    #[arg(long, global = true)]
    pub testnet: bool,

    /// Override API base URL.
    #[arg(long, global = true)]
    pub api_url: Option<String>,

    /// API key override (takes precedence over env/config).
    #[arg(long, global = true)]
    pub api_key: Option<String>,

    /// API secret override (prefer --api-secret-stdin for security).
    #[arg(long, global = true)]
    pub api_secret: Option<String>,

    /// Read API secret from stdin (mutually exclusive with --api-secret and --api-secret-file).
    #[arg(long, global = true, conflicts_with_all = ["api_secret", "api_secret_file"])]
    pub api_secret_stdin: bool,

    /// Path to file containing API secret (mutually exclusive with --api-secret and --api-secret-stdin).
    #[arg(long, global = true, conflicts_with_all = ["api_secret", "api_secret_stdin"])]
    pub api_secret_file: Option<std::path::PathBuf>,

    /// Skip confirmation prompts for destructive operations.
    #[arg(long, alias = "force", global = true)]
    pub yes: bool,

    /// Named credential profile to use (overrides active_profile in config).
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Skip OS keychain; use plaintext config-file fallback.
    /// Also enabled via BITMEX_NO_KEYCHAIN=1.
    #[arg(long, global = true, env = "BITMEX_NO_KEYCHAIN")]
    pub no_keychain: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[expect(private_interfaces)]
#[derive(Subcommand)]
pub enum Command {
    /// Manage API credentials.
    Auth {
        #[command(subcommand)]
        cmd: AuthCommand,
    },
    /// Market data: instruments, quotes, trades, orderbook, funding, etc.
    Market {
        #[command(subcommand)]
        cmd: MarketCommand,
    },
    /// Place and manage orders.
    Order {
        #[command(subcommand)]
        cmd: OrderCommand,
    },
    /// Manage open positions (leverage, margin, etc).
    Position {
        #[command(subcommand)]
        cmd: PositionCommand,
    },
    /// Execution and trade history.
    Execution {
        #[command(subcommand)]
        cmd: ExecutionCommand,
    },
    /// Wallet balances, deposits, withdrawals, and transfers.
    Wallet {
        #[command(subcommand)]
        cmd: WalletCommand,
    },
    /// Staking positions, instruments, and unstaking.
    Staking {
        #[command(subcommand)]
        cmd: StakingCommand,
    },
    /// Account profile, margin, and preferences.
    Account {
        #[command(subcommand)]
        cmd: AccountCommand,
    },
    /// Subaccount management.
    Subaccount {
        #[command(subcommand)]
        cmd: SubaccountCommand,
    },
    /// API key management.
    ApiKey {
        #[command(subcommand)]
        cmd: ApiKeyCommand,
    },
    /// Chat (trollbox) messages and channels.
    Chat {
        #[command(subcommand)]
        cmd: ChatCommand,
    },
    /// Guild management.
    Guild {
        #[command(subcommand)]
        cmd: GuildCommand,
    },
    /// Trading bots (strategies, instances, marketplace).
    Bots {
        #[command(subcommand)]
        cmd: BotsCommand,
    },
    /// Notifications, user events, and price alerts.
    Notifications {
        #[command(subcommand)]
        cmd: NotificationsCommand,
    },
    /// Withdrawal address book management.
    Address {
        #[command(subcommand)]
        cmd: AddressCommand,
    },
    /// Referral code management.
    Referral {
        #[command(subcommand)]
        cmd: ReferralCommand,
    },
    /// Proof of Reserves and Liabilities.
    Porl {
        #[command(subcommand)]
        cmd: PorlCommand,
    },
    /// Site announcements.
    Announce {
        #[command(subcommand)]
        cmd: AnnounceCommand,
    },
    /// WebSocket streaming.
    Ws(WsArgs),
    /// Guided first-time setup wizard.
    Setup,
    /// Interactive REPL shell.
    Shell,
    /// Start a built-in MCP (Model Context Protocol) server over stdio or HTTP.
    Mcp {
        /// Comma-separated service groups, or "all". Default: market,account.
        #[arg(short, long, default_value = "market,account")]
        services: String,
        /// Skip per-call confirmation for dangerous tools.
        #[arg(long)]
        allow_dangerous: bool,
        /// Run as an HTTP server on this port (e.g. 3000). Omit for stdio mode.
        #[arg(long)]
        port: Option<u16>,
        /// Require this bearer token on all HTTP requests (recommended with --port).
        #[arg(long)]
        token: Option<String>,
    },
}

fn build_client(ctx: &AppContext) -> Result<BitmexClient> {
    if let Some(ref url) = ctx.api_url {
        BitmexClient::new_with_url(url.clone(), ctx.testnet, ctx.verbose, ctx.timing)
    } else {
        BitmexClient::new(ctx.testnet, ctx.verbose, ctx.timing)
    }
}

/// Route a parsed command to its handler and return structured output.
pub async fn dispatch(ctx: &AppContext, command: Command) -> Result<()> {
    let out = execute_command(ctx, command).await?;
    render(ctx.format, &out);
    Ok(())
}

/// Route a parsed command to its handler and return the structured output.
pub(crate) async fn execute_command(ctx: &AppContext, command: Command) -> Result<CommandOutput> {
    match command {
        Command::Auth { cmd } => auth_cmds::run(cmd, ctx).await,

        Command::Market { cmd } => {
            let client = build_client(ctx)?;
            market::run(cmd, &client).await
        }

        Command::Order { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            order::run(cmd, &client, &creds, ctx).await
        }

        Command::Position { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            position::run(cmd, &client, &creds, ctx).await
        }

        Command::Execution { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            execution::run(cmd, &client, &creds).await
        }

        Command::Wallet { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            wallet::run(cmd, &client, &creds, ctx).await
        }

        Command::Staking { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            staking::run(cmd, &client, &creds).await
        }

        Command::Account { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            account::run(cmd, &client, &creds).await
        }

        Command::Subaccount { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            subaccount::run(cmd, &client, &creds).await
        }

        Command::ApiKey { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            apikey::run(cmd, &client, &creds).await
        }

        Command::Chat { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            ).ok();
            chat::run(cmd, &client, creds.as_ref()).await
        }

        Command::Guild { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            guild::run(cmd, &client, &creds, ctx).await
        }

        Command::Bots { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            bots::run(cmd, &client, &creds).await
        }

        Command::Notifications { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            notifications::run(cmd, &client, &creds, ctx).await
        }

        Command::Address { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            address::run(cmd, &client, &creds, ctx).await
        }

        Command::Referral { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            referral::run(cmd, &client, &creds, ctx).await
        }

        Command::Porl { cmd } => {
            let client = build_client(ctx)?;
            let creds = config::resolve_credentials(
                ctx.profile.as_deref(),
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.no_keychain,
            )?;
            porl::run(cmd, &client, &creds).await
        }

        Command::Announce { cmd } => {
            let client = build_client(ctx)?;
            announce::run(cmd, &client).await
        }

        Command::Ws(args) => {
            let ws_url = if ctx.testnet {
                exchange::client::WS_TESTNET_URL.to_string()
            } else {
                exchange::client::WS_MAINNET_URL.to_string()
            };
            let needs_auth = args.auth || args.topics.iter().any(|t| ws::is_private_topic(t));
            let creds = if needs_auth {
                config::resolve_credentials(
                    ctx.profile.as_deref(),
                    ctx.api_key.as_deref(),
                    ctx.api_secret.as_deref(),
                    ctx.no_keychain,
                ).ok()
            } else {
                None
            };
            ws::run(args, &ws_url, creds.as_ref()).await
        }

        Command::Setup => utility::setup(ctx).await,

        Command::Shell => {
            crate::cli::shell::run(ctx).await?;
            Ok(CommandOutput::empty())
        }

        Command::Mcp { services, allow_dangerous, port, token } => {
            crate::mcp::server::run_server(&services, allow_dangerous, port, token, ctx).await?;
            Ok(CommandOutput::empty())
        }
    }
}
