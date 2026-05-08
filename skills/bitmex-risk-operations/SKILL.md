---
name: bitmex-risk-operations
version: 1.0.0
description: "Operational risk controls for bitmex-cli: pre-flight checks, dead man's switch, mass cancel, emergency close, and error routing."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-error-recovery", "bitmex-liquidation-guard"]
---

# bitmex-risk-operations

Risk controls that must be active in every trading session. These procedures prevent catastrophic loss from software bugs, network failures, or unexpected market moves.

## Pre-Flight Checklist

Run before starting any automated strategy:

```bash
echo "=== Pre-Flight Checks ==="

# 1. Connectivity
bitmex market instrument --symbol XBTUSD -o json 2>/dev/null | \
  jq '"OK: last price \(.[0].lastPrice)"' || echo "FAIL: cannot reach API"

# 2. Auth
bitmex account me -o json 2>/dev/null | \
  jq '"OK: authenticated as \(.username)"' || echo "FAIL: auth error"

# 3. Margin check
bitmex account margin --currency XBt -o json 2>/dev/null | jq '
  if .marginBalance > 0 and (.availableMargin / .marginBalance) < 0.3 then
    "WARN: low margin \(.availableMargin) / \(.marginBalance)"
  else
    "OK: margin health \(if .marginBalance > 0 then (.availableMargin / .marginBalance * 100 | round / 100) else 100 end)%"
  end
'

# 4. Stale open orders
OPEN=$(bitmex order list --reverse -o json 2>/dev/null | jq '[.[] | select(.ordStatus == "New")] | length')
echo "Open orders: $OPEN"

# 5. Existing positions
bitmex position list -o json 2>/dev/null | \
  jq '[.[] | select(.isOpen == true) | {symbol, currentQty, unrealisedPnl}]'
```

## Position Size Limits

Enforce before every order placement:

```bash
check_position_limit() {
  local SYMBOL=$1
  local MAX_QTY=$2
  local CURRENT
  CURRENT=$(bitmex position list --symbol "$SYMBOL" -o json 2>/dev/null | \
    jq -r '.[0].currentQty // 0')
  ABS=${CURRENT#-}
  if [ "$ABS" -ge "$MAX_QTY" ]; then
    echo "BLOCKED: $SYMBOL position $CURRENT at or above limit $MAX_QTY"
    return 1
  fi
  return 0
}

check_position_limit XBTUSD 5000 || exit 1
```

## Dead Man's Switch

Required for all automated strategies. Cancels all open orders if process dies.

```bash
# Enable: 60-second timeout
bitmex order cancel-after 60000 -o json 2>/dev/null

# Heartbeat — renew every 30 seconds in background
(while true; do
  bitmex order cancel-after 60000 -o json 2>/dev/null > /dev/null
  sleep 30
done) &
DMS_PID=$!

# Register cleanup on exit
trap "
  kill $DMS_PID 2>/dev/null
  bitmex order cancel-all -o json 2>/dev/null
  echo 'Dead man\\'s switch disabled, all orders cancelled'
" EXIT
```

## Mass Cancel

```bash
# Cancel all orders on a specific symbol
bitmex order cancel-all --symbol XBTUSD -o json 2>/dev/null

# Cancel ALL open orders across all symbols
bitmex order cancel-all -o json 2>/dev/null | jq '{cancelled: (. | length)}'
```

## Emergency Close

```bash
# Close single position
bitmex order close-position XBTUSD -o json 2>/dev/null

# Close all open positions
bitmex position list -o json 2>/dev/null | \
  jq -r '[.[] | select(.isOpen == true and .currentQty != 0) | .symbol] | .[]' | \
  while read -r SYM; do
    echo "Closing $SYM..."
    bitmex order close-position "$SYM" -o json 2>/dev/null | \
      jq -c '{symbol: .symbol, orderID, ordStatus}'
  done
```

## Error Category Routing

```bash
handle_error() {
  local RESULT=$1
  local CATEGORY
  CATEGORY=$(echo "$RESULT" | jq -r '.error // "unknown"')
  case "$CATEGORY" in
    rate_limit)
      WAIT=$(echo "$RESULT" | jq -r '.suggestion // ""' | grep -o '[0-9]*' | head -1)
      echo "Rate limited — sleeping ${WAIT:-60}s"
      sleep "${WAIT:-60}"
      ;;
    auth)
      echo "Auth failure — halting strategy"
      exit 1
      ;;
    network)
      echo "Network error — waiting 10s before retry"
      sleep 10
      ;;
    validation)
      echo "Validation error: $(echo "$RESULT" | jq -r '.message')"
      echo "Do not retry without fixing parameters"
      exit 1
      ;;
    *)
      echo "Unhandled error [$CATEGORY]: $(echo "$RESULT" | jq -r '.message')"
      ;;
  esac
}
```

## Daily Risk Report

```bash
echo "=== Daily Risk Report ==="
bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '{marginBalance, walletBalance, unrealisedPnl, availableMargin}'
# Note: realisedPnl on the margin object is always 0 — BitMEX credits realised P&L to
# walletBalance immediately. Use walletBalance delta over time for realised performance.
bitmex execution trade-history --reverse --count 100 -o json 2>/dev/null | \
  jq '{
    trades: length,
    total_fee: (map(.commission // 0) | add),
    volume: (map(.lastQty // 0) | add)
  }'
```
