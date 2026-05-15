---
name: bitmex-recipe-futures-hedge-spot
description: Hedge a long position with a short perpetual.
---

# Futures Hedge for Spot Position

If you hold spot BTC and want to neutralise downside risk without selling, open a short position on the XBTUSD perpetual sized to match your spot exposure.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` and `bc` installed.
- Know your spot BTC quantity (e.g. `SPOT_BTC=0.5`).

## Steps

### 1. Assess the exposure to hedge

```bash
SPOT_BTC=0.5
MARK=$(bitmex market instrument --symbol XBTUSD -o json | jq '.[0].markPrice')
HEDGE_QTY=$(echo "scale=0; $SPOT_BTC * $MARK / 1" | bc)
echo "Need to short $HEDGE_QTY contracts of XBTUSD to hedge $SPOT_BTC BTC"
```

XBTUSD is inverse: 1 contract = 1 USD of BTC exposure at current price.

### 2. Check existing positions

```bash
bitmex position list -o json \
  | jq '[.[] | {symbol, currentQty, unrealisedPnl, liquidationPrice}]'
```

### 3. Open the short hedge (validate first)

```bash
bitmex order sell XBTUSD $HEDGE_QTY --order-type Limit \
  --price $(echo "$MARK * 0.9999" | bc | xargs printf "%.0f") \
  --validate -o json
```

If validate output is correct:

```bash
bitmex order sell XBTUSD $HEDGE_QTY --order-type Limit \
  --price $(echo "$MARK * 0.9999" | bc | xargs printf "%.0f") \
  --yes -o json | jq '{orderID, side, price, orderQty}'
```

### 4. Verify net delta

After the hedge fills, positions should offset your spot exposure:

```bash
bitmex position list -o json \
  | jq '[.[] | select(.symbol == "XBTUSD") | {currentQty, unrealisedPnl}]'
```

If spot = 0.5 BTC long and hedge = 50000 USD short at 100 000, the net delta is approximately zero.

### 5. Adjust hedge as spot moves

Recalculate `HEDGE_QTY` when BTC price moves more than 5%, then add or remove contracts:

```bash
# Add more short if price dropped (contract value fell)
bitmex order sell XBTUSD <adjustment_qty> --order-type Market --yes -o json
```

## Notes

- Use limit orders close to mark price to avoid taker fees where possible.
- This hedge costs nothing in margin if cross-margin is enabled, but does consume initial margin in isolated mode.
- Unwind the hedge before taking delivery of spot if funding rates become too costly.
- This recipe targets XBTUSD (inverse perpetual) only. For linear instruments (e.g. XBTUSDT), the contract value formula and P&L denomination differ — check `lotSize` and `multiplier` from `bitmex market instrument --symbol <SYMBOL>` before sizing a linear hedge.
