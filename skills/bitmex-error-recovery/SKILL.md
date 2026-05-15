---
name: bitmex-error-recovery
version: 1.0.0
description: "Error category handling, duplicate order prevention, retry logic, and partial fill management for bitmex-cli."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-error-recovery

Each error category has a distinct recovery path. Never blindly retry — always inspect category first.

## Error Categories

| Category | Retryable | Action |
|---|---|---|
| `api` | Sometimes | Check `message`; may be invalid params |
| `auth` | No | Check API key permissions and expiry |
| `network` | Yes | Exponential backoff; check if order was placed first |
| `rate_limit` | Yes | Wait until `x-ratelimit-reset`; use `suggestion` field |
| `validation` | No | Fix parameters; use `--validate` to debug |
| `config` | No | Check `~/.config/bitmex/config.toml` |
| `websocket` | Yes | Reconnect with backoff |
| `io` | Sometimes | Disk/pipe issue; check environment |
| `parse` | No | Likely a CLI bug; report |

## Dispatch Pattern

```bash
run_bitmex() {
  RESULT=$(bitmex "$@" -o json 2>/dev/null)
  EXIT=$?
  if [ $EXIT -ne 0 ]; then
    CATEGORY=$(echo "$RESULT" | jq -r '.error // "unknown"')
    case "$CATEGORY" in
      rate_limit)
        WAIT=$(echo "$RESULT" | jq -r '.suggestion // ""' | grep -o '[0-9]*' | head -1)
        sleep "${WAIT:-60}"
        bitmex "$@" -o json 2>/dev/null
        ;;
      network)
        sleep 5
        bitmex "$@" -o json 2>/dev/null
        ;;
      auth)
        echo "Auth error: check BITMEX_API_KEY and BITMEX_API_SECRET" >&2
        return 1
        ;;
      *)
        echo "Error [$CATEGORY]: $(echo "$RESULT" | jq -r '.message')" >&2
        return 1
        ;;
    esac
  fi
  echo "$RESULT"
}
```

## Duplicate Order Prevention

Network errors on order submission leave state uncertain — the order may or may not have been placed. Always check before retrying.

```bash
# Use clOrdID to identify your order
CL_ORD_ID="strategy-dca-$(date +%s)"

# Submit with client order ID
bitmex order buy XBTUSD 100 --price 50000 \
  --cl-ord-id "$CL_ORD_ID" -o json 2>/dev/null

# On network error, check if order exists before retrying
EXISTING=$(bitmex order list --reverse --symbol XBTUSD -o json 2>/dev/null | \
  jq --arg id "$CL_ORD_ID" '[.[] | select(.clOrdID == $id)]')

COUNT=$(echo "$EXISTING" | jq 'length')
if [ "$COUNT" -gt 0 ]; then
  echo "Order already placed, not retrying"
  echo "$EXISTING" | jq '.'
fi
```

## Rate Limit Backoff

```bash
MAX_RETRIES=3
RETRY=0
while [ $RETRY -lt $MAX_RETRIES ]; do
  RESULT=$(bitmex position list -o json 2>/dev/null)
  if [ $? -eq 0 ]; then break; fi
  CATEGORY=$(echo "$RESULT" | jq -r '.error // "unknown"')
  if [ "$CATEGORY" != "rate_limit" ]; then break; fi
  RETRY=$((RETRY + 1))
  sleep $((RETRY * 30))
done
```

## Partial Fill Handling

```bash
ORDER_ID="<id>"
STATUS=$(bitmex order list --reverse --symbol XBTUSD -o json 2>/dev/null | \
  jq -r --arg id "$ORDER_ID" '.[] | select(.orderID == $id) | .ordStatus')
FILLED=$(bitmex order list --reverse --symbol XBTUSD -o json 2>/dev/null | \
  jq -r --arg id "$ORDER_ID" '.[] | select(.orderID == $id) | .cumQty')
REMAINING=$(bitmex order list --reverse --symbol XBTUSD -o json 2>/dev/null | \
  jq -r --arg id "$ORDER_ID" '.[] | select(.orderID == $id) | .leavesQty')

echo "Status: $STATUS | Filled: $FILLED | Remaining: $REMAINING"

# Cancel remainder if partial is sufficient
if [ "$STATUS" = "PartiallyFilled" ]; then
  bitmex order cancel --order-id "$ORDER_ID" -o json 2>/dev/null
fi
```

## 503 Overloaded

BitMEX returns 503 during high-load periods. Treat like `network` error: retry with increasing backoff (5s, 15s, 45s). Do not check order state until service resumes.

## Auth Errors

```bash
# Test auth
RESULT=$(bitmex account me -o json 2>/dev/null)
if echo "$RESULT" | jq -e '.error == "auth"' > /dev/null 2>&1; then
  echo "Invalid or expired API key. Check BITMEX_API_KEY / BITMEX_API_SECRET."
  echo "Verify key has required permissions (order, position, wallet as needed)."
fi
```
