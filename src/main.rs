use std::env;
use std::process;

use clap::Parser;

use bitmex_cli::errors::BitmexError;
use bitmex_cli::output::OutputFormat;
use bitmex_cli::client;
use bitmex_cli::{AppContext, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let format = cli.output.unwrap_or(OutputFormat::Table);

    // Determine secret and track whether it came from the raw --api-secret flag
    let (api_secret, secret_from_flag) = if cli.api_secret_stdin {
        match bitmex_cli::config::read_secret_from_stdin() {
            Ok(s) => (Some(s.expose().to_string()), false),
            Err(e) => {
                bitmex_cli::output::render_error(format, &e);
                process::exit(1);
            }
        }
    } else if let Some(ref path) = cli.api_secret_file {
        match bitmex_cli::config::read_secret_from_file(path) {
            Ok(s) => (Some(s.expose().to_string()), false),
            Err(e) => {
                bitmex_cli::output::render_error(format, &e);
                process::exit(1);
            }
        }
    } else if cli.api_secret.is_some() {
        (cli.api_secret.clone(), true)
    } else {
        (None, false)
    };

    // Emit secret-exposure warning
    if secret_from_flag {
        bitmex_cli::output::warn(
            "Passing --api-secret on the command line exposes it in process listings. \
             Prefer --api-secret-stdin, --api-secret-file, or environment variables.",
        );
    }

    // Resolve URL overrides: CLI flag > env var > default
    let resolve = |cli_flag: Option<&str>, env_name: &str, source_label: &str| -> Option<String> {
        let env_val = env::var(env_name).ok();
        match client::resolve_url_override(cli_flag, env_val.as_deref()) {
            Ok(url) => url,
            Err(e) => {
                let source = if cli_flag.is_some() {
                    source_label
                } else {
                    env_name
                };
                let inner = match &e {
                    BitmexError::Validation { message } => message.clone(),
                    other => other.to_string(),
                };
                bitmex_cli::output::render_error(
                    format,
                    &BitmexError::Validation { message: format!("{source}: {inner}") },
                );
                process::exit(1);
            }
        }
    };

    let api_url = resolve(cli.api_url.as_deref(), "BITMEX_API_URL", "--api-url");
    let ws_url = resolve(None, "BITMEX_WS_URL", "BITMEX_WS_URL");

    // Merge CLI --testnet with the active profile's testnet flag
    let testnet = bitmex_cli::config::effective_testnet(cli.profile.as_deref(), cli.testnet);

    let ctx = AppContext {
        format,
        verbose: cli.verbose,
        testnet,
        api_url,
        ws_url,
        api_key: cli.api_key.clone(),
        api_secret,
        force: cli.yes,
        secret_from_flag,
        mcp_mode: false,
        profile: cli.profile.clone(),
        no_keychain: cli.no_keychain,
        timing: cli.timing,
    };

    match cli.command {
        Some(command) => {
            if let Err(e) = bitmex_cli::dispatch(&ctx, command).await {
                bitmex_cli::output::render_error(ctx.format, &e);
                process::exit(1);
            }
        }
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
        }
    }
}
