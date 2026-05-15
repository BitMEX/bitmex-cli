---
name: bitmex-recipe-emergency-flatten
description: Cancel all orders and close all positions immediately.
---

# Emergency Flatten

Use this when you need to exit all exposure as fast as possible — runaway loss, infra failure, or unexpected news. This workflow cancels every open order first, then closes each open position at market.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` must be set.
- Confirm you intend to close real positions before proceeding.

## Dead Man's Switch (optional but recommended)

Before starting any risky automated operation, arm a dead man's switch so positions close automatically if your process dies:

```bash
bitmex order cancel-after 60000 -o json
```

This instructs BitMEX to cancel all orders if no heartbeat is received within 60 seconds. Renew it regularly in your loop.

## Steps

### 1. Cancel all open orders

```bash
bitmex order cancel-all --yes -o json
```

Verify the cancellation count in the response `count` field.

### 2. List remaining open positions

```bash
bitmex position list -o json \
  | jq '[.[] | select(.currentQty != 0) | {symbol, currentQty, avgEntryPrice, unrealisedPnl}]'
```

### 3. Close each open position at market

Repeat for every symbol returned above:

```bash
bitmex order close-position XBTUSD -o json
bitmex order close-position ETHUSD -o json
```

`close-position` sends a market order for the full open quantity on that symbol.

### 4. Verify all positions are flat

```bash
bitmex position list -o json \
  | jq '[.[] | select(.currentQty != 0)] | length'
```

Expected result: `0`. If non-zero, the market order may still be in-flight — re-check after a few seconds and repeat step 3 for any remaining symbols.

## Notes

- Market close orders consume taker fees. On thin markets the slippage can be significant.
- For large positions, consider splitting into two market orders to reduce impact.
- Always test this flow on `--testnet` first so it executes in under 10 seconds when needed.
