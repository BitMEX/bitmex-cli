# Agent Integration Guide: bitmex-cli

> See [README.md](README.md) for safety warnings and disclaimer.

Self-contained guide for integrating `bitmex-cli` into AI agents, MCP clients, and automated pipelines.

Fast entry points:
- Runtime agent context: `CONTEXT.md`
- Full command contract: `agents/tool-catalog.json`
- Error routing contract: `agents/error-catalog.json`
- Workflow skills: `skills/`

## Installation

### Pre-built binary (recommended)

```bash
curl -sSfL https://raw.githubusercontent.com/BitMEX/bitmex-cli/master/install.sh | sh
```

Downloads a pre-built binary for your platform (macOS/Linux, x86_64/arm64), verifies the SHA256 checksum, and installs to `/usr/local/bin`. No Rust or build tools needed. Requires `curl`, `tar`, and `sha256sum` (or `shasum`). May prompt for `sudo` if `/usr/local/bin` is not writable.

### From source (requires Rust)

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and install
git clone https://github.com/BitMEX/bitmex-cli.git
cd bitmex-cli
cargo install --path .
```

### Verify

```bash
bitmex --version
```

## Authentication

Public commands (market data, announcements) need no credentials. All other commands require a BitMEX API key.

### New users

If the user does not have a BitMEX account:
1. Sign up at [bitmex.com/register](https://www.bitmex.com/register)
2. Complete identity verification (Settings → Verification)
3. Deposit funds (Wallet → Deposit)

### API key setup

1. Direct the user to [bitmex.com/app/apiKeys](https://www.bitmex.com/app/apiKeys) (or [testnet.bitmex.com/app/apiKeys](https://testnet.bitmex.com/app/apiKeys) for testnet).
2. Create a key with **Order** and **Account** permissions (add **Withdraw** only if withdrawals/transfers are needed).
3. Ask the user for the API key and secret.
4. Store credentials:

```bash
bitmex auth set --api-key <KEY> --api-secret <SECRET>
```

For testnet:

```bash
bitmex auth set --profile testnet --testnet --api-key <KEY> --api-secret <SECRET>
```

5. Verify:

```bash
bitmex account me -o json
```

### Alternative: environment variables

For CI, Docker, or single-session use:

```bash
export BITMEX_API_KEY="..."
export BITMEX_API_SECRET="..."
```

### Credential resolution order

1. `--api-key` / `--api-secret` flags
2. `BITMEX_API_KEY` / `BITMEX_API_SECRET` env vars
3. `--profile <name>` flag → OS keychain
4. `BITMEX_PROFILE` / active profile in config → OS keychain
5. Plaintext fallback in config file (when keychain unavailable)

### Required permissions by command group

| Group | BitMEX API permissions |
|-------|----------------------|
| market, announce, chat (read) | None (public) |
| account, execution, position (read) | Order |
| order (place/cancel) | Order |
| wallet (read) | Account |
| wallet (withdraw/transfer) | Withdraw |
| staking, apikey, porl | Account |

## Invocation Pattern

```bash
bitmex <command-group> <subcommand> [args...] -o json 2>/dev/null
```

- Always pass `-o json` for machine-readable output.
- Redirect stderr (`2>/dev/null`) to suppress diagnostic noise.
- Check exit code: `0` = success, non-zero = failure.
- On failure, stdout contains a JSON error envelope.

## Testnet Mode

All commands accept `--testnet` to target `https://testnet.bitmex.com`. Use testnet to validate agent workflows without risking real funds:

```bash
bitmex --testnet market instrument --active -o json
bitmex --testnet order buy XBTUSD 100 --price 50000 --validate -o json
```

