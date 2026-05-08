---
name: bitmex-ws-streaming
version: 1.0.0
description: "WebSocket streaming on bitmex-cli: topics, auth, NDJSON output, and piping to jq."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-ws-streaming

WebSocket subscriptions bypass the REST rate limit. Use them for any data that updates more frequently than once per minute.

## Connecting

```bash
# One or more topics
bitmex ws <topic1> [topic2...] [--auth] -o json 2>/dev/null
```

Output is NDJSON — one JSON object per line. Pipe to `jq -c` for compact streaming or `jq` for pretty-printing per message.

## Public Topics

No authentication required.

```bash
# Real-time trade feed
bitmex ws trade:XBTUSD -o json 2>/dev/null | \
  jq -c '.data[]? | {timestamp, side, price, size}'

# Level 2 order book (top 25 levels)
bitmex ws orderBookL2_25:XBTUSD -o json 2>/dev/null

# Top 10 order book
bitmex ws orderBook10:XBTUSD -o json 2>/dev/null | \
  jq -c '{bids: .data[0].bids[:3], asks: .data[0].asks[:3]}'

# Best bid/ask quote
bitmex ws quote:XBTUSD -o json 2>/dev/null | \
  jq -c '.data[]? | {bidPrice, bidSize, askPrice, askSize}'

# Instrument ticker (mark price, last price, funding rate)
bitmex ws instrument:XBTUSD -o json 2>/dev/null | \
  jq -c '.data[]? | {symbol, lastPrice, markPrice, fundingRate} | select(. != {})'

# Funding rate updates
bitmex ws funding -o json 2>/dev/null | \
  jq -c '.data[]? | {symbol, fundingRate, nextFundingTime}'

# Liquidation events
bitmex ws liquidation -o json 2>/dev/null | \
  jq -c '.data[]? | {timestamp, symbol, side, price, leavesQty}'

# Settlement
bitmex ws settlement -o json 2>/dev/null

# Insurance fund
bitmex ws insurance -o json 2>/dev/null

# Announcements
bitmex ws announcement -o json 2>/dev/null | \
  jq -c '.data[]? | {date, title}'
```

## Private Topics (--auth)

Requires `BITMEX_API_KEY` and `BITMEX_API_SECRET`.

```bash
# Open orders (new, amended, cancelled, filled)
bitmex ws --auth order -o json 2>/dev/null | \
  jq -c '.data[]? | {orderID, symbol, side, price, ordQty, ordStatus}'

# Execution fills
bitmex ws --auth execution -o json 2>/dev/null | \
  jq -c '.data[]? | select(.execType == "Trade") | {symbol, side, lastPx, lastQty, commission}'

# Position updates
bitmex ws --auth position -o json 2>/dev/null | \
  jq -c '.data[]? | {symbol, currentQty, markPrice, unrealisedPnl, liquidationPrice}'

# Margin / account state
bitmex ws --auth margin -o json 2>/dev/null | \
  jq -c '.data[]? | {marginBalance, availableMargin, unrealisedPnl}'

# Wallet balance
bitmex ws --auth wallet -o json 2>/dev/null | \
  jq -c '.data[]? | {currency, amount, withdrawableAmount}'

# Transactions
bitmex ws --auth transact -o json 2>/dev/null
```

## Multi-Topic Subscriptions

```bash
# Combine public and private in one connection
bitmex ws --auth trade:XBTUSD instrument:XBTUSD order position margin -o json 2>/dev/null

# Multi-symbol public feed
bitmex ws trade:XBTUSD trade:ETHUSD trade:SOLUSD -o json 2>/dev/null | \
  jq -c '.data[]? | {symbol, price, size}'
```

## Testnet WebSocket

```bash
bitmex --testnet ws trade:XBTUSD -o json 2>/dev/null
bitmex --testnet ws --auth order position margin -o json 2>/dev/null
```

## Processing NDJSON

```bash
# Write stream to file
bitmex ws trade:XBTUSD -o json 2>/dev/null >> /tmp/bitmex-trades.ndjson

# Process each line as it arrives
bitmex ws --auth execution -o json 2>/dev/null | while IFS= read -r line; do
  EXEC_TYPE=$(echo "$line" | jq -r '.data[0].execType // empty')
  if [ "$EXEC_TYPE" = "Trade" ]; then
    echo "FILL: $(echo "$line" | jq -c '.data[0] | {symbol, side, lastPx, lastQty}')"
  fi
done
```

## Reconnect on Disconnect

```bash
# Auto-reconnect loop
while true; do
  bitmex ws --auth position margin order -o json 2>/dev/null
  echo "WebSocket disconnected, reconnecting in 5s..." >&2
  sleep 5
done
```
