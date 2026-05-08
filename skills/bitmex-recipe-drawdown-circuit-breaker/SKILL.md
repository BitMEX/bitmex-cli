---
name: bitmex-recipe-drawdown-circuit-breaker
description: Halt trading when portfolio drawdown exceeds threshold.
---

# Drawdown Circuit Breaker

Monitor margin balance against a high-water mark and automatically flatten all positions when drawdown exceeds your configured threshold.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` and `bc` installed.
- Choose a drawdown threshold, e.g. `MAX_DRAWDOWN_PCT=5`.

## Steps

### 1. Record the high-water mark

Capture margin balance at the start of each session or daily. `marginBalance` (not `walletBalance`) is used here because it includes `unrealisedPnl`, so a large adverse price move will trigger the breaker even before positions are closed.

```bash
HIGH_WATER=$(bitmex account margin --currency XBt -o json | jq '.marginBalance')
echo "$HIGH_WATER" > /tmp/bitmex_hwm.txt
```

On subsequent checks, load it:

```bash
HIGH_WATER=$(cat /tmp/bitmex_hwm.txt)
```

### 2. Get current margin balance

```bash
CURRENT=$(bitmex account margin --currency XBt -o json | jq '.marginBalance')
```

> **Note:** This circuit breaker covers XBt-denominated positions only. USDT positions require a separate check using `--currency USDt`.

### 3. Calculate drawdown percentage

```bash
DRAWDOWN=$(echo "scale=4; ($HIGH_WATER - $CURRENT) / $HIGH_WATER * 100" | bc)
echo "Current drawdown: ${DRAWDOWN}%"
```

### 4. Trip the breaker

```bash
MAX_DRAWDOWN_PCT=5

if (( $(echo "$DRAWDOWN > $MAX_DRAWDOWN_PCT" | bc -l) )); then
  echo "CIRCUIT BREAKER TRIPPED: ${DRAWDOWN}% drawdown exceeds ${MAX_DRAWDOWN_PCT}%"

  # Cancel all open orders
  bitmex order cancel-all --yes -o json

  # Close all open positions
  bitmex position list -o json \
    | jq -r '[.[] | select(.currentQty != 0) | .symbol] | .[]' \
    | while read -r SYM; do
        bitmex order close-position "$SYM" -o json
      done

  echo "All positions closed. Manual review required before resuming."
  exit 1
fi
```

### 5. Update high-water mark if balance is at a new peak

```bash
if (( $(echo "$CURRENT > $HIGH_WATER" | bc -l) )); then
  echo "$CURRENT" > /tmp/bitmex_hwm.txt
fi
```

## Notes

- Run this check at the top of every order-placement loop.
- Use a 5% threshold for conservative bots; aggressive strategies may use 10–15%.
- Always test the full breaker flow on `--testnet` before connecting to live funds.
