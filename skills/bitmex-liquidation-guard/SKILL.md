---
name: bitmex-liquidation-guard
version: 1.0.0
description: "Liquidation prevention on bitmex-cli: margin health, emergency flatten, dead man's switch, and real-time monitoring."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-position-risk", "bitmex-ws-streaming"]
---

# bitmex-liquidation-guard

Liquidation permanently destroys margin. These controls must be active whenever a leveraged position is open.

## Margin Health Check

```bash
bitmex account margin --currency XBt -o json 2>/dev/null | jq '
  {
    marginBalance,
    availableMargin,
    maintMargin,
    health_pct: ((.availableMargin / .marginBalance * 100) | round / 100)
  }
'
```

Alert threshold: if `availableMargin / marginBalance < 0.20` (less than 20% buffer), reduce position.

## Liquidation Price Buffer

```bash
bitmex position list -o json 2>/dev/null | jq '
  [.[] | select(.isOpen == true) | {
    symbol,
    markPrice,
    liquidationPrice,
    buffer_pct: (((.markPrice - .liquidationPrice) / .markPrice * 100) | fabs | (. * 100 | round) / 100)
  }]
'
```

Minimum acceptable buffer: 5%. Below 3%, emergency procedures apply.

## Alert Thresholds via WebSocket

```bash
# Monitor margin in real-time; alert if below threshold
THRESHOLD=20  # percent
bitmex ws --auth margin -o json 2>/dev/null | \
  jq -c --argjson t "$THRESHOLD" '
    .data[]? |
    ((.availableMargin / .marginBalance * 100) | round / 100) as $pct |
    if $pct < $t then "ALERT: margin health \($pct)% below threshold \($t)%" else empty end
  '
```

## Step 1: Reduce Leverage

When margin buffer is shrinking, reduce exposure first:

```bash
# Reduce leverage before it becomes critical
bitmex position leverage XBTUSD 5 -o json 2>/dev/null

# Transfer more margin into isolated position
bitmex position transfer-margin XBTUSD 500000 -o json 2>/dev/null
```

## Step 2: Emergency Cancel-All

Cancel all open orders immediately to stop adding exposure:

```bash
bitmex order cancel-all -o json 2>/dev/null | \
  jq '{cancelled: (. | length)}'
```

## Step 3: Emergency Flatten

Close all open positions immediately. This uses market orders — taker fee applies.

```bash
# Close XBTUSD position immediately
bitmex order close-position XBTUSD -o json 2>/dev/null

# Close all positions programmatically
bitmex position list -o json 2>/dev/null | \
  jq -r '[.[] | select(.isOpen == true and .currentQty != 0) | .symbol] | .[]' | \
  while read -r SYM; do
    echo "Closing $SYM..."
    bitmex order close-position "$SYM" -o json 2>/dev/null | \
      jq '{symbol: .symbol, orderID}'
  done
```

## Dead Man's Switch

The dead man's switch cancels all open orders if not renewed. Essential for automated strategies — a crash or network outage will not leave orphaned orders.

```bash
# Enable: cancels all orders if not renewed within 60 seconds
bitmex order cancel-after 60000 -o json 2>/dev/null

# Heartbeat loop — renew every 30 seconds
while true; do
  RESULT=$(bitmex order cancel-after 60000 -o json 2>/dev/null)
  if [ $? -ne 0 ]; then
    echo "DMS renewal failed: $(echo $RESULT | jq -r '.message')" >&2
  fi
  sleep 30
done
```

To disable the dead man's switch, set timeout to 0:

```bash
bitmex order cancel-after 0 -o json 2>/dev/null
```

## Real-Time Liquidation Feed

Monitor when other traders get liquidated (useful as a market signal):

```bash
bitmex ws liquidation -o json 2>/dev/null | \
  jq -c '.data[]? | {timestamp, symbol, side, price, leavesQty}'
```

## Full Emergency Sequence

```bash
emergency_flatten() {
  echo "EMERGENCY: Cancelling all orders..."
  bitmex order cancel-all -o json 2>/dev/null

  echo "EMERGENCY: Closing all positions..."
  bitmex position list -o json 2>/dev/null | \
    jq -r '[.[] | select(.isOpen == true and .currentQty != 0) | .symbol] | .[]' | \
    while read -r SYM; do
      bitmex order close-position "$SYM" -o json 2>/dev/null
    done

  echo "EMERGENCY: Done. Verifying..."
  bitmex position list -o json 2>/dev/null | \
    jq '[.[] | select(.isOpen == true) | {symbol, currentQty}]'
}
```
