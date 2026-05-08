---
name: bitmex-basis-trading
version: 1.0.0
description: "Delta-neutral basis trading between BitMEX perpetuals and fixed-date futures: entry, monitoring, and exit."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-execution", "bitmex-position-risk"]
---

# bitmex-basis-trading

Basis trading captures the spread between a perpetual and a fixed-date future on the same underlying. The spread converges to zero at expiry, yielding a predictable return when entered at a wide basis.

Always validate on testnet first: prefix commands with `bitmex --testnet` until the strategy behaves as expected.

## What Is the Basis?

```
basis = (futures_price - perp_price) / perp_price * 100
```

A positive basis (futures above perp) typically reflects cost-of-carry. The strategy: long perp + short future (or the reverse), targeting convergence.

## Step 1: Fetch Both Instrument Prices

```bash
# Perpetual
PERP=$(bitmex market instrument --symbol XBTUSD -o json 2>/dev/null | \
  jq -r '.[0].lastPrice')

# Nearest quarterly future (example: XBTM25)
FUT=$(bitmex market instrument --symbol XBTM25 -o json 2>/dev/null | \
  jq -r '.[0].lastPrice')

echo "Perp: $PERP | Future: $FUT"
```

## Step 2: Calculate Basis

```bash
BASIS=$(echo "scale=6; ($FUT - $PERP) / $PERP * 100" | bc -l)
echo "Basis: ${BASIS}%"

# Days to expiry for annualized estimate
SETTLE=$(bitmex market instrument --symbol XBTM25 -o json 2>/dev/null | \
  jq -r '.[0].settle')
echo "Settlement: $SETTLE"
```

## Step 3: List Available Futures

```bash
# All active fixed-date futures
bitmex market instrument --active -o json 2>/dev/null | \
  jq '[.[] | select(.typ == "FFCCSX") | {symbol, lastPrice, settle, openInterest}]'
```

## Step 4: Enter Delta-Neutral Position

Positive basis (futures expensive): short the future, long the perp.

```bash
QTY=1000

# Validate both legs
bitmex order buy XBTUSD $QTY --price "$PERP" \
  --order-type Limit --exec-inst ParticipateDoNotInitiate \
  --validate -o json 2>/dev/null

bitmex order sell XBTM25 $QTY --price "$FUT" \
  --order-type Limit --exec-inst ParticipateDoNotInitiate \
  --validate -o json 2>/dev/null

# Execute after user confirmation
bitmex order buy XBTUSD $QTY --price "$PERP" \
  --order-type Limit --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null

bitmex order sell XBTM25 $QTY --price "$FUT" \
  --order-type Limit --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null
```

## Step 5: Monitor Convergence

```bash
# Check current basis
PERP_NOW=$(bitmex market instrument --symbol XBTUSD -o json 2>/dev/null | jq -r '.[0].lastPrice')
FUT_NOW=$(bitmex market instrument --symbol XBTM25 -o json 2>/dev/null | jq -r '.[0].lastPrice')
BASIS_NOW=$(echo "scale=6; ($FUT_NOW - $PERP_NOW) / $PERP_NOW * 100" | bc -l)
echo "Current basis: ${BASIS_NOW}%"

# Check both legs' PnL
bitmex position list -o json 2>/dev/null | \
  jq '[.[] | select(.symbol == "XBTUSD" or .symbol == "XBTM25") | {symbol, currentQty, unrealisedPnl, realisedPnl}]'
```

## Step 6: Exit When Basis Closes

Exit both legs simultaneously when basis is near zero or at expiry:

```bash
TARGET_BASIS=0.1  # exit if basis below 0.1%
if [ "$(echo "${BASIS_NOW#-} < $TARGET_BASIS" | bc -l)" = "1" ]; then
  echo "Basis converged: ${BASIS_NOW}% — exiting both legs"

  # Close perp leg
  bitmex order close-position XBTUSD -o json 2>/dev/null

  # Close futures leg
  bitmex order close-position XBTM25 -o json 2>/dev/null
fi
```

## Carry Estimate

```bash
# Annualized carry for a given basis and days to expiry
DAYS=90
echo "scale=4; $BASIS / $DAYS * 365" | bc -l
# e.g., 2% basis over 90 days = ~8.1% annualized
```

## Risk Considerations

- Execution risk: both legs must fill; partial fill leaves a naked directional position
- Margin risk: each leg consumes margin independently
- Roll risk: if the future expires before exit, roll the position
- Basis can widen before converging — maintain margin buffer
- Funding risk: the perpetual leg accrues funding every 8 hours; in positive-funding environments the long perp leg pays, eroding carry — model funding into your breakeven calculation
- Instrument scope: this guide uses inverse pairs (XBTUSD/XBTM25); linear-linear pairs (e.g. XBTUSDT/XBTUSDTM26) work similarly provided lot sizes and multipliers match, but P&L is denominated in USDT rather than XBT
- Spot-future basis: the classic version uses spot BTC (off-exchange) vs a fixed-date future rather than a perpetual; this eliminates funding risk but requires managing spot custody separately and is not covered here
