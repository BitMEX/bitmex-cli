---
name: bitmex-recipe-weekly-rebalance
description: Weekly rebalance to maintain target position allocations.
---

# Weekly Portfolio Rebalance

Each week, compare your actual position sizes against target allocations and place orders to bring them back in line.

Always validate on testnet first: prefix commands with `bitmex --testnet` until the strategy behaves as expected.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` and `bc` installed.
- Define target allocations as a fraction of total wallet balance.

## Steps

### 1. Get current positions

```bash
bitmex position list -o json \
  | jq '[.[] | {symbol, currentQty, markValue, unrealisedPnl}]'
```

### 2. Get total wallet balance

```bash
WALLET=$(bitmex wallet balance -o json | jq '.amount')
echo "Total wallet: $WALLET satoshis"
```

### 3. Calculate target sizes

Example: 60% XBTUSD, 25% ETHUSD, 15% SOLUSDT of total notional:

```bash
XBTUSD_MARK=$(bitmex market instrument --symbol XBTUSD -o json | jq '.[0].markPrice')
ETHUSD_MARK=$(bitmex market instrument --symbol ETHUSD -o json | jq '.[0].markPrice')

# Convert wallet from satoshis to USD equivalent
WALLET_USD=$(echo "scale=2; $WALLET / 100000000 * $XBTUSD_MARK" | bc)

TARGET_XBTUSD=$(echo "scale=0; $WALLET_USD * 0.60 / 1" | bc)
TARGET_ETHUSD=$(echo "scale=0; $WALLET_USD * 0.25 / 1" | bc)
```

### 4. Calculate drift and rebalance orders

```bash
CURRENT_XBTUSD=$(bitmex position list -o json \
  | jq '[.[] | select(.symbol == "XBTUSD")] | .[0].currentQty // 0')

DELTA=$(echo "$TARGET_XBTUSD - $CURRENT_XBTUSD" | bc)
echo "XBTUSD drift: $DELTA contracts"
```

### 5. Place rebalance orders (validate first, then submit)

```bash
# If delta is positive, buy the difference; if negative, sell
if (( $(echo "$DELTA > 0" | bc -l) )); then
  bitmex order buy XBTUSD $(echo "$DELTA" | xargs printf "%.0f") \
    --order-type Limit --price $(echo "$XBTUSD_MARK * 1.0001" | bc | xargs printf "%.0f") \
    --validate -o json
elif (( $(echo "$DELTA < 0" | bc -l) )); then
  bitmex order sell XBTUSD $(echo "-1 * $DELTA" | bc | xargs printf "%.0f") \
    --order-type Limit --price $(echo "$XBTUSD_MARK * 0.9999" | bc | xargs printf "%.0f") \
    --validate -o json
fi
```

Replace `--validate` with `--yes` after reviewing the output.

### 6. Verify after rebalance

```bash
bitmex position list -o json \
  | jq '[.[] | {symbol, currentQty, markValue}]'
```

## Notes

- Use a 5% drift threshold to avoid over-trading: only rebalance a leg if drift exceeds 5% of target.
- Schedule for low-volatility hours (e.g. Sunday 04:00 UTC) to reduce slippage.
