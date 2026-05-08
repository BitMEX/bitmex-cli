---
name: bitmex-testnet-to-live
version: 1.0.0
description: "Promotion from BitMEX testnet to live: checklist, small-size ramp-up, safety controls, and rollback procedure."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-testnet-strategy", "bitmex-risk-operations", "bitmex-liquidation-guard"]
---

# bitmex-testnet-to-live

Promote a strategy from testnet to live only after systematic verification. Start with 10% of intended size and scale up gradually.

## Pre-Promotion Checklist

Complete all items before switching to live credentials:

- [ ] Strategy ran for 5+ complete sessions on testnet
- [ ] PnL is consistent and explained (not lucky outliers)
- [ ] Stop losses triggered correctly in at least 2 tests
- [ ] Dead man's switch tested: process kill cancels all orders within timeout
- [ ] Error recovery tested: network drop, rate limit, auth failure
- [ ] Position cap enforced: orders are rejected when limit is reached
- [ ] No open positions or orders remain after session end
- [ ] Average fill prices are within expected slippage bounds
- [ ] All `--validate` calls succeed before live calls are made

## Testnet PnL Summary

Run before deciding to promote:

```bash
bitmex --testnet execution trade-history --reverse --count 200 -o json 2>/dev/null | \
  jq '
    {
      sessions: "review manually",
      trades: length,
      realised_pnl_sats: (map(.realisedPnl // 0) | add),
      total_fees_sats: (map(.commission // 0) | add),
      win_rate: (
        (map(select((.realisedPnl // 0) > 0)) | length) /
        length * 100 | round / 100
      )
    }
  '
```

## Switch to Live Credentials

```bash
# Unset testnet credentials
unset BITMEX_API_KEY
unset BITMEX_API_SECRET

# Set live credentials
export BITMEX_API_KEY="live-api-key"
export BITMEX_API_SECRET="live-api-secret"

# Confirm live account
bitmex account me -o json 2>/dev/null | jq '{username, id}'
bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '{marginBalance, availableMargin}'
```

## First Live Session: 10% Size

Start with 10% of the intended production position size:

```bash
PRODUCTION_QTY=1000
INITIAL_QTY=$((PRODUCTION_QTY / 10))  # 100

echo "First live order: $INITIAL_QTY contracts (10% of intended $PRODUCTION_QTY)"

# Pre-flight
bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '{availableMargin, marginBalance}'

# Enable dead man's switch immediately
bitmex order cancel-after 60000 -o json 2>/dev/null

# Place first live order
bitmex order buy XBTUSD $INITIAL_QTY \
  --price 50000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  --validate -o json 2>/dev/null

# Wait for user to confirm, then execute
bitmex order buy XBTUSD $INITIAL_QTY \
  --price 50000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null | jq '{orderID, ordStatus, price}'
```

## Confirm First Fill

```bash
bitmex ws --auth execution -o json 2>/dev/null | \
  jq -c '.data[]? | select(.execType == "Trade") | {symbol, side, lastPx, lastQty, ordStatus}'
```

## Scale-Up Gate

Only increase size after verifying each milestone:

| Session | Max Size | Condition to Advance |
|---|---|---|
| 1–2 | 10% | Fills match expected price, stop fired correctly |
| 3–4 | 25% | PnL positive, no unexpected positions |
| 5–6 | 50% | Error recovery tested on live |
| 7+ | 100% | Full review, user sign-off |

## Safety Controls for Live

```bash
MAX_POSITION=500  # live cap during ramp-up

# Position cap check (run before every order)
CURRENT=$(bitmex position list --symbol XBTUSD -o json 2>/dev/null | \
  jq -r '.[0].currentQty // 0')
if [ "${CURRENT#-}" -ge "$MAX_POSITION" ]; then
  echo "Live cap reached: $CURRENT / $MAX_POSITION — halting"
  exit 1
fi
```

## Rollback Procedure

If live behavior diverges from testnet (unexpected fills, wrong side, margin burn):

```bash
# 1. Cancel all orders
bitmex order cancel-all -o json 2>/dev/null

# 2. Close all positions
bitmex position list -o json 2>/dev/null | \
  jq -r '[.[] | select(.isOpen == true and .currentQty != 0) | .symbol] | .[]' | \
  while read -r SYM; do
    bitmex order close-position "$SYM" -o json 2>/dev/null
  done

# 3. Capture state for review
bitmex execution trade-history --reverse --count 50 -o json 2>/dev/null > /tmp/live-rollback-$(date +%s).json
bitmex account margin --currency XBt -o json 2>/dev/null

# 4. Return to testnet for investigation
export BITMEX_API_KEY="testnet-key"
export BITMEX_API_SECRET="testnet-secret"
```
