---
name: bitmex-position-risk
version: 1.0.0
description: "Position risk management on bitmex-cli: leverage, margin, funding costs, risk limits, and close procedures."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-types"]
---

# bitmex-position-risk

Active position management prevents liquidation and controls drawdown. Check state before every order.

## Read Current Positions

```bash
# All open positions
bitmex position list -o json 2>/dev/null | \
  jq '[.[] | select(.isOpen == true) | {
    symbol, currentQty, markPrice, avgEntryPrice,
    unrealisedPnl, realisedPnl, liquidationPrice,
    leverage, marginCallPrice
  }]'

# Single symbol
bitmex position list --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {currentQty, markPrice, unrealisedPnl, liquidationPrice, leverage}'
```

## Set Leverage

Adjust leverage on an isolated margin position:

```bash
# Set 10x leverage on XBTUSD
bitmex position leverage XBTUSD 10 -o json 2>/dev/null | \
  jq '{symbol, leverage}'

# Cross leverage (share wallet margin across positions)
bitmex position cross-leverage XBTUSD 5 -o json 2>/dev/null
```

## Isolated vs Cross Margin

```bash
# Switch to isolated margin
bitmex position isolate XBTUSD --enabled -o json 2>/dev/null

# Switch to cross margin (--enabled false)
bitmex position isolate XBTUSD -o json 2>/dev/null
```

## Transfer Margin

Move satoshis into or out of an isolated position to adjust liquidation price:

```bash
# Add 100,000 satoshis to XBTUSD isolated margin
bitmex position transfer-margin XBTUSD 100000 -o json 2>/dev/null

# Remove margin (negative amount)
bitmex position transfer-margin XBTUSD -50000 -o json 2>/dev/null
```

## Check Account Margin

```bash
bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '{
    marginBalance,
    availableMargin,
    unrealisedPnl,
    realisedPnl,
    marginLeverage,
    maintMargin
  }'
```

## Risk Limit

BitMEX uses tiered risk limits. Higher notional exposure requires higher initial margin.

```bash
# Check current risk limit
bitmex position list --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {riskLimit, riskValue}'

# Set new risk limit (in satoshis)
bitmex position risk-limit XBTUSD 20000000000 -o json 2>/dev/null
```

## Funding Cost Monitoring

Funding is charged/paid every 8 hours. A long position pays when funding is positive.

```bash
# Current funding rate and next payment time
bitmex market funding --symbol XBTUSD -o json 2>/dev/null | \
  jq 'last | {fundingRate, fundingInterval, timestamp,
       "cost_per_8h_pct": (.fundingRate * 100 | round / 100)}'

# Estimate funding cost for a position
RATE=$(bitmex market funding --symbol XBTUSD -o json 2>/dev/null | jq -r '.[0].fundingRate')
QTY=$(bitmex position list --symbol XBTUSD -o json 2>/dev/null | jq -r '.[0].currentQty // 0')
echo "Estimated 8h funding cost: $(echo "$QTY * $RATE" | bc -l) USD"
```

## Liquidation Price Awareness

```bash
# How far is current price from liquidation?
bitmex position list --symbol XBTUSD -o json 2>/dev/null | jq '
  .[0] |
  if .markPrice == null or .liquidationPrice == null then
    {symbol, markPrice, liquidationPrice, gap_pct: null}
  else
    ((.markPrice - .liquidationPrice) / .markPrice * 100 | fabs) as $pct |
    {symbol, markPrice, liquidationPrice, gap_pct: (($pct * 100 | round) / 100)}
  end
'
```

## Close Position

```bash
# Market close (immediate, taker fee)
bitmex order close-position XBTUSD -o json 2>/dev/null

# Limit close (preferred — maker rebate)
PRICE=$(bitmex market quote --symbol XBTUSD -o json 2>/dev/null | jq -r 'last | .bidPrice')
QTY=$(bitmex position list --symbol XBTUSD -o json 2>/dev/null | jq -r '.[0].currentQty | if . > 0 then . else (. * -1) end')
SIDE=$(bitmex position list --symbol XBTUSD -o json 2>/dev/null | jq -r 'if .[0].currentQty > 0 then "sell" else "buy" end')
bitmex order "$SIDE" XBTUSD "$QTY" --price "$PRICE" --exec-inst ReduceOnly -o json 2>/dev/null
```
