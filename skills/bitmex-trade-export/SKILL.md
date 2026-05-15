---
name: bitmex-trade-export
version: 1.0.0
description: "Export trade, execution, and wallet data from bitmex-cli: history fetch, date filtering, CSV conversion, and reconciliation."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex", "jq"]
  depends: ["bitmex-shared"]
---

# bitmex-trade-export

Export trade and execution data for record-keeping, tax reporting, and strategy analysis. All commands are read-only.

## Trade History

```bash
# Last 500 trades
bitmex execution trade-history --reverse --count 500 -o json 2>/dev/null | \
  jq '[.[] | {
    timestamp,
    symbol,
    side,
    lastPx,
    lastQty,
    commission,
    realisedPnl,
    execType,
    clOrdID
  }]'

# Single symbol
bitmex execution trade-history --symbol XBTUSD --reverse --count 500 -o json 2>/dev/null | \
  jq '[.[] | select(.execType == "Trade")]'
```

## Execution List (All Types)

Includes trades, funding payments, and settlements:

```bash
bitmex execution list --reverse --count 500 -o json 2>/dev/null | \
  jq '[.[] | {timestamp, symbol, execType, side, lastPx, lastQty, commission, realisedPnl}]'

# Funding payments only
bitmex execution list --reverse --count 500 -o json 2>/dev/null | \
  jq '[.[] | select(.execType == "Funding") | {timestamp, symbol, commission, realisedPnl}]'

# Settlements only
bitmex execution list --reverse --count 500 -o json 2>/dev/null | \
  jq '[.[] | select(.execType == "Settlement") | {timestamp, symbol, lastPx, realisedPnl}]'
```

## Date Filtering

```bash
# Trades from a specific start date
bitmex execution trade-history \
  --start-time "2026-01-01T00:00:00.000Z" \
  --reverse --count 500 -o json 2>/dev/null | \
  jq '[.[] | {timestamp, symbol, side, lastPx, lastQty, commission}]'

# Date range
bitmex execution trade-history \
  --start-time "2026-01-01T00:00:00.000Z" \
  --end-time "2026-03-01T00:00:00.000Z" \
  --reverse --count 500 -o json 2>/dev/null
```

## Wallet Transaction History

```bash
bitmex wallet history --currency XBt -o json 2>/dev/null | \
  jq '[.[] | {timestamp, transactType, amount, fee, address, transactStatus}]'
```

## Convert to CSV

```bash
# Trade history as CSV
bitmex execution trade-history --reverse --count 500 -o json 2>/dev/null | \
  jq -r '
    ["timestamp","symbol","side","price","qty","commission","realisedPnl","clOrdID"],
    (.[] | [.timestamp, .symbol, .side, .lastPx, .lastQty, (.commission // 0), (.realisedPnl // 0), (.clOrdID // "")]) |
    @csv
  ' > /tmp/bitmex-trades.csv

echo "Exported to /tmp/bitmex-trades.csv"
head -5 /tmp/bitmex-trades.csv
```

## Execution CSV with Exec Type

```bash
bitmex execution list --reverse --count 500 -o json 2>/dev/null | \
  jq -r '
    ["timestamp","symbol","execType","side","price","qty","commission","pnl"],
    (.[] | [
      .timestamp, .symbol, .execType, (.side // ""),
      (.lastPx // 0), (.lastQty // 0),
      (.commission // 0), (.realisedPnl // 0)
    ]) |
    @csv
  ' > /tmp/bitmex-executions.csv
```

## PnL Summary by Symbol

```bash
bitmex execution trade-history --reverse --count 500 -o json 2>/dev/null | \
  jq '
    group_by(.symbol) |
    map({
      symbol: .[0].symbol,
      trades: length,
      realised_pnl: (map(.realisedPnl // 0) | add),
      total_fees: (map(.commission // 0) | add),
      volume: (map(.lastQty // 0) | add)
    }) | sort_by(.realised_pnl) | reverse
  '
```

## Monthly Summary

```bash
bitmex execution trade-history --reverse --count 500 -o json 2>/dev/null | \
  jq '
    group_by(.timestamp[:7]) |
    map({
      month: .[0].timestamp[:7],
      trades: length,
      realised_pnl: (map(.realisedPnl // 0) | add),
      total_fees: (map(.commission // 0) | add)
    })
  '
```

## Reconciliation Check

Compare execution history against current position:

```bash
# Expected avg entry from fills
AVG=$(bitmex execution trade-history --symbol XBTUSD --reverse --count 200 -o json 2>/dev/null | \
  jq '
    [.[] | select(.side == "Buy" and .execType == "Trade")] |
    (map(.lastQty * .lastPx) | add) / (map(.lastQty) | add) | round / 100
  ')

# Reported avg entry from position
REPORTED=$(bitmex position list --symbol XBTUSD -o json 2>/dev/null | \
  jq -r '.[0].avgEntryPrice // 0')

echo "Calculated avg entry: $AVG | Reported: $REPORTED"
```
