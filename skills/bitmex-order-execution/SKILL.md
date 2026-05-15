---
name: bitmex-order-execution
version: 1.0.0
description: "Safe order execution flow for bitmex-cli: validate, place, monitor, cancel, and dead man's switch."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-types", "bitmex-error-recovery"]
---

# bitmex-order-execution

Follow this sequence for every order. Never skip validation or the position check.

## Pre-Flight Checks

```bash
# 1. Check current position
bitmex position list --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {symbol, currentQty, markPrice, unrealisedPnl, liquidationPrice}'

# 2. Check available margin
bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '{availableMargin, marginBalance, marginLeverage}'

# 3. Check for stale open orders
bitmex order list --reverse --symbol XBTUSD -o json 2>/dev/null | \
  jq '[.[] | select(.ordStatus == "New" or .ordStatus == "PartiallyFilled") | {orderID, side, price, leavesQty}]'
```

## Step 1: Validate

Always validate before submitting. Validation checks parameters without placing the order.

```bash
bitmex order buy XBTUSD 100 \
  --price 50000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  --validate -o json 2>/dev/null
```

A successful validate response confirms the order would be accepted. A non-zero exit or error envelope means fix parameters before continuing.

## Step 2: Place Order

After validation and user confirmation:

```bash
CL_ORD_ID="strategy-$(date +%s)"

ORDER=$(bitmex order buy XBTUSD 100 \
  --price 50000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  --cl-ord-id "$CL_ORD_ID" \
  -o json 2>/dev/null)

echo "$ORDER" | jq '{orderID, clOrdID, price, ordQty, ordStatus}'
ORDER_ID=$(echo "$ORDER" | jq -r '.orderID')
```

## Step 3: Monitor Fill

Via REST polling:

```bash
bitmex order list --reverse --symbol XBTUSD -o json 2>/dev/null | \
  jq --arg id "$ORDER_ID" '.[] | select(.orderID == $id) | {ordStatus, cumQty, leavesQty, avgPx}'
```

Via WebSocket (preferred — no REST cost):

```bash
bitmex ws --auth execution order -o json 2>/dev/null | \
  jq -c --arg id "$ORDER_ID" '
    .data[]? | select(.orderID == $id or .clOrdID != null) |
    {execType, ordStatus, lastPx, lastQty, cumQty, leavesQty}
  '
```

## Step 4: Amend If Needed

Change price without cancelling and re-placing:

```bash
bitmex order amend --order-id "$ORDER_ID" --price 50200 -o json 2>/dev/null
```

## Step 5: Cancel

```bash
# Cancel specific order
bitmex order cancel --order-id "$ORDER_ID" -o json 2>/dev/null

# Cancel all orders on a symbol
bitmex order cancel-all --symbol XBTUSD -o json 2>/dev/null

# Cancel all orders (emergency)
bitmex order cancel-all -o json 2>/dev/null
```

## Dead Man's Switch

Enable at session start. Cancels all open orders if not renewed within the timeout. Critical for automated strategies.

```bash
# Cancel all open orders if not renewed within 60 seconds
bitmex order cancel-after 60000 -o json 2>/dev/null

# Renew in your heartbeat loop
while true; do
  bitmex order cancel-after 60000 -o json 2>/dev/null > /dev/null
  sleep 30
done &
DMS_PID=$!

# On exit, cancel the DMS and clean up
trap "kill $DMS_PID; bitmex order cancel-all -o json 2>/dev/null" EXIT
```

## Close Position

```bash
# Market close (taker fee, immediate)
bitmex order close-position XBTUSD -o json 2>/dev/null

# Limit close (maker rebate, may not fill immediately)
QTY=$(bitmex position list --symbol XBTUSD -o json 2>/dev/null | jq '.[0].currentQty // 0')
if [ "$QTY" -gt 0 ]; then
  PRICE=$(bitmex market orderbook XBTUSD --depth 1 -o json 2>/dev/null | jq '[.[] | select(.side == "Buy")] | first | .price')
  bitmex order sell XBTUSD "$QTY" --price "$PRICE" --exec-inst ReduceOnly -o json 2>/dev/null
fi
```

## Sell Order

```bash
bitmex order sell XBTUSD 100 \
  --price 52000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  --validate -o json 2>/dev/null

bitmex order sell XBTUSD 100 \
  --price 52000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null
```
