#!/usr/bin/env bash
# Market Monitor
# Monitors prices, volumes, spreads, and order book depth for multiple pairs.
# Requirements: bitmex (on PATH), jq, bc
set -euo pipefail

PAIRS="BTCUSD ETHUSD SOLUSD"
SPREAD_THRESHOLD="0.5"

echo "=== Market Monitor ==="
echo "Pairs: $PAIRS"
echo "Spread alert threshold: ${SPREAD_THRESHOLD}%"
echo ""

# System status
echo "--- System Status ---"
bitmex status -o json 2>/dev/null | jq -r '"Status: \(.status // "unknown")"'
echo ""

# Ticker data
echo "--- Ticker Data ---"
for PAIR in $PAIRS; do
  TICKER=$(bitmex ticker "$PAIR" -o json 2>/dev/null)
  ASK=$(echo "$TICKER" | jq -r '.[].a[0] // "N/A"')
  BID=$(echo "$TICKER" | jq -r '.[].b[0] // "N/A"')
  LAST=$(echo "$TICKER" | jq -r '.[].c[0] // "N/A"')
  VOL=$(echo "$TICKER" | jq -r '.[].v[1] // "N/A"')
  echo "$PAIR: Ask=$ASK Bid=$BID Last=$LAST Vol24h=$VOL"
  if command -v bc >/dev/null 2>&1 && [ "$ASK" != "N/A" ] && [ "$BID" != "N/A" ]; then
    SPREAD=$(echo "scale=4; ($ASK - $BID) / $BID * 100" | bc -l 2>/dev/null || echo "N/A")
    echo "  Spread: ${SPREAD}%"
    ALERT=$(echo "$SPREAD > $SPREAD_THRESHOLD" | bc -l 2>/dev/null || echo "0")
    [ "$ALERT" = "1" ] && echo "  ** ALERT: Spread exceeds threshold **"
  fi
done
echo ""

# Order book depth
echo "--- BTCUSD Order Book (top 5) ---"
BOOK=$(bitmex orderbook BTCUSD --count 5 -o json 2>/dev/null)
echo "Asks:" && echo "$BOOK" | jq -r '.[].asks[:5][] | "  \(.[0]) @ \(.[1])"' 2>/dev/null || echo "  (error)"
echo "Bids:" && echo "$BOOK" | jq -r '.[].bids[:5][] | "  \(.[0]) @ \(.[1])"' 2>/dev/null || echo "  (error)"
echo ""

# Recent trades
echo "--- BTCUSD Recent Trades (last 5) ---"
TRADES=$(bitmex trades BTCUSD --count 5 -o json 2>/dev/null)
echo "$TRADES" | jq -r '.[keys[0]][:5][] | "\(.[3]) \(if .[4] == "b" then "BUY " else "SELL" end) \(.[1]) @ \(.[0])"' 2>/dev/null || echo "  (error)"
echo ""

# OHLC
echo "--- BTCUSD Hourly OHLC (last 3) ---"
OHLC=$(bitmex ohlc BTCUSD --interval 60 -o json 2>/dev/null)
echo "$OHLC" | jq -r '.[keys[0]][-3:][] | "  O:\(.[1]) H:\(.[2]) L:\(.[3]) C:\(.[4]) V:\(.[6])"' 2>/dev/null || echo "  (error)"
echo ""

echo "=== Monitor complete ==="
