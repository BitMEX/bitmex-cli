---
name: bitmex-recipe-price-level-alerts
description: Set up price level alerts for key levels.
---

# Price Level Alerts

Get notified when a symbol crosses a key price level. Three approaches: API-based alerts, polling, and WebSocket streaming.

## Prerequisites

- No API key needed for market data approaches.
- `BITMEX_API_KEY` set for API alert registration.
- `jq` and `bc` installed.

## Approach 1: Register API Alerts

BitMEX can push notifications to your account when levels are hit:

```bash
# Alert when XBTUSD goes above 100000
bitmex notifications add-alert XBTUSD 100000 --above -o json \
  | jq '{alertID, symbol, price, direction}'

# Alert when XBTUSD drops below 90000 (default direction is below; omit --above)
bitmex notifications add-alert XBTUSD 90000 -o json \
  | jq '{alertID, symbol, price, direction}'
```

List active alerts:

```bash
bitmex notifications alerts -o json | jq '[.[] | {alertID, symbol, price, direction, triggered}]'
```

## Approach 2: Polling Loop

Simple and reliable; polls every 15 seconds:

```bash
SYMBOL=XBTUSD
ALERT_ABOVE=100000
ALERT_BELOW=90000

while true; do
  PRICE=$(bitmex market instrument --symbol $SYMBOL -o json | jq '.[0].lastPrice')
  TS=$(date -u +%H:%M:%S)

  if (( $(echo "$PRICE > $ALERT_ABOVE" | bc -l) )); then
    echo "$TS ALERT: $SYMBOL $PRICE crossed ABOVE $ALERT_ABOVE"
    # Add notification hook here (e.g. curl to webhook, send SMS)
    break
  elif (( $(echo "$PRICE < $ALERT_BELOW" | bc -l) )); then
    echo "$TS ALERT: $SYMBOL $PRICE crossed BELOW $ALERT_BELOW"
    break
  fi

  sleep 15
done
```

## Approach 3: WebSocket Stream with jq Filter

Zero-poll, lowest latency:

```bash
bitmex ws instrument:XBTUSD \
  | jq --argjson above 100000 --argjson below 90000 \
      'select(.data != null) | .data[]
       | select(.lastPrice != null)
       | select(.lastPrice > $above or .lastPrice < $below)
       | {symbol, lastPrice, timestamp}'
```

## Notes

- Polling at 15-second intervals on one symbol uses ~20 req/5 min — well within the 300 req/5 min rate limit.
- For multiple symbols, use the WebSocket approach with comma-separated topics: `bitmex ws instrument:XBTUSD instrument:ETHUSD`.
