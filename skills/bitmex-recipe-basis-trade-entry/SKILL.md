---
name: bitmex-recipe-basis-trade-entry
description: Enter a perp-futures basis trade when premium exceeds threshold.
---

# Basis Trade Entry

A basis trade is delta-neutral: long the cheaper leg (usually the perpetual) and short the more expensive leg (the fixed-date future) when the premium between them exceeds the cost of carry. The trade profits as the basis converges to zero at expiry.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` and `bc` installed.
- Know the target symbols: e.g. `XBTUSD` (perp) and `XBTM25` (fixed-date future).

## Steps

### 1. Get perpetual mark price

```bash
PERP_PRICE=$(bitmex market instrument --symbol XBTUSD -o json \
  | jq '.[0].markPrice')
echo "Perp: $PERP_PRICE"
```

### 2. Get futures mark price

```bash
FUT_PRICE=$(bitmex market instrument --symbol XBTM25 -o json \
  | jq '.[0].markPrice')
echo "Futures: $FUT_PRICE"
```

### 3. Calculate basis

```bash
BASIS=$(echo "scale=6; ($FUT_PRICE - $PERP_PRICE) / $PERP_PRICE * 100" | bc)
echo "Basis: ${BASIS}%"
```

### 4. Check entry threshold

Only enter if basis exceeds 1% (adjust to your cost model):

```bash
THRESHOLD=1.0
if (( $(echo "$BASIS > $THRESHOLD" | bc -l) )); then
  echo "Basis ${BASIS}% > ${THRESHOLD}% threshold — entering trade"
else
  echo "Basis too thin, skipping"
  exit 0
fi
```

### 5. Enter both legs (validate first)

```bash
QTY=100

# Long perp
bitmex order buy XBTUSD $QTY --order-type Limit \
  --price $PERP_PRICE --validate -o json

# Short future
bitmex order sell XBTM25 $QTY --order-type Limit \
  --price $FUT_PRICE --validate -o json
```

If validate output looks correct, submit both with `--yes` instead of `--validate`.

### 6. Monitor convergence

```bash
bitmex position list -o json \
  | jq '[.[] | select(.symbol == "XBTUSD" or .symbol == "XBTM25")
         | {symbol, currentQty, avgEntryPrice, unrealisedPnl}]'
```

Exit both legs when basis narrows to under 0.1% or at futures expiry.

## Notes

- Size both legs equally in USD notional to maintain delta neutrality.
- Account for maker/taker fees on both entry and exit when calculating the net carry.
- The perpetual leg exposes you to funding rate volatility; net carry = basis − cumulative funding over hold period. Check `bitmex market funding --symbol XBTUSD` before entering.
- For carry estimates, exit logic, and roll procedures, see the `bitmex-basis-trading` skill.
