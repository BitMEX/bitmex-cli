# bitmex-cli Runtime Context for AI Agents

This file is optimised for runtime agent use. It defines how to call `bitmex` safely and reliably.

## Installation

```bash
curl -sSfL https://raw.githubusercontent.com/BitMEX/bitmex-cli/master/install.sh | sh
```

Pre-built binary for macOS and Linux (x86_64, arm64). No Rust needed. Verify: `bitmex --version`.

## Core Invocation Contract

Always call:

```bash
bitmex <command-group> <subcommand> [args...] -o json 2>/dev/null
```

Rules:
- Always pass `-o json`.
- Treat `stdout` as the only machine data channel.
- Treat `stderr` as diagnostics only (warnings, migration notices, verbose output).
- Exit code `0` = success. Non-zero = failure; `stdout` contains a JSON error envelope.

## Authentication

### Recommended: environment variables

```bash
export BITMEX_API_KEY="your-key"
export BITMEX_API_SECRET="your-secret"
```

### Profile-based (for multi-account setups)

```bash
export BITMEX_PROFILE="trading"   # select a named keychain profile
```

Profiles are created with `bitmex auth add` and stored in the OS keychain (macOS Keychain,
Linux Secret Service, Windows Credential Manager). Use `--no-keychain` or
`BITMEX_NO_KEYCHAIN=1` to force plaintext config-file storage in CI/headless environments.

### Credential resolution order

1. `--api-key` / `--api-secret` flags
2. `BITMEX_API_KEY` / `BITMEX_API_SECRET` env vars
3. `--profile <name>` flag → OS keychain
4. `BITMEX_PROFILE` / active profile in config → OS keychain
5. Plaintext fallback in config file (when keychain unavailable)

Public market data (market, announce, chat read) requires no credentials.

## Testnet

Add `--testnet` to any command to target `https://testnet.bitmex.com`:

```bash
bitmex --testnet market instrument --active -o json
```

Profiles can store a `testnet` flag — a profile created with `bitmex auth add --testnet`
will automatically target testnet without needing the flag each time.

## Asset Coverage

BitMEX trades crypto derivatives. Key symbols:

| Type | Example symbols |
|------|----------------|
| Perpetual (inverse) | `XBTUSD`, `ETHUSD` |
| Perpetual (linear) | `XBTUSDT`, `ETHUSDT` |
| Fixed-date futures | `XBTH25`, `XBTM25` |
| Spot | `XBT`, `ETH` (wallet balances) |

Use `bitmex market instrument --active -o json` to list all tradeable instruments.

## Command Groups

| Group | Auth | Description |
|-------|------|-------------|
| `market` | No | Instruments, quotes, trades, orderbook, funding, liquidations, stats |
| `order` | Yes | Place, amend, cancel orders |
| `position` | Yes | List positions, set leverage, manage margin |
| `execution` | Yes | Fill history, trade history with PnL |
| `wallet` | Yes | Balances, deposits, withdrawals, transfers |
| `staking` | Yes | Staking positions and instruments |
| `account` | Yes | Profile, margin, commission, preferences |
| `subaccount` | Yes | Subaccount management |
| `apikey` | Yes | API key listing |
| `chat` | Mixed | Trollbox read (public), post (auth) |
| `guild` | Yes | Guild management |
| `bots` | Yes | Trading bots |
| `notifications` | Yes | Price alerts, user events |
| `address` | Yes | Withdrawal address book |
| `referral` | Yes | Referral codes |
| `porl` | Yes | Proof of Reserves verification |
| `announce` | No | Site announcements |
| `ws` | Mixed | WebSocket streaming (NDJSON to stdout) |

## Safety Rules

1. **Never place orders without explicit user approval.** Always validate first with `--validate`.
2. **Use `--testnet` for strategy testing.** No real money involved.
3. **Check `dangerous: true` in `agents/tool-catalog.json`** before executing any command.
4. **Gate withdrawals and transfers behind confirmation.** Use `--yes` only after explicit user approval.
5. **Never log or print `BITMEX_API_SECRET`.**

## Error Envelope

On failure, exit code is non-zero and stdout contains:

```json
{ "error": "<category>", "message": "<human-readable description>" }
```

The `error` field is a stable category code for programmatic routing:

