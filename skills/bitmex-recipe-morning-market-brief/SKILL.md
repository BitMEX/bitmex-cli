---
name: bitmex-recipe-morning-market-brief
description: Morning summary: prices, funding, positions, and wallet.
---

# Morning Market Brief

A structured morning routine that pulls announcements, instrument snapshots, funding rates, positions, wallet balance, and open orders in one pass.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set for private endpoints.
- `jq` installed for JSON shaping.

## Steps

### 1. Check announcements

Platform notices, maintenance windows, and fee changes appear here first.

```bash
bitmex announce list -o json | jq '.[] | {title, date, body: .content[:120]}'
```

### 2. Active instruments snapshot

Grab last price, mark price, and 24 h volume for all active contracts.

```bash
bitmex market instrument --active -o json \
  | jq '[.[] | {symbol, lastPrice, markPrice, volume24h: .volume24h}]'
```

Filter to perps only:

```bash
bitmex market instrument --active -o json \
  | jq '[.[] | select(.typ == "FFWCSX") | {symbol, lastPrice, markPrice, fundingRate}]'
```

### 3. Funding rates (last interval)

```bash
bitmex market funding --count 1 --reverse -o json \
  | jq '[.[] | {symbol, fundingRate, timestamp}]'
```

### 4. Open positions

```bash
bitmex position list -o json \
  | jq '[.[] | select(.currentQty != 0) | {symbol, currentQty, avgEntryPrice, unrealisedPnl, liquidationPrice}]'
```

### 5. Wallet balance

```bash
bitmex account margin -o json \
  | jq '{walletBalance, marginBalance, availableMargin, unrealisedPnl, currency}'
```

### 6. Open orders

```bash
bitmex order list --reverse -o json \
  | jq '[.[] | {orderID, symbol, side, orderQty, price, ordStatus}]'
```

## Tips

- Run as a morning cron at 08:55 UTC before the 09:00 funding settlement.
- Pipe all output to a dated log: `>> ~/logs/brief-$(date +%F).json`.
- On high-volatility days, re-run funding step every 15 minutes to track rate spikes.
