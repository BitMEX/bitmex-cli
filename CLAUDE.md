# CLAUDE.md

> See [README.md](README.md) for safety warnings and disclaimer.

Integration guidance for Claude Code, Claude Desktop, and other Anthropic-powered agents interacting with `bitmex-cli`.

Fast entry points:
- Runtime context: `CONTEXT.md`
- Full command contract: `agents/tool-catalog.json`

## What is bitmex-cli?

A command-line interface for trading on BitMEX — perpetual contracts, futures, and spot. Every command returns structured JSON. Designed for AI agents and automated pipelines.

## Installation

```bash
curl -sSfL https://raw.githubusercontent.com/BitMEX/bitmex-cli/master/install.sh | sh
```

Pre-built binary, no Rust needed. Verify: `bitmex --version`.

For source builds: install Rust via [rustup](https://rustup.rs), then `cargo install --path .`.

## Invocation

Call `bitmex` as a subprocess. Always use `-o json` and redirect stderr:

```bash
bitmex <command> [args...] -o json 2>/dev/null
```

## Authentication

Set environment variables before invoking:

```bash
export BITMEX_API_KEY="your-key"
export BITMEX_API_SECRET="your-secret"
```

### Credential setup

If the user needs credentials:
1. Direct them to [bitmex.com/app/apiKeys](https://www.bitmex.com/app/apiKeys) to create an API key with **Order** and **Account** permissions.
2. Ask for the key and secret, then store:

```bash
bitmex auth set --api-key <KEY> --api-secret <SECRET>
```

3. Verify: `bitmex account me -o json`

Public market data commands (market, announce, chat read) require no credentials.

## Key Conventions

- stdout is always valid JSON on success, or a JSON error envelope on failure.
- The `error` field in error envelopes is a stable category code: `api`, `auth`, `network`, `rate_limit`, `validation`, `config`, `websocket`, `io`, `parse`.
- WebSocket commands emit NDJSON (one JSON object per line).
- `--testnet` flag targets `https://testnet.bitmex.com` — no real money.
- Exit code 0 = success, non-zero = failure.

## Safety Rules

1. Never execute any command marked `dangerous` without explicit user confirmation. The `dangerous` field in `agents/tool-catalog.json` is the authoritative list.
2. Use `--validate` flag to dry-run order commands before submitting.
3. Use `--testnet` for testing strategies safely.
4. Gate all order placement, cancellation, withdrawal, and transfer operations behind user approval.
5. Never log or display API secrets.

## Common Operations

### Get market data (no auth)

```bash
bitmex market instrument --symbol XBTUSD -o json
bitmex market orderbook XBTUSD --depth 10 -o json
bitmex market trades --symbol XBTUSD --count 20 -o json
bitmex market funding --symbol XBTUSD -o json
```

### Check account (auth required)

```bash
bitmex account me -o json
bitmex wallet balance -o json
bitmex position list -o json
bitmex execution trade-history -o json
```

### Place orders (auth required, dangerous)

Prices must be multiples of `tickSize`; quantities must be multiples of `lotSize`. Fetch these from `bitmex market instrument --symbol <SYMBOL> -o json | jq '.[0] | {tickSize, lotSize}'` before placing. Misaligned values return `400 Invalid price` or `400 Invalid quantity`.

```bash
# Fetch constraints
constraints=$(bitmex market instrument --symbol XBTUSD -o json | jq '.[0] | {tickSize, lotSize}')
tick_size=$(echo "$constraints" | jq -r '.tickSize')
lot_size=$(echo "$constraints" | jq -r '.lotSize')

# Preview request body (local dry-run only — no exchange-side validation)
bitmex order buy XBTUSD 100 --price 50000 --validate -o json

# Execute (--yes required for non-interactive/agent use)
bitmex order buy XBTUSD 100 --price 50000 --yes -o json
```

### Peg, chaser & trailing-stop orders (auth required, dangerous)

Pegged orders price relative to the live market. `chase` and `trailing-stop` are guided
wrappers that set `ordType`/`execInst` and enforce the offset sign locally (a wrong sign
returns a `validation` error before reaching the exchange). Their sign rules are opposite:

```bash
# Pegged limit — requires exec_inst Fixed (PrimaryPeg = near touch, MarketPeg = far touch)
bitmex order buy XBTUSD 100 --order-type Pegged --exec-inst Fixed \
  --peg-price-type PrimaryPeg --peg-offset-value -1 --yes -o json

# Chaser — follows the top of book. Buy offset <= 0, Sell offset >= 0. Max 5 per account.
bitmex order chase XBTUSD Buy 100 --offset -1 --yes -o json
bitmex order chase XBTUSD Sell 100 --offset 1 --bothways --yes -o json

# Trailing stop — Sell offset <= 0, Buy offset >= 0 (opposite of chaser)
bitmex order trailing-stop XBTUSD Sell 100 --offset -100 --yes -o json
```

### Testnet

```bash
bitmex --testnet market instrument --active -o json
bitmex --testnet order buy XBTUSD 100 --price 50000 -o json
```

### Error handling

```bash
RESULT=$(bitmex account me -o json 2>/dev/null)
if [ $? -ne 0 ]; then
  CATEGORY=$(echo "$RESULT" | jq -r '.error // "unknown"')
  # Route on category: auth, rate_limit, network, api, validation, etc.
fi
```

## Tool Discovery

Load `agents/tool-catalog.json` for the full machine-readable command contract. Each entry includes parameters, types, auth requirements, and a `dangerous` flag.

## Full Documentation

- `AGENTS.md`: Complete agent integration guide
- `CONTEXT.md`: Runtime-optimized context for tool-using agents
- `agents/tool-catalog.json`: All commands with parameters
- `agents/error-catalog.json`: Error categories with retry guidance
