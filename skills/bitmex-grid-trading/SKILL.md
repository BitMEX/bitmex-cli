---
name: bitmex-grid-trading
version: 1.0.0
description: "Grid trading on BitMEX perpetuals: setup, order placement, fill monitoring, replacement, and shutdown."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-execution", "bitmex-ws-streaming", "bitmex-liquidation-guard"]
---

# bitmex-grid-trading

A grid strategy places limit orders at fixed intervals above and below the current price, profiting from oscillation within a range.

**Always test with `--testnet` first. A trending market will fill all orders on one side, creating a large directional position.**

## Parameters

| Parameter | Example | Description |
|---|---|---|
| SYMBOL | XBTUSD | Perpetual contract |
| GRID_LOW | 48000 | Lower bound of grid |
| GRID_HIGH | 52000 | Upper bound |
| GRID_COUNT | 8 | Number of levels (buy + sell) |
| QTY_PER_LEVEL | 100 | Contracts per order |
| MAX_POSITION | 800 | Hard cap: QTY × (GRID_COUNT/2) |

## Testnet Setup

```bash
export BITMEX_API_KEY="testnet-key"
export BITMEX_API_SECRET="testnet-secret"
FLAG="--testnet"

# Verify connectivity
bitmex $FLAG market instrument --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {symbol, lastPrice}'
```

## Calculate Grid Levels

```bash
GRID_LOW=48000
GRID_HIGH=52000
GRID_COUNT=8
QTY=100
SYMBOL="XBTUSD"

STEP=$(echo "scale=0; ($GRID_HIGH - $GRID_LOW) / $GRID_COUNT" | bc)
echo "Grid step: $STEP"

# Generate levels
for i in $(seq 0 $GRID_COUNT); do
  LEVEL=$(echo "$GRID_LOW + $STEP * $i" | bc)
  echo "Level $i: $LEVEL"
done
```

## Place Grid Orders

```bash
CURRENT=$(bitmex market instrument --symbol $SYMBOL -o json 2>/dev/null | jq -r '.[0].lastPrice')

for i in $(seq 0 $((GRID_COUNT - 1))); do
  LEVEL=$(echo "$GRID_LOW + $STEP * $i" | bc)

  if [ "$(echo "$LEVEL < $CURRENT" | bc -l)" = "1" ]; then
    # Below current price: buy order
    bitmex order buy $SYMBOL $QTY \
      --price "$LEVEL" \
      --order-type Limit \
      --exec-inst ParticipateDoNotInitiate \
      -o json 2>/dev/null | jq -c '{orderID, side, price, ordQty}'
  else
    # Above current price: sell order
    bitmex order sell $SYMBOL $QTY \
      --price "$LEVEL" \
      --order-type Limit \
      --exec-inst ParticipateDoNotInitiate \
      -o json 2>/dev/null | jq -c '{orderID, side, price, ordQty}'
  fi
  sleep 0.5  # stay within rate limit
done
```

## Monitor Fills via WebSocket

```bash
bitmex ws --auth execution -o json 2>/dev/null | \
  jq -c '.data[]? | select(.execType == "Trade") | {
    symbol, side, lastPx, lastQty, ordStatus, clOrdID
  }'
```

## Replace Filled Orders

When a buy fills, place a sell one step above. When a sell fills, place a buy one step below.

```bash
bitmex ws --auth execution -o json 2>/dev/null | while IFS= read -r line; do
  EXEC_TYPE=$(echo "$line" | jq -r '.data[0].execType // empty')
  if [ "$EXEC_TYPE" = "Trade" ]; then
    SIDE=$(echo "$line" | jq -r '.data[0].side')
    FILL_PX=$(echo "$line" | jq -r '.data[0].lastPx')

    if [ "$SIDE" = "Buy" ]; then
      NEW_PRICE=$(echo "$FILL_PX + $STEP" | bc)
      NEW_SIDE="sell"
    else
      NEW_PRICE=$(echo "$FILL_PX - $STEP" | bc)
      NEW_SIDE="buy"
    fi

    # Check position cap before replacing
    POS=$(bitmex position list --symbol $SYMBOL -o json 2>/dev/null | jq -r '.[0].currentQty // 0')
    if [ "$(echo "${POS#-} < $MAX_POSITION" | bc -l)" = "1" ]; then
      bitmex order "$NEW_SIDE" $SYMBOL $QTY \
        --price "$NEW_PRICE" \
        --order-type Limit \
        --exec-inst ParticipateDoNotInitiate \
        -o json 2>/dev/null
    fi
  fi
done
```

## Shutdown

Cancel all open orders cleanly:

```bash
bitmex order cancel-all --symbol $SYMBOL -o json 2>/dev/null | \
  jq '{cancelled: (. | length)}'

# Optionally close any residual position
bitmex order close-position $SYMBOL -o json 2>/dev/null
```

## Risk Warning

Grid trading requires the price to stay within the defined range. If the market trends strongly in one direction, all orders on one side will fill, resulting in a large unhedged position. Use the position cap (`MAX_POSITION`) and stop loss checks from `bitmex-liquidation-guard`.
