---
name: bitmex-market-intel
version: 1.0.0
description: "Market data commands for bitmex-cli: instruments, order books, trades, candles, funding, and quotes."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-market-intel

All market data commands require no authentication. Use these for dashboards, signal generation, and pre-trade analysis.

## Instrument Info

Retrieve contract specifications, mark price, open interest, and 24h volume.

```bash
# Single instrument
bitmex market instrument --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {symbol, lastPrice, markPrice, openInterest, volume24h: .volume}'

# All active instruments
bitmex market instrument --active -o json 2>/dev/null | \
  jq '[.[] | {symbol, lastPrice, fundingRate, openInterest}]'

# Index prices only
bitmex market instrument --indices -o json 2>/dev/null | \
  jq '[.[] | {symbol, lastPrice}]'
```

## Order Book

```bash
# Top 25 levels (default) — response is a flat array of {side, price, size}
bitmex market orderbook XBTUSD -o json 2>/dev/null | \
  jq '{bids: [.[] | select(.side=="Buy") | [.price,.size]][:5], asks: [.[] | select(.side=="Sell") | [.price,.size]][:5]}'

# Top 5 levels
bitmex market orderbook XBTUSD --depth 5 -o json 2>/dev/null

# Mid price calculation
bitmex market orderbook XBTUSD --depth 1 -o json 2>/dev/null | \
  jq '([.[] | select(.side=="Buy") | .price][0] + [.[] | select(.side=="Sell") | .price][0]) / 2'
```

## Recent Trades

```bash
# Last 100 trades
bitmex market trades --symbol XBTUSD --reverse --count 100 -o json 2>/dev/null | \
  jq '[.[] | {timestamp, side, price, size}]'

# Bucketed candles (OHLCV)
bitmex market trades --symbol XBTUSD --bucketed --bin-size 1h -o json 2>/dev/null | \
  jq '[.[] | {timestamp, open, high, low, close, volume}]'

# 5-minute candles
bitmex market trades --symbol XBTUSD --bucketed --bin-size 5m -o json 2>/dev/null | \
  jq 'last'
```

## Quote (Best Bid/Ask)

```bash
# Current best bid/ask
bitmex market quote --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {bidPrice, bidSize, askPrice, askSize}'

# Bucketed quote history
bitmex market quote --symbol XBTUSD --bucketed --bin-size 1h -o json 2>/dev/null
```

## Funding Rate

Perpetual contracts pay or receive funding every 8 hours (00:00, 08:00, 16:00 UTC).

```bash
# Current funding rate
bitmex market funding --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {symbol, fundingRate, fundingRateDaily: (.fundingRate * 3), timestamp}'

# Funding history (last 10 periods)
bitmex market funding --symbol XBTUSD --count 10 --reverse -o json 2>/dev/null | \
  jq '[.[] | {timestamp, fundingRate}]'
```

## Liquidations

```bash
# Recent liquidations for XBTUSD
bitmex market liquidation --symbol XBTUSD -o json 2>/dev/null | \
  jq '[.[] | {timestamp, side, price, leavesQty}]'
```

## Settlement

```bash
# Upcoming and past settlements
bitmex market settlement --symbol XBTM25 -o json 2>/dev/null | \
  jq '.[0] | {timestamp, settledPrice}'
```

## Insurance Fund

```bash
bitmex market insurance -o json 2>/dev/null | \
  jq '[.[] | {currency, walletBalance}]'
```

## Exchange Stats

```bash
# 24h volume and open interest across all products
bitmex market stats -o json 2>/dev/null | \
  jq '[.[] | {rootSymbol, currency, volume24h, openInterest}]'
```

## Leaderboard

```bash
# Top traders by notional
bitmex market leaderboard --method notional -o json 2>/dev/null | jq '.[0:5]'

# Top traders by ROE (not supported on mainnet — omit or use --method notional)
# bitmex market leaderboard --method roe -o json 2>/dev/null | jq '.[0:5]'
```
