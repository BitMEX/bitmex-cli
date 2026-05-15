---
name: bitmex-recipe-trailing-stop-runner
description: Ride a trend with a trailing stop that locks in profits on reversal.
---

# Trailing Stop Runner

Enter a directional position and manage it with a trailing stop that ratchets up as price moves in your favour, then exits on reversal.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` installed.
- Decide: `SYMBOL`, `QTY`, `TRAIL_PCT` (e.g. `2` for 2%).

## Steps

### 1. Enter the position (validate first)

```bash
ENTRY_PRICE=95000
bitmex order buy XBTUSD 100 --order-type Limit --price $ENTRY_PRICE --validate -o json
# If validate looks good, submit:
bitmex order buy XBTUSD 100 --order-type Limit --price $ENTRY_PRICE --yes -o json
```

### 2. Set an initial stop-loss

Place a stop at 2% below entry with `ReduceOnly` so it can only close, not flip:

```bash
STOP_PRICE=$(echo "$ENTRY_PRICE * 0.98" | bc | xargs printf "%.0f")
bitmex order sell XBTUSD 100 --order-type Stop \
  --stop-px $STOP_PRICE --exec-inst ReduceOnly --yes -o json \
  | jq '{orderID, stopPx, ordStatus}'
```

Save the `orderID` returned for later cancellation.

### 3. Monitor price via WebSocket

```bash
bitmex ws instrument:XBTUSD -o json | jq -c '.data[0].lastPrice'
```

### 4. Trail the stop as price rises

When price moves up by `TRAIL_PCT`, cancel the old stop and place a new one:

```bash
NEW_STOP=$(echo "$CURRENT_PRICE * 0.98" | bc | xargs printf "%.0f")

# Cancel old stop
bitmex order cancel --order-id $STOP_ORDER_ID --yes -o json

# Place new stop
STOP_ORDER_ID=$(bitmex order sell XBTUSD 100 --order-type Stop \
  --stop-px $NEW_STOP --exec-inst ReduceOnly --yes -o json | jq -r '.orderID')

echo "Stop moved to $NEW_STOP"
```

### 5. Monitor for fill

```bash
bitmex ws --auth execution -o json | jq -c 'select(.data[].ordStatus == "Filled")'
```

When the stop fills, the position is closed and the trade is complete.

## Notes

- Never widen a stop — only move it in the direction of the trade.
- On volatile assets, trail by ATR rather than a fixed percentage.
- Always test the full loop on `--testnet` before running with real funds.
