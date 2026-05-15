---
name: bitmex-twap-execution
version: 1.0.0
description: "TWAP execution for large orders on bitmex-cli: slicing, rate-limit awareness, fill tracking, and abort handling."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-execution", "bitmex-rate-limits", "bitmex-fee-optimization"]
---

# bitmex-twap-execution

Time-Weighted Average Price (TWAP) splits a large order into smaller slices executed at fixed intervals, reducing market impact.

## Strategy Parameters

| Parameter | Example | Description |
|---|---|---|
| SYMBOL | XBTUSD | Contract |
| TOTAL_QTY | 10000 | Total contracts to execute |
| SLICES | 20 | Number of child orders |
| INTERVAL_SECONDS | 300 | 5 minutes between slices |
| SIDE | buy | buy or sell |
| ORDER_TYPE | Limit | Limit for maker, Market for guaranteed fill |

With 20 slices at 5-min intervals, total execution takes ~100 minutes.

## Rate Limit Check

20 slices × ~3 requests each (place + check + heartbeat) = 60 requests. Well within 300 req/5min budget.

```bash
TOTAL_QTY=10000
SLICES=20
QTY_PER_SLICE=$(echo "$TOTAL_QTY / $SLICES" | bc)
echo "Qty per slice: $QTY_PER_SLICE"
```

## TWAP Execution Loop

```bash
SYMBOL="XBTUSD"
SIDE="buy"
TOTAL_QTY=10000
SLICES=20
INTERVAL=300  # seconds
QTY_PER_SLICE=$((TOTAL_QTY / SLICES))
FILLED=0
FAILED=0
SESSION_ID="twap-$(date +%Y%m%d%H%M%S)"

echo "Starting TWAP: $TOTAL_QTY $SIDE $SYMBOL in $SLICES slices"

for i in $(seq 1 $SLICES); do
  echo "Slice $i/$SLICES: placing $QTY_PER_SLICE $SIDE"

  # Get current best price for limit order
  if [ "$SIDE" = "buy" ]; then
    PRICE=$(bitmex market orderbook $SYMBOL --depth 1 -o json 2>/dev/null | jq '[.[] | select(.side == "Buy")] | .[0].price')
  else
    PRICE=$(bitmex market orderbook $SYMBOL --depth 1 -o json 2>/dev/null | jq '[.[] | select(.side == "Sell")] | .[0].price')
  fi

  CL_ORD_ID="${SESSION_ID}-${i}"
  RESULT=$(bitmex order $SIDE $SYMBOL $QTY_PER_SLICE \
    --price "$PRICE" \
    --order-type Limit \
    --exec-inst ParticipateDoNotInitiate \
    --cl-ord-id "$CL_ORD_ID" \
    -o json 2>/dev/null)

  if [ $? -eq 0 ]; then
    FILLED=$((FILLED + QTY_PER_SLICE))
    echo "Slice $i placed: $(echo "$RESULT" | jq -c '{orderID, price, ordQty}')"
  else
    FAILED=$((FAILED + 1))
    echo "Slice $i failed: $(echo "$RESULT" | jq -r '.message')"
  fi

  # Don't sleep after last slice
  if [ $i -lt $SLICES ]; then
    sleep $INTERVAL
  fi
done

echo "TWAP complete. Placed: $FILLED / $TOTAL_QTY. Failed slices: $FAILED"
```

## Track Average Fill Price

```bash
# After session, calculate average fill
bitmex execution trade-history --symbol $SYMBOL --reverse --count 100 -o json 2>/dev/null | \
  jq --arg session "$SESSION_ID" '
    [.[] | select(.clOrdID != null and (.clOrdID | startswith($session))) | select(.execType == "Trade")] |
    {
      slices_filled: length,
      total_qty: (map(.lastQty) | add),
      avg_fill_px: (
        (map(.lastQty * .lastPx) | add) /
        (map(.lastQty) | add)
        | round / 100
      ),
      total_commission: (map(.commission // 0) | add)
    }
  '
```

## Abort TWAP

Cancel all unfilled slices and stop the loop:

```bash
# Cancel all open orders for this symbol
bitmex order cancel-all --symbol $SYMBOL -o json 2>/dev/null | \
  jq '{cancelled: (. | length)}'

# Check what was filled so far
bitmex position list --symbol $SYMBOL -o json 2>/dev/null | \
  jq '.[0] | {currentQty, avgEntryPrice, unrealisedPnl}'
```

## Partial Fill Handling

If a limit slice only partially fills before the next interval:

```bash
# Check status of the last slice by clOrdID
LAST_CL_ID="${SESSION_ID}-${i}"
bitmex order list --reverse --symbol $SYMBOL -o json 2>/dev/null | \
  jq --arg id "$LAST_CL_ID" '
    [.[] | select(.clOrdID == $id)] |
    .[0] | {ordStatus, cumQty, leavesQty}
  '

# Cancel unfilled remainder and proceed with next slice
bitmex order cancel --order-id "$ORDER_ID" -o json 2>/dev/null
```

## Testnet Validation

```bash
# Run full TWAP on testnet with small quantities
export BITMEX_API_KEY="testnet-key"
export BITMEX_API_SECRET="testnet-secret"
SYMBOL="XBTUSD" SLICES=5 TOTAL_QTY=500 INTERVAL=10  # fast test
```
