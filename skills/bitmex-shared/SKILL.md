---
name: bitmex-shared
version: 1.0.0
description: "Shared runtime contract for bitmex-cli: auth, invocation, parsing, and safety."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
---

# bitmex-shared

**This tool is experimental. Commands execute real financial transactions on BitMEX. Use `--testnet` before using real funds. See `DISCLAIMER.md` for full terms.**

## Invocation Contract

Always call:

```bash
bitmex <command-group> <subcommand> [args...] -o json 2>/dev/null
```

Rules:
- Parse `stdout` only.
- Treat `stderr` as diagnostics.
- Exit code `0` = success.
- Non-zero exit = failure; `stdout` contains a JSON error envelope.

## Authentication

```bash
export BITMEX_API_KEY="your-key"
export BITMEX_API_SECRET="your-secret"
```

Public market data (`market`, `announce`, `chat read`) requires no credentials. All account, order, position, wallet, and staking commands require auth.

## Testnet

Use `--testnet` for all testing. Connects to `testnet.bitmex.com`. No real money at risk. Obtain testnet API keys from `testnet.bitmex.com/app/apiKeys`.

```bash
bitmex --testnet market instrument --active -o json
bitmex --testnet order buy XBTUSD 100 --price 50000 --validate -o json
bitmex --testnet position list -o json
```

## Error Envelope

```json
{ "error": "<category>", "message": "<description>" }
```

Categories: `api`, `auth`, `network`, `rate_limit`, `validation`, `config`, `websocket`, `io`, `parse`.

Rate limit errors include `suggestion`, `retryable`, and `docs_url` fields:

```json
{ "error": "rate_limit", "message": "...", "suggestion": "Wait 47s", "retryable": true }
```

## Auth Mechanism

BitMEX uses HMAC-SHA256 signatures. Every authenticated request includes `api-key`, `api-expires`, and `api-signature` headers. The CLI handles this automatically when `BITMEX_API_KEY` and `BITMEX_API_SECRET` are set.

```bash
# Verify credentials are working
bitmex account me -o json 2>/dev/null | jq '.username'
```

## Config File

Configuration is stored at `~/.config/bitmex/config.toml`. Use `bitmex auth set` to write credentials:

```bash
bitmex auth set --api-key <key> --api-secret <secret>
bitmex auth show   # confirm (secret is masked)
bitmex auth reset  # clear stored credentials
```

## Safety Rules

1. Never place orders without explicit user approval.
2. Always use `--validate` before live order submission.
3. Use `--testnet` for all new strategies.
4. Never log or print `BITMEX_API_SECRET`.
5. Gate withdrawals, transfers, and cancellations behind `--yes` or explicit user confirmation.
6. Always check exit code before acting on output.

## Dangerous Commands

Any command that modifies state is considered dangerous: `order buy/sell/amend/cancel/cancel-all/close-position`, `position leverage/isolate/transfer-margin/risk-limit`, `wallet withdraw/transfer`, `staking unstake`. Never execute these without explicit user confirmation.

## Error Handling Pattern

```bash
RESULT=$(bitmex market instrument --symbol XBTUSD -o json 2>/dev/null)
if [ $? -ne 0 ]; then
  CATEGORY=$(echo "$RESULT" | jq -r '.error // "unknown"')
  MESSAGE=$(echo "$RESULT" | jq -r '.message // ""')
  echo "Error [$CATEGORY]: $MESSAGE"
  exit 1
fi
echo "$RESULT" | jq '.'
```
