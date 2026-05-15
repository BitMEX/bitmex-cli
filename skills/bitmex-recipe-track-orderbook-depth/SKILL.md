---
name: bitmex-recipe-track-orderbook-depth
description: Monitor order book depth and bid-ask imbalance for liquidity signals.
---

# Order Book Depth Tracker

Measure bid/ask quantity imbalance at the top of the book as a short-term directional signal. Heavy imbalance toward bids suggests buying pressure; heavy ask-side imbalance suggests selling pressure.

## Prerequisites

- No API key required.
- `jq` and `bc` installed.

## Steps

### 1. Snapshot the top 25 levels

```bash
bitmex market orderbook XBTUSD --depth 25 -o json \
  | jq '{bids: [.[] | select(.side=="Buy") | {price: .price, size: .size}],
         asks: [.[] | select(.side=="Sell") | {price: .price, size: .size}]}'
```

### 2. Calculate bid/ask imbalance from snapshot

```bash
BOOK=$(bitmex market orderbook XBTUSD --depth 10 -o json)

BID_QTY=$(echo "$BOOK" | jq '[.[] | select(.side=="Buy") | .size] | add')
ASK_QTY=$(echo "$BOOK" | jq '[.[] | select(.side=="Sell") | .size] | add')
TOTAL=$(echo "$BID_QTY + $ASK_QTY" | bc)

IMBALANCE=$(echo "scale=4; $BID_QTY / $TOTAL * 100" | bc)
echo "Bid imbalance: ${IMBALANCE}% (>70% = bullish signal, <30% = bearish)"
```

### 3. Stream live order book updates

```bash
bitmex ws orderBookL2_25:XBTUSD \
  | jq 'select(.action == "update" or .action == "insert") | .data[:3]'
```

### 4. Detect large wall insertions

```bash
bitmex ws orderBookL2_25:XBTUSD \
  | jq 'select(.action == "insert") | .data[]
        | select(.size > 5000000) | {side, price, size}'
```

Walls above 5 million contracts on one side often act as temporary price magnets or blockers.

### 5. Periodic imbalance polling loop

```bash
while true; do
  BOOK=$(bitmex market orderbook XBTUSD --depth 10 -o json)
  BID=$(echo "$BOOK" | jq '[.[] | select(.side=="Buy") | .size] | add')
  ASK=$(echo "$BOOK" | jq '[.[] | select(.side=="Sell") | .size] | add')
  IMBALANCE=$(echo "scale=2; $BID / ($BID + $ASK) * 100" | bc)
  echo "$(date -u +%H:%M:%S) Bid imbalance: ${IMBALANCE}%"

  if (( $(echo "$IMBALANCE > 70" | bc -l) )); then
    echo "Strong bid pressure detected"
  elif (( $(echo "$IMBALANCE < 30" | bc -l) )); then
    echo "Strong ask pressure detected"
  fi

  sleep 10
done
```

## Notes

- Imbalance is a leading indicator but can be spoofed. Combine with recent trade flow from `bitmex market trades`.
- Rate limit is 300 req/5 min — polling every 10 seconds on one symbol uses ~30 req/5 min, safely within limits.
