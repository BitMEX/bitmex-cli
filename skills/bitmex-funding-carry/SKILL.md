---
name: bitmex-funding-carry
version: 1.0.0
description: "Capture funding rate payments on BitMEX perpetuals: scan rates, entry, monitoring, yield calculation, and exit."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-execution", "bitmex-position-risk"]
---

# bitmex-funding-carry

BitMEX perpetuals pay or receive funding every 8 hours. A positive funding rate means longs pay shorts. A negative rate means shorts pay longs. A funding carry trade holds the position that receives payment.

## How Funding Works

- Funding interval: every 8 hours (00:00, 08:00, 16:00 UTC)
- Rate sign: positive = longs pay shorts; negative = shorts pay longs
- Payment: `funding_cost = position_value * funding_rate`
- For XBT-margined contracts, payment is in XBt (satoshis)

## Step 1: Scan Funding Rates

```bash
# All perps sorted by funding rate (most positive first)
bitmex market instrument --active -o json 2>/dev/null | \
  jq '[.[] | select(.fundingRate != null) | {
    symbol,
    fundingRate,
    annualized_pct: (.fundingRate * 3 * 365 * 100 | round / 100),
    nextFundingTime
  }] | sort_by(.fundingRate) | reverse'

# Historical funding for XBTUSD (last 10 periods)
bitmex market funding --symbol XBTUSD --count 10 --reverse -o json 2>/dev/null | \
  jq '[.[] | {timestamp, fundingRate}]'
```

## Step 2: Assess Entry Threshold

Only enter if the annualized yield exceeds transaction costs and risk premium. A rule of thumb:

```bash
# Annualized carry threshold check (e.g., > 10% annualized)
RATE=$(bitmex market funding --symbol XBTUSD --count 1 --reverse -o json 2>/dev/null | \
  jq -r '.[0].fundingRate // 0')
ANNUALIZED=$(echo "$RATE * 3 * 365 * 100" | bc -l)
echo "Annualized funding yield: ${ANNUALIZED}%"

THRESHOLD=10
if [ "$(echo "$ANNUALIZED > $THRESHOLD" | bc -l)" = "1" ]; then
  echo "Funding attractive — consider short position"
fi
```

## Step 3: Enter Position

If funding is positive (longs pay shorts), take a short to receive payment:

```bash
# Validate first
bitmex order sell XBTUSD 1000 \
  --order-type Limit \
  --price 50000 \
  --exec-inst ParticipateDoNotInitiate \
  --validate -o json 2>/dev/null

# Execute after confirmation
bitmex order sell XBTUSD 1000 \
  --order-type Limit \
  --price 50000 \
  --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null
```

If funding is negative (shorts pay longs), enter long:

```bash
bitmex order buy XBTUSD 1000 \
  --order-type Limit \
  --price 50000 \
  --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null
```

## Step 4: Monitor Ongoing Funding

```bash
# Stream funding updates
bitmex ws funding:XBTUSD -o json 2>/dev/null | \
  jq -c '.data[]? | {symbol, fundingRate, nextFundingTime}'

# Check realised PnL including funding payments
bitmex position list --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {currentQty, realisedPnl, unrealisedPnl}'
```

## Step 5: Exit When Rate Normalises

```bash
# Check if rate has converged toward zero
RATE=$(bitmex market funding --symbol XBTUSD --count 1 --reverse -o json 2>/dev/null | \
  jq -r '.[0].fundingRate // 0')
EXIT_THRESHOLD=0.0001  # < 0.01% per 8h no longer worth holding

if [ "$(echo "${RATE#-} < $EXIT_THRESHOLD" | bc -l)" = "1" ]; then
  echo "Funding normalised at $RATE — exiting position"
  bitmex order close-position XBTUSD -o json 2>/dev/null
fi
```

## Annualized Yield Calculation

> Note: annualized yield is illustrative only. Funding rates change every 8 hours and can flip from payment to cost. Past rates are not indicative of future returns.

```bash
# Per 8h rate → annualized
# 3 payments/day × 365 days = 1095 periods/year
bitmex market funding --symbol XBTUSD --count 30 --reverse -o json 2>/dev/null | \
  jq '
    (map(.fundingRate) | add / length) as $avg |
    {
      avg_rate_per_8h: $avg,
      annualized_pct: ($avg * 1095 * 100 | round / 100)
    }
  '
```

## Risk Considerations

- Directional risk: funding positions carry full mark-to-market PnL
- Funding rate risk: rates can flip, changing from income to cost
- Leverage risk: use low leverage (2–5x) for carry trades
- Liquidation risk: maintain >20% margin buffer at all times
