---
name: bitmex-alert-patterns
version: 1.0.0
description: "Price, funding, liquidation, and balance alerts using polling and WebSocket on bitmex-cli."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-rate-limits", "bitmex-ws-streaming"]
---

# bitmex-alert-patterns

Two approaches: REST polling (low-frequency) and WebSocket streaming (real-time). Use WebSocket wherever possible to stay within the 300 req/5min REST budget.

## Native Price Alerts

BitMEX supports native price alerts via the notifications API:

```bash
# Alert when XBTUSD goes above 55000
bitmex notifications add-alert XBTUSD 55000 --above -o json 2>/dev/null

# Alert when XBTUSD drops below 45000
bitmex notifications add-alert XBTUSD 45000 -o json 2>/dev/null

# List active alerts
bitmex notifications alerts -o json 2>/dev/null

# Delete an alert
bitmex notifications delete-alert <id> -o json 2>/dev/null
```

## WebSocket Price Alert

Real-time price monitoring with no REST cost:

```bash
# Alert when last price crosses threshold
THRESHOLD=55000
bitmex ws instrument:XBTUSD -o json 2>/dev/null | \
  jq -c --argjson t "$THRESHOLD" '
    .data[]? | select(.lastPrice != null) |
    if .lastPrice > $t then "ALERT: XBTUSD above \($t): \(.lastPrice)"
    else empty end
  '
```

## REST Polling Alert

For lower-frequency checks (e.g., hourly funding):

```bash
THRESHOLD=0.001  # 0.1% per 8h
while true; do
  RATE=$(bitmex market funding --symbol XBTUSD -o json 2>/dev/null | \
    jq -r '.[0].fundingRate // 0')
  if [ "$(echo "$RATE > $THRESHOLD" | bc -l)" = "1" ]; then
    echo "ALERT: XBTUSD funding rate $RATE exceeds threshold $THRESHOLD"
  fi
  sleep 300  # check every 5 minutes
done
```

## Funding Rate Alert

```bash
# WebSocket funding alert
bitmex ws funding -o json 2>/dev/null | \
  jq -c '.data[]? | select(.fundingRate != null and (.fundingRate | fabs) > 0.001) |
    "ALERT: \(.symbol) funding \(.fundingRate) at \(.timestamp)"'
```

## Liquidation Alert

Monitor large liquidations as a market signal:

```bash
# Stream all liquidations
bitmex ws liquidation -o json 2>/dev/null | \
  jq -c '.data[]? | "LIQUIDATION: \(.side) \(.leavesQty) \(.symbol) @ \(.price)"'

# Filter for large liquidations only
bitmex ws liquidation -o json 2>/dev/null | \
  jq -c '.data[]? | select(.leavesQty > 100000) |
    "LARGE LIQ: \(.side) \(.leavesQty) \(.symbol) @ \(.price)"'
```

## Balance Change Alert (Authenticated)

```bash
# Stream wallet and margin updates
bitmex ws --auth wallet margin -o json 2>/dev/null | \
  jq -c '
    if .table == "margin" then
      .data[]? | "MARGIN: available=\(.availableMargin) balance=\(.marginBalance)"
    elif .table == "wallet" then
      .data[]? | "WALLET: \(.currency) balance=\(.amount)"
    else empty end
  '
```

## Position Unrealised PnL Alert

> **Note:** This monitors `unrealisedPnl` only — mark-to-market movement. Funding costs accumulate separately every 8 hours and are not reflected until charged. For a full position P&L view, also monitor the funding rate.

```bash
PNL_THRESHOLD=-50000  # in satoshis (XBt instruments); use appropriate units for USDT instruments
bitmex ws --auth position -o json 2>/dev/null | \
  jq -c --argjson t "$PNL_THRESHOLD" '
    .data[]? | select(.unrealisedPnl != null and .unrealisedPnl < $t) |
    "ALERT: \(.symbol) unrealised PnL \(.unrealisedPnl) below threshold"
  '
```

## Order Fill Notification

```bash
# Alert on every fill
bitmex ws --auth execution -o json 2>/dev/null | \
  jq -c '.data[]? | select(.execType == "Trade") |
    "FILL: \(.side) \(.lastQty) \(.symbol) @ \(.lastPx) [\(.ordStatus)]"'
```

## Announcement Monitoring

```bash
# Check for urgent system announcements
bitmex announce urgent -o json 2>/dev/null | jq '[.[] | {date, title, content}]'

# Poll every 10 minutes for new announcements
while true; do
  bitmex announce urgent -o json 2>/dev/null | jq -r '.[] | "\(.date): \(.title)"'
  sleep 600
done
```
