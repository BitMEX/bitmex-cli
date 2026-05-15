---
name: bitmex-recipe-testnet-strategy-backtest
description: Validate a strategy across multiple testnet sessions before going live.
---

# Testnet Strategy Validation

Run your strategy on the BitMEX testnet for multiple independent sessions, track performance statistics, and only promote to live after passing consistency criteria.

## Prerequisites

- `jq` and `bc` installed.
- No real API credentials needed — testnet uses separate keys from `testnet.bitmex.com`.
- Set `BITMEX_API_KEY` and `BITMEX_API_SECRET` to your testnet credentials.

## Steps

### 1. Run one strategy session on testnet

Execute your strategy with the `--testnet` flag throughout:

```bash
# Example: place entry order
bitmex --testnet order buy XBTUSD 100 --order-type Limit \
  --price <entry_price> --yes -o json | jq '{orderID, price, ordStatus}'

# Monitor fills
bitmex --testnet ws --auth execution -o json | jq -c '.data[]? | {side, price, orderQty, ordStatus}'
```

### 2. Record session results

After each session, extract the trade history:

```bash
SESSION_DATE=$(date -u +%Y%m%dT%H%M%SZ)

bitmex --testnet execution trade-history --count 100 -o json \
  > /tmp/session_${SESSION_DATE}.json

jq '{
  trades: length,
  realisedPnl: ([.[].realisedPnl] | add // 0),
  totalFees: ([.[].commission] | add // 0),
  symbols: ([.[].symbol] | unique)
}' /tmp/session_${SESSION_DATE}.json
```

### 3. Calculate session statistics

```bash
jq '
  . as $trades |
  ($trades | length) as $n |
  ([.[] | select(.realisedPnl > 0)] | length) as $wins |
  {
    totalTrades: $n,
    winRate: (if $n > 0 then ($wins / $n * 100) else 0 end),
    avgPnl: ([.[].realisedPnl] | add // 0 | . / (if $n > 0 then $n else 1 end)),
    maxLoss: ([.[].realisedPnl] | min // 0)
  }
' /tmp/session_${SESSION_DATE}.json
```

### 4. Aggregate across sessions (after 5+ sessions)

```bash
cat /tmp/session_*.json | jq -s '
  flatten |
  {
    totalTrades: length,
    totalPnl: ([.[].realisedPnl] | add // 0),
    totalFees: ([.[].commission] | add // 0),
    winCount: ([.[] | select(.realisedPnl > 0)] | length)
  } |
  . + {winRate: (.winCount / .totalTrades * 100),
       netPnl: (.totalPnl - .totalFees)}
'
```

### 5. Promotion checklist

Promote to live only if all criteria pass:

- At least 5 independent sessions completed
- Win rate > 50%
- Average trade PnL positive after fees
- No single session loss exceeds 10% of starting balance
- Maximum drawdown within acceptable range

### 6. Promote to live

Remove `--testnet` flag and use your live API credentials. Start at 10% of testnet position sizes for the first week.

## Notes

- Testnet prices mirror live prices but fills may differ due to different liquidity.
- Re-validate on testnet after any significant strategy parameter change.