| Category | Meaning | Retry? |
|----------|---------|--------|
| `api` | Exchange rejected the request (4xx other than 429) | Depends on message |
| `auth` | Invalid key, bad signature, insufficient permissions | No |
| `network` | TCP connection failure or timeout | Yes |
| `rate_limit` | HTTP 429 or 503 (overloaded) | Yes — wait for reset |
| `validation` | Bad CLI arguments | No |
| `config` | Missing or invalid configuration | No |
| `websocket` | WebSocket connection error | Yes |
| `io` | Local file I/O error | No |
| `parse` | Unexpected response format from exchange | No |

### Rate limit errors

When `error` is `rate_limit`, three additional fields are present as client-side hints
to assist retry logic. **These are not from the BitMEX API response** — they are added
by the CLI itself:

```json
{
  "error": "rate_limit",
  "message": "BitMEX rate limit exceeded: ... Retry after Unix timestamp 1713229600.",
  "suggestion": "BitMEX allows 300 requests per 5 minutes. Back off and retry after x-ratelimit-reset (Unix seconds).",
  "retryable": true,
  "docs_url": "https://www.bitmex.com/app/restAPI#Rate-Limits"
}
```

- `message` — includes the Unix retry timestamp parsed from the `x-ratelimit-reset` response header (when present)
- `suggestion` — static guidance string for AI agents
- `retryable` — always `true` for 429/503; included for programmatic decision-making
- `docs_url` — link to BitMEX rate limit documentation

The actual retry deadline is in the `message` field as a Unix timestamp. Parse it or
read the `x-ratelimit-reset` header directly from the HTTP response if you have access to it.

## Rate Limits

300 requests / 5 minutes for authenticated REST endpoints. Response headers:

- `x-ratelimit-remaining` — requests left in the current window
- `x-ratelimit-reset` — Unix timestamp when the budget resets

On `error: "rate_limit"`, wait until `x-ratelimit-reset`. Use WebSocket for real-time data instead of polling REST.

## Key Commands

```bash
# Market data
bitmex market instrument --symbol XBTUSD -o json
bitmex market orderbook XBTUSD --depth 10 -o json
bitmex market trades --symbol XBTUSD --count 20 -o json
bitmex market funding --symbol XBTUSD -o json

# Account
bitmex account me -o json
bitmex wallet balance -o json
bitmex position list -o json
bitmex execution trade-history --count 20 -o json

# Orders
bitmex order buy  XBTUSD 100 --price 50000 --validate -o json  # dry-run
bitmex order buy  XBTUSD 100 --price 50000 --yes -o json       # live
bitmex order sell XBTUSD 100 --price 52000 --yes -o json
bitmex order cancel --order-id <id> --yes -o json
bitmex order cancel-all --symbol XBTUSD --yes -o json
bitmex order list --symbol XBTUSD -o json

# Positions
bitmex position list -o json
bitmex position leverage XBTUSD 10 -o json

# WebSocket (NDJSON to stdout)
bitmex ws trade:XBTUSD orderBookL2_25:XBTUSD
bitmex ws --auth position order execution
```

## Order Constraints: Tick Size and Lot Size

Before placing an order, fetch the instrument's constraints:

```bash
constraints=$(bitmex market instrument --symbol XBTUSD -o json | jq '.[0] | {tickSize, lotSize}')
tick_size=$(echo "$constraints" | jq -r '.tickSize')
lot_size=$(echo "$constraints" | jq -r '.lotSize')
```

| Field | Meaning | Violation error |
|-------|---------|----------------|
| `tickSize` | Minimum price increment — price must be a multiple | `400 Invalid price` |
| `lotSize` | Minimum quantity increment — qty must be a multiple | `400 Invalid quantity` |

Round before submitting:

```bash
# price rounded to nearest tick
price=$(echo "$raw_price $tick_size" | awk '{printf "%g", int($1/$2+0.5)*$2}')
# qty rounded down to nearest lot
qty=$(echo "$raw_qty $lot_size" | awk '{printf "%g", int($1/$2)*$2}')
```

`--validate` is a local dry-run that prints the constructed order JSON without submitting to the exchange. It does not perform server-side price/qty validation — alignment must be correct before live submission.

## MCP Server

```bash
bitmex mcp -s market,account,order,position,wallet
```

Exposes commands as MCP tools over stdio. Credentials are read from the active keychain
profile automatically. See `AGENTS.md` for full MCP config.
