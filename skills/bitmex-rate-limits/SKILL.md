---
name: bitmex-rate-limits
version: 1.0.0
description: "BitMEX REST rate limit handling: 300 req/5min budget, headers, backoff, and WebSocket alternatives."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-rate-limits

BitMEX enforces a REST rate limit of 300 requests per 5-minute window. Exceeding the limit returns HTTP 429 and a `rate_limit` error envelope.

## Budget Overview

- Limit: 300 requests per 5 minutes (rolling window)
- Single endpoint — no separate futures budget
- Unauthenticated requests share a per-IP budget
- Authenticated requests use a per-key budget

## Response Headers

Every REST response includes:

```
x-ratelimit-limit: 300
x-ratelimit-remaining: 247
x-ratelimit-reset: 1711900800
```

`x-ratelimit-reset` is a Unix timestamp. When `x-ratelimit-remaining` reaches 0, wait until reset before the next request.

## Rate Limit Error

```json
{
  "error": "rate_limit",
  "message": "Rate limit exceeded",
  "suggestion": "Wait 47 seconds until reset",
  "retryable": true,
  "docs_url": "https://www.bitmex.com/app/restAPI"
}
```

Always check `retryable` before retrying. Check `suggestion` for the recommended wait.

## Handling in Shell

```bash
RESULT=$(bitmex market instrument --active -o json 2>/dev/null)
EXIT=$?

if [ $EXIT -ne 0 ]; then
  CATEGORY=$(echo "$RESULT" | jq -r '.error // "unknown"')
  if [ "$CATEGORY" = "rate_limit" ]; then
    WAIT=$(echo "$RESULT" | jq -r '.suggestion // "Wait 300 seconds"' | grep -o '[0-9]*' | head -1)
    echo "Rate limited. Sleeping ${WAIT}s..."
    sleep "$WAIT"
    # Retry once
    RESULT=$(bitmex market instrument --active -o json 2>/dev/null)
  fi
fi
```

## Budget Estimation

Plan request counts before polling loops:

| Operation | Requests |
|---|---|
| Single instrument fetch | 1 |
| Orderbook + quote + funding | 3 |
| Position + margin + open orders | 3 |
| 10-symbol screening loop | 10–30 |

A polling loop at 1-second intervals will exhaust the 300-request budget in 5 minutes. Use WebSocket for real-time data.

## WebSocket: No Rate Limit

WebSocket subscriptions are not subject to the REST rate limit. Prefer WebSocket for any data that updates faster than once per minute.

```bash
# Real-time trade feed — no REST cost
bitmex ws trade:XBTUSD -o json 2>/dev/null

# Real-time order book — no REST cost
bitmex ws orderBookL2_25:XBTUSD -o json 2>/dev/null

# Authenticated real-time position and margin updates
bitmex ws --auth position margin -o json 2>/dev/null
```

## Recommended Polling Intervals

| Data Type | REST Interval | Prefer WS? |
|---|---|---|
| Instrument info | 60s | No |
| Order book | 5s | Yes |
| Trades | 10s | Yes |
| Funding rate | 300s | No |
| Position / margin | 30s | Yes |
| Open orders | 10s | Yes |

## Burst Strategy

When a strategy needs many symbols at once, batch reads and cache results:

```bash
# Fetch all active instruments once, then filter locally
ALL=$(bitmex market instrument --active -o json 2>/dev/null)
echo "$ALL" | jq '[.[] | select(.rootSymbol == "XBT")]'
echo "$ALL" | jq '[.[] | select(.typ == "FFWCSX")]'  # perps only
```
