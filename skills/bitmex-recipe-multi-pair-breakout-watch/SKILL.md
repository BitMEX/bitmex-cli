---
name: bitmex-recipe-multi-pair-breakout-watch
description: Monitor multiple symbols for price breakouts.
---

# Multi-Pair Breakout Watch

Watch several perpetual contracts simultaneously for price breakouts above resistance or below support, optionally confirmed by volume.

## Prerequisites

- No API key required for market data.
- `jq` installed.
- Define levels for each symbol you want to watch.

## Steps

### 1. Get reference prices for all active instruments

```bash
bitmex market instrument --active -o json \
  | jq '[.[] | select(.typ == "FFWCSX") | {symbol, lastPrice, high24h: .highPrice, low24h: .lowPrice}]'
```

Use `highPrice` and `lowPrice` as initial support/resistance candidates.

### 2. Define your watch levels

```bash
declare -A RESISTANCE=( ["XBTUSD"]=100000 ["ETHUSD"]=4000 ["SOLUSDT"]=250 )
declare -A SUPPORT=(    ["XBTUSD"]=90000  ["ETHUSD"]=3500 ["SOLUSDT"]=200 )
```

### 3. Stream prices via WebSocket (multiple symbols)

```bash
bitmex ws trade:XBTUSD trade:ETHUSD trade:SOLUSDT \
  | jq '{symbol: .data[0].symbol, price: .data[0].price}'
```

### 4. Polling alert loop (alternative to WebSocket)

```bash
SYMBOLS="XBTUSD ETHUSD SOLUSDT"

while true; do
  for SYM in $SYMBOLS; do
    PRICE=$(bitmex market instrument --symbol $SYM -o json | jq '.[0].lastPrice')

    R=${RESISTANCE[$SYM]}
    S=${SUPPORT[$SYM]}

    if (( $(echo "$PRICE > $R" | bc -l) )); then
      echo "BREAKOUT UP: $SYM @ $PRICE > resistance $R"
    elif (( $(echo "$PRICE < $S" | bc -l) )); then
      echo "BREAKDOWN: $SYM @ $PRICE < support $S"
    fi
  done
  sleep 30
done
```

### 5. Confirm with recent volume

```bash
bitmex market trades --symbol XBTUSD --bucketed --bin-size 1h --count 3 -o json \
  | jq '[.[] | {timestamp, volume, open, close}]'
```

A breakout on above-average volume (>1.5x the 3-period average) is more reliable than a low-volume spike.

## Notes

- Stream via WebSocket for latency-sensitive setups; polling is fine for swing-level monitoring.
- Combine with the `recipe-price-level-alerts` skill for simple one-shot notifications.
