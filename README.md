# bitmex-cli

![license](https://img.shields.io/badge/license-MIT-green)

The official BitMEX command-line interface for interacting with the [BitMEX](https://www.bitmex.com) exchange. Execute trades, query market data, and manage your account from the terminal. Supports integration with AI coding agents for automation.

Single binary, no runtime dependencies. 134 commands with structured JSON output and real-time streaming covering every REST and WebSocket endpoint. Built-in [MCP](https://modelcontextprotocol.io) server for direct agent integration.

Compatible with major AI coding agents including Claude, Codex, Cursor, Copilot, Gemini, and others. Automate workflows such as market monitoring, order execution, and portfolio reporting — all from a single prompt.

---

> [!WARNING]
> **⚠️ WARNING — PLEASE READ BEFORE USE**
>
> This is experimental software that interacts with the live BitMEX exchange and can execute real financial transactions that may result in loss of funds. Use of this tool is subject to the [BitMEX Terms of Service](https://www.bitmex.com/terms).
>
> BitMEX services are not available in all jurisdictions — you are responsible for ensuring your use complies with applicable laws.
>
> Use `--testnet` to test safely with no real money at risk. Read [DISCLAIMER.md](DISCLAIMER.md) before using with real funds or AI agents.


## Get Started with AI

Getting started is easy! Paste the following into your AI agent to set up bitmex-cli for your BitMEX trading account:

```
Set up bitmex-cli for my BitMEX trading account.

## Install
Run: tmpfile="$(mktemp)" && curl -sSfL https://raw.githubusercontent.com/BitMEX/bitmex-cli/master/install.sh -o "$tmpfile" && sh "$tmpfile"

## Account Setup
If I don't have a BitMEX account yet, walk me through:
1. Sign up at https://www.bitmex.com/register
2. Complete identity verification
3. Deposit funds

## API Key
Guide me to https://www.bitmex.com/app/apiKeys to create an API key with
"Order" and "Account" permissions (add "Withdraw" only if I need withdrawals).
Ask me to choose:
- Share my key and secret with you, and run: bitmex auth set --api-key <KEY> --api-secret <SECRET>
- Or keep keys private: ask me to open a terminal and run `bitmex auth add` myself (do NOT run this for me — it requires an interactive terminal)

## Verify
Run: bitmex account me -o json

## MCP (optional)
Ask me if I want to set up the built-in MCP server (`bitmex mcp`), which
lets you (the AI agent) call bitmex commands directly as tools without subprocess wrappers
— smoother integration for agents that support MCP.

## What's Next
Use the bitmex-* skills available to you and ask me what I want to do. Offer these:
- Morning market brief (prices, funding rates, positions)
- Start a DCA bot (dollar-cost average into a position)
- Launch a grid trading bot (buy dips, sell rallies automatically)
- Scan for funding rate carry opportunities
- Monitor my positions and P&L
And suggest starting with --testnet to practice safely before using real funds.
```

## Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Testnet](#testnet)
- [Commands](#commands)
- [API Keys & Configuration](#api-keys--configuration)
- [MCP Server](#mcp-server)
- [WebSocket Streaming](#websocket-streaming)
- [AI Agents](#ai-agents)
- [Contributing](#contributing)
- [License & Disclaimer](#license--disclaimer)

## Installation

### Homebrew (macOS/Linux)

```bash
brew install BitMEX/tap/bitmex-cli
```

### Install script

```bash
curl -sSfL https://raw.githubusercontent.com/BitMEX/bitmex-cli/master/install.sh | sh
```

Downloads a pre-built binary for your platform (macOS/Linux, x86_64/arm64), verifies the SHA256 checksum, and installs to `~/.local/bin`. No Rust or build tools needed. No `sudo` required. Requires `curl`, `tar`, and `sha256sum` (or `shasum`). Override the install location with `BITMEX_INSTALL_DIR`.

### GitHub Releases

Download binaries directly from [Releases](https://github.com/BitMEX/bitmex-cli/releases).

### From source (requires Rust)

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and install
cargo install --path .
```
### Verify

```bash
bitmex --version
```

## Quick Start

```bash
# Public market data — no auth needed
bitmex market instrument --symbol XBTUSD -o json
bitmex market orderbook XBTUSD --depth 10 -o json
bitmex market trades --symbol XBTUSD --count 20 -o json
bitmex market funding --symbol XBTUSD -o json

# Set up credentials (stored securely in OS keychain)
bitmex auth add

# Account data
bitmex account me -o json
bitmex wallet balance -o json
bitmex position list -o json

# Place a limit buy (use --testnet first!)
bitmex order buy XBTUSD 100 --price 50000 --validate -o json   # dry-run
bitmex order buy XBTUSD 100 --price 50000 -o json               # live
```

## Testnet

All commands accept `--testnet` to target `https://testnet.bitmex.com` instead of mainnet. Use this to validate workflows without risking real funds:

```bash
bitmex --testnet market instrument --active -o json
bitmex --testnet order buy XBTUSD 100 --price 50000 -o json
```

Testnet requires separate API keys from [testnet.bitmex.com](https://testnet.bitmex.com).

## Commands

Every command returns structured JSON with `-o json`. All commands support `--help`.

### Market data (no auth)

```bash
bitmex market instrument [--symbol XBTUSD] [--active] [--indices]
bitmex market quote [--symbol XBTUSD] [--bucketed] [--bin-size 1h]
bitmex market trades [--symbol XBTUSD] [--count 100]
bitmex market orderbook XBTUSD [--depth 25]
bitmex market funding [--symbol XBTUSD]
bitmex market liquidation [--symbol XBTUSD]
bitmex market settlement [--symbol XBTUSD]
bitmex market insurance
bitmex market stats
bitmex market leaderboard [--method notional|roe]
bitmex market composite-index [--symbol .BXBT]
bitmex announce list
bitmex announce urgent
```

### Orders (auth required)

```bash
bitmex order buy  XBTUSD 100 --price 50000 [--order-type Limit] [--validate]
bitmex order sell XBTUSD 100 --price 52000 [--tif GoodTillCancel]
bitmex order buy  XBTUSD 100 --price 50000 --strategy Long   # Hedge Mode leg
bitmex order sell XBTUSD 100 --price 52000 --strategy Short  # Hedge Mode leg
bitmex order amend --order-id <id> --price 51000
bitmex order cancel --order-id <id>
bitmex order cancel-all [--symbol XBTUSD]
bitmex order cancel-after 60000        # Dead Man's Switch (ms)
bitmex order list [--symbol XBTUSD]
```

100% position-closing orders — `orderQty` is omitted, so they track the full
position dynamically (BitMEX renders them as SL/TP 100%). `--side sell` closes a
long, `--side buy` closes a short. `--trigger` is required with `--stop-px`/`--tp-px`.

```bash
bitmex order close XBTUSD --side sell --stop-px 50000 --trigger mark   # Stop-Loss 100%
bitmex order close XBTUSD --side sell --tp-px 60000 --trigger last     # Take-Profit 100%
bitmex order close XBTUSD --side sell --stop-px 50000 --tp-px 60000 --trigger mark  # OCO bracket
bitmex order close XBTUSD --side sell                                  # market close 100%
```

### Positions (auth required)

```bash
bitmex position list [--symbol XBTUSD]          # table shows the `strategy` column
bitmex position leverage XBTUSD 10
bitmex position cross-leverage XBTUSD 5
bitmex position isolate XBTUSD --enabled
bitmex position risk-limit XBTUSD 20000000000
bitmex position transfer-margin XBTUSD 100000   # satoshis
```

### Hedge Mode (MultiWay) (auth required)

Hedge Mode lets you hold independent Long and Short positions on the same
contract instead of netting into one. It is an account-level setting and applies
to uncapped derivatives only. Switching is rejected while the account has open
orders or isolated-margin positions.

```bash
bitmex account position-mode multiway   # enable Hedge Mode (alias: hedge)
bitmex account position-mode oneway      # back to One-Way (netting)
bitmex position mode multiway            # alias for `account position-mode`
```

Once enabled, tag each order leg with its strategy:

```bash
bitmex order buy  XBTUSD 100 --price 50000 --strategy Long
bitmex order sell XBTUSD 100 --price 52000 --strategy Short
```

See [What is Hedge Mode](https://support.bitmex.com/hc/en-gb/articles/32985620308381-What-is-Hedge-Mode)
and [How traders use Hedge Mode](https://support.bitmex.com/hc/en-gb/articles/36691242524317-How-can-traders-use-Hedge-Mode-to-improve-their-trading-and-strategies).

### Execution history (auth required)

```bash
bitmex execution list [--symbol XBTUSD] [--count 100]
bitmex execution trade-history [--symbol XBTUSD]
```

### Wallet (auth required)

```bash
bitmex wallet balance [--currency XBt]
bitmex wallet history [--currency XBt]
bitmex wallet summary
bitmex wallet deposit [--currency XBt]
bitmex wallet assets
bitmex wallet networks
bitmex wallet withdraw --currency XBt --network Bitcoin --address <addr> --amount <sats>
bitmex wallet transfer --currency XBt --amount <sats>
```

### Account (auth required)

```bash
bitmex account me
bitmex account margin
bitmex account commission
bitmex account volume
bitmex account affiliate
bitmex account csa
bitmex account execution-history
bitmex account preferences --prefs '{"locale":"en-US"}'
bitmex account margining-mode REGULAR_MARGIN
```

### Staking (auth required)

```bash
bitmex staking status
bitmex staking instruments
bitmex staking unstake <symbol> <amount>
bitmex staking pending-unstake
```

### Other command groups

```bash
bitmex subaccount add <name>
bitmex api-key list
bitmex chat read [--channel-id 1]
bitmex chat post "Hello"
bitmex guild info
bitmex bots strategies
bitmex bots instances
bitmex notifications alerts
bitmex address list
bitmex referral list
bitmex porl nonce
```

### Credentials

```bash
bitmex auth add                           # interactive wizard — recommended
bitmex auth set --profile trading         # non-interactive upsert
bitmex auth list                          # list all profiles
bitmex auth show [--profile <name>]       # show masked key + storage source
bitmex auth use <name>                    # change active profile
bitmex auth delete --profile <name>       # remove a profile
bitmex auth reset                         # clear the default profile
```

## API Keys & Configuration

### Credential storage

Credentials are stored in the **OS native keychain** (macOS Keychain, Linux Secret Service, Windows Credential Manager). The config file `~/Library/Application Support/bitmex/config.toml` (macOS) or `~/.config/bitmex/config.toml` (Linux) holds only non-sensitive metadata — profile names, testnet flag, active profile.

On headless systems without a keychain, add `--no-keychain` (or set `BITMEX_NO_KEYCHAIN=1`) to fall back to plaintext storage in the config file.

### Profiles

A profile bundles an API key/secret pair and a testnet flag under a name:

```bash
# Add profiles interactively
bitmex auth add                           # guided wizard
bitmex auth add --profile testnet --testnet

# Switch between profiles
bitmex auth use trading
bitmex --profile trading order list       # override for a single command
```

Credential resolution order (highest wins):
1. `--api-key` / `--api-secret` flags
2. `BITMEX_API_KEY` / `BITMEX_API_SECRET` env vars
3. `--profile <name>` flag → OS keychain
4. `active_profile` from config → OS keychain
5. Plaintext config file fallback (when keychain unavailable)

### Environment variables

```bash
export BITMEX_API_KEY="your-key"
export BITMEX_API_SECRET="your-secret"
export BITMEX_PROFILE="trading"           # select profile by name
export BITMEX_NO_KEYCHAIN=1               # force plaintext fallback (CI/Docker)
```

### Required BitMEX API permissions

| Command group | Required permissions |
|---|---|
| market, announce, chat (read) | None (public) |
| account, execution, position (read) | Order |
| order (place/cancel) | Order |
| wallet | Withdraw |
| staking, apikey, porl | Account |

## MCP Server

`bitmex-cli` includes a built-in [MCP](https://modelcontextprotocol.io) server that exposes CLI commands as tools directly to AI agents — no subprocess wrappers needed.

```bash
bitmex mcp
bitmex mcp -s market,account          # restrict to specific groups
bitmex mcp -s all --allow-dangerous   # all commands including order placement
bitmex mcp -s all --port 3000         # run as standalone HTTP server
```

### Recommended: HTTP mode (credentials stay private)

Running the MCP server as a standalone HTTP process means your API credentials are loaded from the OS keychain by your own terminal session — they are never passed to or visible by the AI agent.

**Step 1** — Store your credentials in the keychain:

```bash
bitmex auth add --profile trading         # interactive wizard
bitmex auth add --profile testnet --testnet
```

**Step 2** — Start the MCP server in a terminal:

```bash
bitmex mcp -s all --allow-dangerous --profile trading --port 3000
```

Credentials are resolved from the keychain once at startup. The AI agent connects over HTTP and never sees the key or secret.

**Step 3** — Point your MCP client at the running server:

```json
{
  "mcpServers": {
    "bitmex": {
      "type": "http",
      "url": "http://127.0.0.1:3000/mcp"
    }
  }
}
```

For Claude Code:

```bash
claude mcp add --transport http bitmex http://127.0.0.1:3000/mcp
```

### Subprocess mode (stdio)

If you prefer the client to spawn the server automatically, add it to your MCP config. Credentials can be passed via a named profile (read from the keychain at each call) or explicitly via environment variables.

**Using a keychain profile:**

```json
{
  "mcpServers": {
    "bitmex": {
      "command": "bitmex",
      "args": ["mcp", "-s", "all", "--profile", "trading"]
    }
  }
}
```

**Using environment variables (CI / Docker / no keychain):**

```json
{
  "mcpServers": {
    "bitmex": {
      "command": "bitmex",
      "args": ["mcp", "-s", "all"],
      "env": {
        "BITMEX_API_KEY": "your-key",
        "BITMEX_API_SECRET": "your-secret"
      }
    }
  }
}
```

> **Note:** In subprocess mode the MCP client config contains credentials. Use HTTP mode to keep credentials out of config files and away from AI agents.

## WebSocket Streaming

Stream real-time data as NDJSON to stdout:

```bash
# Public topics
bitmex ws trade:XBTUSD
bitmex ws orderBookL2_25:XBTUSD
bitmex ws funding instrument

# Private topics (require --auth)
bitmex ws --auth position order execution margin

# Multiple topics
bitmex ws trade:XBTUSD orderBookL2_25:XBTUSD quote:XBTUSD
```

Use `--testnet` to stream from testnet:

```bash
bitmex --testnet ws trade:XBTUSD
```

## AI Agents

Every command returns structured JSON with `-o json`. The built-in MCP server exposes commands as tools directly to AI agents.

| Resource | Description |
|----------|-------------|
| [AGENTS.md](AGENTS.md) | Full integration guide — installation, auth, invocation, error handling, safety |
| [CONTEXT.md](CONTEXT.md) | Compact runtime context for tool-using agents |
| [agents/tool-catalog.json](agents/tool-catalog.json) | 134 commands with parameter schemas and `dangerous` flags |
| [agents/error-catalog.json](agents/error-catalog.json) | Error categories with retry guidance |
| [skills/](skills/) | 49 workflow skills — DCA bots, grid trading, market briefs, risk operations |

## Interactive Shell

```bash
bitmex shell
```

Runs an interactive REPL with command history at `~/.config/bitmex/history`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License & Disclaimer

MIT — see [LICENSE](LICENSE).

**This software interacts with a live financial exchange and can result in real monetary loss. It is provided as-is with no warranty. You are solely responsible for all trading decisions and outcomes.**

Use of this tool is subject to the [BitMEX Terms of Service](https://www.bitmex.com/terms). Read [DISCLAIMER.md](DISCLAIMER.md) before use.
