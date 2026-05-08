---
name: bitmex-rebalancing
version: 1.0.0
description: "Rebalancing positions on bitmex-cli: check allocations, calculate deltas, place orders, and validate on testnet."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-execution", "bitmex-position-risk"]
---

# bitmex-rebalancing

Rebalancing adjusts current positions toward target allocations. On BitMEX, this means adding to or reducing perpetual/futures positions, not converting between currencies.

## Step 1: Read Current State

```bash
# All open positions with mark-to-market value
bitmex position list -o json 2>/dev/null | \
  jq '[.[] | select(.isOpen == true) | {
    symbol,
    currentQty,
    markPrice,
    notional: (.currentQty * .markPrice),
    unrealisedPnl
  }]'

# Total wallet balance
bitmex wallet balance --currency XBt -o json 2>/dev/null | \
  jq '{currency, amount}'

# Account margin summary
bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '{marginBalance, availableMargin, unrealisedPnl}'
```

## Step 2: Define Target Allocation

Example: 60% long XBTUSD, 40% long ETHUSD by notional.

```bash
MARGIN=$(bitmex account margin --currency XBt -o json 2>/dev/null | jq -r '.marginBalance')
echo "Total margin (satoshis): $MARGIN"

# Target notional per symbol (in USD equivalent)
# These are user-defined constants
TARGET_XBTUSD=6000   # contracts
TARGET_ETHUSD=4000   # contracts
MIN_REBAL_QTY=50     # ignore deltas smaller than this
```

## Step 3: Calculate Deltas

```bash
CURRENT_XBT=$(bitmex position list --symbol XBTUSD -o json 2>/dev/null | \
  jq -r '.[0].currentQty // 0')
CURRENT_ETH=$(bitmex position list --symbol ETHUSD -o json 2>/dev/null | \
  jq -r '.[0].currentQty // 0')

DELTA_XBT=$((TARGET_XBTUSD - CURRENT_XBT))
DELTA_ETH=$((TARGET_ETHUSD - CURRENT_ETH))

echo "XBTUSD: current=$CURRENT_XBT target=$TARGET_XBTUSD delta=$DELTA_XBT"
echo "ETHUSD: current=$CURRENT_ETH target=$TARGET_ETHUSD delta=$DELTA_ETH"
```

## Step 4: Validate on Testnet First

```bash
FLAG="--testnet"
# Validate XBTUSD rebalance
if [ "${DELTA_XBT#-}" -ge "$MIN_REBAL_QTY" ]; then
  if [ "$DELTA_XBT" -gt 0 ]; then
    SIDE="buy"; QTY=$DELTA_XBT
  else
    SIDE="sell"; QTY=${DELTA_XBT#-}
  fi
  OB_SIDE=$([ "$SIDE" = "buy" ] && echo "Buy" || echo "Sell")
  PRICE=$(bitmex $FLAG market orderbook XBTUSD --depth 1 -o json 2>/dev/null | \
    jq --arg s "$OB_SIDE" '[.[] | select(.side == $s)] | .[0].price')
  bitmex $FLAG order $SIDE XBTUSD $QTY --price "$PRICE" \
    --order-type Limit --validate -o json 2>/dev/null | \
    jq '{side, ordQty, price, ordType}'
fi
```

## Step 5: Execute (After Approval)

```bash
place_rebal_order() {
  local SYMBOL=$1
  local DELTA=$2

  ABS_DELTA=${DELTA#-}
  if [ "$ABS_DELTA" -lt "$MIN_REBAL_QTY" ]; then
    echo "$SYMBOL delta $DELTA below threshold, skipping"
    return 0
  fi

  if [ "$DELTA" -gt 0 ]; then
    SIDE="buy"
    PRICE=$(bitmex market orderbook $SYMBOL --depth 1 -o json 2>/dev/null | jq '[.[] | select(.side == "Buy")] | .[0].price')
  else
    SIDE="sell"
    PRICE=$(bitmex market orderbook $SYMBOL --depth 1 -o json 2>/dev/null | jq '[.[] | select(.side == "Sell")] | .[0].price')
  fi

  echo "Rebalancing $SYMBOL: $SIDE $ABS_DELTA @ $PRICE"
  bitmex order $SIDE $SYMBOL $ABS_DELTA \
    --price "$PRICE" \
    --order-type Limit \
    --exec-inst ParticipateDoNotInitiate \
    -o json 2>/dev/null | jq '{orderID, side, price, ordQty}'
}

# Execute after user confirmation
place_rebal_order "XBTUSD" "$DELTA_XBT"
place_rebal_order "ETHUSD" "$DELTA_ETH"
```

## Step 6: Verify Result

```bash
sleep 10
bitmex position list -o json 2>/dev/null | \
  jq '[.[] | select(.isOpen == true) | {symbol, currentQty, unrealisedPnl}]'
```

## Min Trade Threshold

Always enforce `MIN_REBAL_QTY` to avoid excessive small orders that erode returns with fees. A good threshold is 1–5% of target position size.