Testnet requires separate API keys from [testnet.bitmex.com](https://testnet.bitmex.com).

## Output Format

All commands return JSON on stdout. Arrays of objects are typical for list endpoints:

```json
[
  {
    "symbol": "XBTUSD",
    "lastPrice": 50000,
    "markPrice": 49998,
    "fundingRate": 0.0001,
    ...
  }
]
```

Error envelopes always contain `error` (stable category code) and `message`:

```json
{ "error": "auth", "message": "Authentication failed: Invalid API Key." }
```

## Error Categories

| Category | Meaning | Retry? |
|----------|---------|--------|
| `api` | Exchange rejected the request (4xx other than 429) | Depends on message |
| `auth` | Invalid key, bad signature, insufficient permissions | No |
| `network` | TCP connection failure or timeout | Yes |
| `rate_limit` | HTTP 429 or 503 (exchange overloaded) | Yes — wait for reset |
| `validation` | Bad CLI arguments | No |
| `config` | Missing or invalid configuration | No |
| `websocket` | WebSocket connection error | Yes |
| `io` | Local file I/O error | No |
| `parse` | Unexpected response format | No |

### Rate limit envelope

`rate_limit` errors include three extra fields added **client-side** (not from BitMEX):

```json
{
  "error": "rate_limit",
  "message": "BitMEX rate limit exceeded: ... Retry after Unix timestamp 1713229600.",
  "suggestion": "BitMEX allows 300 requests per 5 minutes. Back off and retry after x-ratelimit-reset.",
  "retryable": true,
  "docs_url": "https://www.bitmex.com/app/restAPI#Rate-Limits"
}
```

- `message` includes the Unix retry timestamp from the `x-ratelimit-reset` response header
- `suggestion` / `retryable` / `docs_url` are static hints for AI agent retry logic

## Safety Protocol for Agents

1. **Never execute dangerous commands** without explicit user approval. Check the `dangerous` field in `agents/tool-catalog.json`.
2. **Validate orders first**: always pass `--validate` before submitting live orders.
3. **Test on testnet**: use `--testnet` for new strategies.
4. **Confirm destructive actions**: cancellations, withdrawals, transfers require `--yes` flag or interactive confirmation.
5. **Never log secrets**: never print `BITMEX_API_SECRET` in output.

## Dangerous Commands (require --yes or user confirmation)

All order placement, amendment, and cancellation commands. All withdrawal and transfer commands. Position isolation, leverage changes, margin transfers. See `dangerous: true` entries in `agents/tool-catalog.json`.

## MCP Server

The built-in MCP server exposes bitmex-cli commands as tools directly to MCP clients:

```bash
bitmex mcp                            # market + account (safe defaults)
bitmex mcp -s all                     # all groups except streaming
bitmex mcp -s all --allow-dangerous   # all commands including order placement
```

Claude Desktop config (credentials from OS keychain):

```json
{
  "mcpServers": {
    "bitmex": {
      "command": "bitmex",
      "args": ["mcp", "-s", "market,account,order,position,wallet"]
    }
  }
}
```

For CI or environments without a keychain, pass credentials via environment variables:

```json
{
  "mcpServers": {
    "bitmex": {
      "command": "bitmex",
      "args": ["mcp", "-s", "market,account,order,position,wallet"],
      "env": {
        "BITMEX_API_KEY": "your-key",
        "BITMEX_API_SECRET": "your-secret"
      }
    }
  }
}
```

## WebSocket Streaming

For real-time data, use `bitmex ws` instead of polling REST endpoints:

```bash
# Public — no auth needed
bitmex ws trade:XBTUSD
bitmex ws orderBookL2_25:XBTUSD instrument

# Private — requires --auth
bitmex ws --auth position order execution margin wallet
```

Output is NDJSON (one JSON object per line) to stdout. Ctrl-C to stop.

Available public topics: `trade`, `quote`, `instrument`, `orderBookL2_25`, `orderBook10`, `funding`, `liquidation`, `settlement`, `insurance`, `announcement`, `chat`

Available private topics: `order`, `execution`, `position`, `margin`, `wallet`, `transact`, `affiliate`, `privateNotifications`

## Rate Limits

BitMEX allows 300 requests per 5 minutes for authenticated REST endpoints. Check response headers:

- `x-ratelimit-remaining` — requests left in current window
- `x-ratelimit-reset` — Unix timestamp when budget resets

When rate limited (`error: "rate_limit"`), wait until `x-ratelimit-reset` before retrying. Use WebSocket streaming for real-time data instead of polling.

## Common Agent Patterns

### Morning market brief

```bash
bitmex market instrument --symbol XBTUSD -o json
bitmex market funding --symbol XBTUSD --count 3 --reverse -o json
bitmex market stats -o json
```

### Monitor positions and PnL

```bash
bitmex position list -o json
bitmex execution trade-history --count 20 -o json
bitmex wallet balance -o json
```

### Hedge Mode (MultiWay)

Hedge Mode is an account-level setting that lets you hold independent Long and Short positions on the same contract (uncapped derivatives only). Switching is rejected while the account has open orders or isolated-margin positions.

```bash
bitmex account position-mode multiway --yes -o json   # enable (alias: hedge)
bitmex account position-mode oneway --yes -o json      # disable (netting)
```

Once enabled, tag each order leg with `--strategy Long` or `--strategy Short`. Each position carries a `strategy` field (`Long`/`Short`/`OneWay`); the account carries `positionMode`.

```bash
bitmex order buy  XBTUSD 100 --price 50000 --strategy Long --yes -o json
bitmex order sell XBTUSD 100 --price 52000 --strategy Short --yes -o json
```

### Tick size and lot size alignment

Every instrument enforces a minimum price increment (`tickSize`) and minimum quantity increment (`lotSize`). Submitting a price or quantity that isn't a multiple of these will return a `400 Invalid price` or `400 Invalid quantity` error.

Fetch constraints before placing:

```bash
constraints=$(bitmex market instrument --symbol XBTUSD -o json | jq '.[0] | {tickSize, lotSize}')
tick_size=$(echo "$constraints" | jq -r '.tickSize')
lot_size=$(echo "$constraints" | jq -r '.lotSize')
```

Round before submitting:

```bash
# round price to nearest tick
price=$(echo "$raw_price $tick_size" | awk '{printf "%g", int($1/$2+0.5)*$2}')
# round qty down to nearest lot
qty=$(echo "$raw_qty $lot_size" | awk '{printf "%g", int($1/$2)*$2}')
```

### Safe order placement workflow

```bash
# 1. Fetch tick/lot constraints and align price/qty before submitting
constraints=$(bitmex market instrument --symbol XBTUSD -o json | jq '.[0] | {tickSize, lotSize}')

# 2. Preview the constructed request body (local only — does not validate against exchange)
bitmex order buy XBTUSD 100 --price 50000 --validate -o json

# 3. Confirm with user, then execute (--yes skips interactive prompt for agent use)
bitmex order buy XBTUSD 100 --price 50000 --yes -o json
```

### Stream live data to a pipeline

```bash
bitmex ws trade:XBTUSD | jq -c '{time: .timestamp, price: .price, size: .size}'
```
