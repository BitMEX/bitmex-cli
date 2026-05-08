---
name: bitmex-stop-take-profit
version: 1.0.0
description: "Stop-loss and take-profit orders on bitmex-cli: Stop, StopLimit, bracket orders, ReduceOnly, and verification."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-types", "bitmex-order-execution"]
---

# bitmex-stop-take-profit

Protective orders must be placed immediately after entering a position. Always use `ReduceOnly` to prevent the stop from flipping your position.

## Stop Market (Fast Exit)

Triggers a market order when `stopPx` is reached. Guaranteed to execute but at market price.

```bash
# Stop loss for a long position: sell if price drops to 48000
bitmex order sell XBTUSD 100 \
  --order-type Stop \
  --stop-px 48000 \
  --exec-inst ReduceOnly,MarkPrice \
  --validate -o json 2>/dev/null

bitmex order sell XBTUSD 100 \
  --order-type Stop \
  --stop-px 48000 \
  --exec-inst ReduceOnly,MarkPrice \
  -o json 2>/dev/null
```

For a short position (stop above entry):

```bash
bitmex order buy XBTUSD 100 \
  --order-type Stop \
  --stop-px 52000 \
  --exec-inst ReduceOnly,MarkPrice \
  -o json 2>/dev/null
```

## Stop Limit (Controlled Slippage)

Places a limit at `--price` when `stopPx` is touched. May not fill in fast markets.

```bash
# Stop limit: trigger at 48000, limit at 47900
bitmex order sell XBTUSD 100 \
  --order-type StopLimit \
  --stop-px 48000 \
  --price 47900 \
  --exec-inst ReduceOnly,MarkPrice \
  --validate -o json 2>/dev/null
```

## Take Profit

Take profit uses the opposite-side stop order type. For a long, place a sell above entry:

```bash
# Take profit: sell limit when price reaches 55000
bitmex order sell XBTUSD 100 \
  --order-type LimitIfTouched \
  --stop-px 55000 \
  --price 55000 \
  --exec-inst ReduceOnly \
  --validate -o json 2>/dev/null
```

## Bracket Order (Entry + Stop + Take Profit)

Place all three legs after entry fills:

```bash
ENTRY_QTY=100
ENTRY_PRICE=50000
STOP_PRICE=48000
TP_PRICE=55000

# Confirm entry is filled
bitmex ws --auth execution -o json 2>/dev/null &
WS_PID=$!

# Entry order
bitmex order buy XBTUSD $ENTRY_QTY \
  --price $ENTRY_PRICE \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null

# After fill confirmed, place stop and TP
bitmex order sell XBTUSD $ENTRY_QTY \
  --order-type Stop \
  --stop-px $STOP_PRICE \
  --exec-inst ReduceOnly,MarkPrice \
  -o json 2>/dev/null

bitmex order sell XBTUSD $ENTRY_QTY \
  --order-type LimitIfTouched \
  --stop-px $TP_PRICE \
  --price $TP_PRICE \
  --exec-inst ReduceOnly \
  -o json 2>/dev/null

kill $WS_PID
```

## Verify Protective Orders Are Active

```bash
bitmex order list --reverse --symbol XBTUSD -o json 2>/dev/null | \
  jq '[.[] | select(.ordStatus == "New") | {
    orderID, side, ordType, stopPx, price, execInst, ordStatus
  }]'
```

If you see neither a stop nor a take-profit in the open orders list after entering a position, place them immediately.

## Manual Trailing Stop

BitMEX does not support native trailing stops. Implement via polling:

```bash
TRAIL_OFFSET=500  # trail 500 points below peak
PEAK=50000
SYMBOL=XBTUSD
STOP_ORDER_ID=""

while true; do
  LAST=$(bitmex market instrument --symbol $SYMBOL -o json 2>/dev/null | jq -r '.[0].lastPrice')
  if [ "$(echo "$LAST > $PEAK" | bc -l)" = "1" ]; then
    PEAK=$LAST
    NEW_STOP=$(echo "$PEAK - $TRAIL_OFFSET" | bc)
    echo "New peak $PEAK → trail stop at $NEW_STOP"
    if [ -n "$STOP_ORDER_ID" ]; then
      bitmex order amend --order-id "$STOP_ORDER_ID" --stop-px "$NEW_STOP" -o json 2>/dev/null
    fi
  fi
  sleep 10
done
```

## Cancel Protective Orders on Close

When closing a position, cancel its protective orders:

```bash
bitmex order cancel-all --symbol XBTUSD -o json 2>/dev/null
bitmex order close-position XBTUSD -o json 2>/dev/null
```
