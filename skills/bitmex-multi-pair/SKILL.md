---
name: bitmex-multi-pair
version: 1.0.0
description: "Multi-symbol screening, funding rate comparison, and WebSocket multi-subscribe on bitmex-cli."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-market-intel", "bitmex-rate-limits"]
---

# bitmex-multi-pair

BitMEX offers perpetuals and fixed-date futures across multiple underlyings. Fetch all active instruments once, filter locally to minimize API calls.

## Fetch All Active Instruments

```bash
# Fetch once and cache
ALL=$(bitmex market instrument --active -o json 2>/dev/null)

# Perpetuals only (typ == FFWCSX)
echo "$ALL" | jq '[.[] | select(.typ == "FFWCSX") | {symbol, lastPrice, fundingRate, volume}]'

# Fixed-date futures only
echo "$ALL" | jq '[.[] | select(.typ == "FFCCSX") | {symbol, lastPrice, settleDate: .settle}]'

# Equity perpetuals
echo "$ALL" | jq '[.[] | select(.rootSymbol == "AAPL") | {symbol, lastPrice}]'
```

## Funding Rate Screening

Find the highest-paying funding rates across all perps:

```bash
bitmex market instrument --active -o json 2>/dev/null | \
  jq '[.[] | select(.fundingRate != null) | {symbol, fundingRate, annualized: (.fundingRate * 3 * 365)}] | sort_by(.fundingRate) | reverse | .[0:10]'
```

Identify negative funding (longs get paid):

```bash
bitmex market instrument --active -o json 2>/dev/null | \
  jq '[.[] | select(.fundingRate != null and .fundingRate < 0) | {symbol, fundingRate}]'
```

## Volume Screening

Find most liquid markets:

```bash
bitmex market stats -o json 2>/dev/null | \
  jq '[.[] | {rootSymbol, currency, volume24h, openInterest}] | sort_by(.volume24h) | reverse | .[0:5]'
```

Filter by minimum volume from active instruments:

```bash
bitmex market instrument --active -o json 2>/dev/null | \
  jq '[.[] | select(.volume != null and .volume > 10000000) | {symbol, volume, lastPrice}]'
```

## Loop Over Symbol List

When you need per-symbol data not available in bulk (e.g., order book):

```bash
SYMBOLS=("XBTUSD" "ETHUSD" "SOLUSD")
for SYM in "${SYMBOLS[@]}"; do
  DEPTH=$(bitmex market orderbook "$SYM" --depth 1 -o json 2>/dev/null | \
    jq '{symbol: .[0].symbol, spread: (([.[] | select(.side == "Sell")] | .[0].price) - ([.[] | select(.side == "Buy")] | .[0].price))}')
  echo "$DEPTH"
  sleep 1  # stay within 300 req/5min budget
done
```

## WebSocket Multi-Subscribe

Subscribe to multiple symbols in a single connection:

```bash
# Trades for two symbols
bitmex ws trade:XBTUSD trade:ETHUSD -o json 2>/dev/null | \
  jq -c '{symbol: .data[0].symbol, price: .data[0].price, size: .data[0].size}'

# Order book for multiple symbols
bitmex ws orderBookL2_25:XBTUSD orderBookL2_25:ETHUSD -o json 2>/dev/null

# Funding rates for all instruments (no symbol filter = all)
bitmex ws funding instrument -o json 2>/dev/null | \
  jq -c 'select(.table == "funding") | .data[] | {symbol, fundingRate}'
```

## Authenticated Multi-Symbol Position Monitor

```bash
# Monitor all positions in real-time
bitmex ws --auth position -o json 2>/dev/null | \
  jq -c '.data[] | {symbol, currentQty, markPrice, unrealisedPnl, liquidationPrice}'
```

## Identify Basis Opportunities

Compare perp vs nearest future across multiple underlyings:

```bash
bitmex market instrument --active -o json 2>/dev/null | \
  jq '
    (map(select(.typ == "FFWCSX")) | map({root: .rootSymbol, perp: .lastPrice, sym: .symbol})) as $perps |
    (map(select(.typ == "FFCCSX")) | map({root: .rootSymbol, fut: .lastPrice, sym: .symbol})) as $futs |
    [$perps[] as $p | $futs[] | select(.root == $p.root) | {
      root: .root, perp: $p.perp, future: .fut,
      basis_pct: ((.fut - $p.perp) / $p.perp * 100)
    }]
  '
```
