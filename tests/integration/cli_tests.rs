use assert_cmd::Command;
use predicates::prelude::*;

#[allow(deprecated)]
fn bitmex() -> Command {
    Command::cargo_bin("bitmex").unwrap()
}

#[test]
fn help_flag_shows_usage() {
    bitmex()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("BitMEX CLI"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn version_flag_shows_version() {
    bitmex()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("bitmex"));
}

#[test]
fn no_args_prints_help() {
    bitmex()
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn unknown_command_fails() {
    bitmex().arg("nonexistent-command").assert().failure();
}

#[test]
fn order_help_shows_subcommands() {
    bitmex()
        .args(["order", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("buy"))
        .stdout(predicate::str::contains("sell"))
        .stdout(predicate::str::contains("cancel"));
}

#[test]
fn auth_help_shows_subcommands() {
    bitmex()
        .args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("use"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("reset"));
}

#[test]
fn output_flag_accepts_table_and_json() {
    bitmex()
        .args(["--output", "table", "--help"])
        .assert()
        .success();

    bitmex()
        .args(["--output", "json", "--help"])
        .assert()
        .success();
}

#[test]
fn output_flag_rejects_invalid() {
    bitmex()
        .args(["--output", "xml", "--help"])
        .assert()
        .failure();
}

#[test]
fn mcp_help_shows_allow_dangerous_flag() {
    bitmex()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--allow-dangerous"))
        .stdout(predicate::str::contains(
            "Skip per-call confirmation for dangerous tools",
        ));
}

#[test]
fn market_help_shows_subcommands() {
    bitmex()
        .args(["market", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("instrument"))
        .stdout(predicate::str::contains("orderbook"))
        .stdout(predicate::str::contains("trades"));
}

#[test]
fn ws_help_shows_usage() {
    bitmex()
        .args(["ws", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("WebSocket streaming"))
        .stdout(predicate::str::contains("TOPICS"));
}

#[test]
fn wallet_help_shows_subcommands() {
    bitmex()
        .args(["wallet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("balance"));
}

#[test]
fn position_help_shows_subcommands() {
    bitmex()
        .args(["position", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("mode"));
}

#[test]
fn account_help_shows_position_mode() {
    bitmex()
        .args(["account", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("position-mode"));
}

#[test]
fn position_mode_help_lists_modes() {
    bitmex()
        .args(["account", "position-mode", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("oneway"))
        .stdout(predicate::str::contains("multiway"))
        .stdout(predicate::str::contains("hedge"));
}

#[test]
fn order_buy_help_shows_strategy_flag() {
    bitmex()
        .args(["order", "buy", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--strategy"));
}
