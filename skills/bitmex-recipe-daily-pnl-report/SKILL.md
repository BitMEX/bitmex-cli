---
name: bitmex-recipe-daily-pnl-report
description: Daily realised P&L summary from trades and wallet history.
---

# Daily P&L Report

Aggregate today's realised P&L, fees paid, and unrealised exposure from trade history and wallet snapshots.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` installed.
- Set `TODAY` to the current UTC date.

```bash
TODAY=$(date -u +%Y-%m-%d)
```

## Steps

### 1. Fetch today's trade history

```bash
bitmex execution trade-history --reverse --count 500 -o json \
  | jq --arg d "$TODAY" '[.[] | select(.timestamp | startswith($d))]' \
  > /tmp/trades_today.json
```

### 2. Sum realised P&L by currency

> P&L units depend on the instrument's settlement currency. XBt-settled instruments (e.g. XBTUSD) report in satoshis; divide by 1e8 for XBT. USDT-settled instruments (e.g. XBTUSDT) report in USDT units. Always group by `homeNotional` currency rather than summing across all symbols.

```bash
jq 'group_by(.currency) | map({
  currency: .[0].currency,
  realisedPnl: ([.[].realisedPnl] | add // 0)
})' /tmp/trades_today.json
```

### 3. Total fees paid today

```bash
jq '[.[].commission] | add // 0' /tmp/trades_today.json
```

### 4. Breakdown by symbol

```bash
jq 'group_by(.symbol) | map({
  symbol: .[0].symbol,
  trades: length,
  realisedPnl: ([.[].realisedPnl] | add // 0),
  feesTotal: ([.[].commission] | add // 0)
})' /tmp/trades_today.json
```

### 5. Wallet balance delta

Pull opening and closing wallet snapshots from history:

```bash
bitmex wallet history --currency XBt --count 100 -o json \
  | jq --arg d "$TODAY" '[.[] | select(.timestamp | startswith($d))]
      | if length == 0 then {first: null, last: null, delta: 0}
        else {first: .[-1].walletBalance, last: .[0].walletBalance,
              delta: ((.[0].walletBalance // 0) - (.[-1].walletBalance // 0))}
        end'
```

### 6. Unrealised P&L on open positions

```bash
bitmex position list -o json \
  | jq '{totalUnrealisedPnl: ([.[].unrealisedPnl] | add // 0),
         positions: [.[] | select(.currentQty != 0) | {symbol, currentQty, unrealisedPnl}]}'
```

## Tips

- Automate with a cron at 23:55 UTC and append to a CSV for trend analysis.
- XBt-settled instruments (inverse, e.g. XBTUSD) report P&L in satoshis; divide by 1e8 for XBT. USDT-settled instruments report in USDT. Never sum across currencies.
