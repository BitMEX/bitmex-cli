---
name: bitmex-portfolio-intel
version: 1.0.0
description: "Portfolio analysis on bitmex-cli: balance, positions, trade history, execution fills, margin state, and volume."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-portfolio-intel

Read-only commands for understanding current portfolio state. All commands in this skill require authentication but make no changes.

## Wallet Balance

```bash
# XBt (satoshi) balance
bitmex wallet balance --currency XBt -o json 2>/dev/null | \
  jq '{currency, amount, withdrawableAmount, pendingDebit, pendingCredit}'

# All currencies (returns single object for default currency; use --currency to specify)
bitmex wallet balance -o json 2>/dev/null | \
  jq '{currency, amount, withdrawableAmount}'

# Wallet summary across accounts (returns array of wallet events)
bitmex wallet summary -o json 2>/dev/null | \
  jq '[.[] | {transactType, amount, walletBalance, currency}]'
```

## Open Positions

```bash
# All open positions
bitmex position list -o json 2>/dev/null | \
  jq '[.[] | select(.isOpen == true) | {
    symbol, currentQty, avgEntryPrice, markPrice,
    unrealisedPnl, realisedPnl, liquidationPrice,
    leverage, roe: .unrealisedRoePcnt
  }]'

# Single symbol detail
bitmex position list --symbol XBTUSD -o json 2>/dev/null | jq '.[0]'
```

## Trade History

```bash
# Last 100 trades
bitmex execution trade-history --reverse --count 100 -o json 2>/dev/null | \
  jq '[.[] | {timestamp, symbol, side, lastPx, lastQty, commission, execType}]'

# Filter by symbol
bitmex execution trade-history --symbol XBTUSD --reverse --count 100 -o json 2>/dev/null | \
  jq '[.[] | select(.execType == "Trade") | {timestamp, side, lastPx, lastQty}]'

# PnL summary from trade history
bitmex execution trade-history --reverse --count 500 -o json 2>/dev/null | \
  jq '
    {
      total_trades: length,
      total_volume: (map(.lastQty // 0) | add),
      total_fees: (map(.commission // 0) | add),
      realised_pnl: (map(.realisedPnl // 0) | add)
    }
  '
```

## Execution Fills

```bash
# All executions including funding, settlement, and trades
bitmex execution list --reverse --count 100 -o json 2>/dev/null | \
  jq '[.[] | {timestamp, symbol, execType, side, lastPx, lastQty, commission}]'

# Funding payments received/paid
bitmex execution list --reverse --count 200 -o json 2>/dev/null | \
  jq '[.[] | select(.execType == "Funding") | {timestamp, symbol, commission, realisedPnl}]'
```

## Account Margin State

```bash
bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '{
    marginBalance,
    walletBalance,
    unrealisedPnl,
    realisedPnl,
    availableMargin,
    maintenanceMargin,
    marginLeverage,
    liquidationPrice: .grossLastValue
  }'
```

## 30-Day Volume

Used for fee tier assessment:

```bash
bitmex account volume -o json 2>/dev/null | \
  jq '.[0] | {advUsd, advUsdContract, advUsdSpot}'
```

## Commission Tiers

```bash
# Returns object keyed by symbol; query a specific symbol:
bitmex account commission -o json 2>/dev/null | \
  jq '.XBTUSD | {makerFee, takerFee, settlementFee}'

# All symbols:
bitmex account commission -o json 2>/dev/null | \
  jq 'to_entries | map({symbol: .key, makerFee: .value.makerFee, takerFee: .value.takerFee, settlementFee: .value.settlementFee})'
```

## Wallet Transaction History

```bash
# All wallet events (deposits, withdrawals, funding)
bitmex wallet history --currency XBt -o json 2>/dev/null | \
  jq '[.[] | {timestamp, transactType, amount, fee, address}]'

# Deposits only
bitmex wallet history --currency XBt -o json 2>/dev/null | \
  jq '[.[] | select(.transactType == "Deposit") | {timestamp, amount}]'
```

## Snapshot Report

```bash
echo "=== Portfolio Snapshot ==="
bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '"Balance: \(.marginBalance) sats | Unrealised PnL: \(.unrealisedPnl) sats"'
bitmex position list -o json 2>/dev/null | \
  jq '"Positions: \([.[] | select(.isOpen == true)] | length)"'
bitmex order list --reverse -o json 2>/dev/null | \
  jq '"Open orders: \([.[] | select(.ordStatus == "New" or .ordStatus == "PartiallyFilled")] | length)"'
```
