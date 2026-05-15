---
name: bitmex-recipe-funding-rate-scan
description: Scan perpetual contracts for funding rate carry opportunities.
---

# Funding Rate Scan

Perpetual contracts charge a funding payment every 8 hours between longs and shorts. When the rate is extreme, the paying side subsidizes the receiving side — this creates a carry trade opportunity.

## Prerequisites

- No API key required for market data.
- `jq` and `bc` installed.

## Steps

### 1. Get all active perpetuals with current funding rate

```bash
bitmex market instrument --active -o json \
  | jq '[.[] | select(.typ == "FFWCSX") | {symbol, fundingRate, markPrice, volume24h}]'
```

### 2. Annualize the funding rate

BitMEX pays funding every 8 hours (3 times per day, 1095 times per year):

```bash
bitmex market instrument --active -o json \
  | jq '[.[] | select(.typ == "FFWCSX") | {
      symbol,
      fundingRate,
      annualizedPct: (.fundingRate * 3 * 365 * 100 | . * 100 | round / 100)
    }] | sort_by(.annualizedPct) | reverse'
```

### 3. Pull recent funding history (3 intervals) per symbol

```bash
bitmex market funding --symbol XBTUSD --count 3 --reverse -o json \
  | jq '[.[] | {symbol, fundingRate, timestamp}]'
```

Repeat for high-rate symbols identified in step 2.

### 4. Rank opportunities

Symbols with `annualizedPct` above **+54.75%** (0.05% per 8 h) are strong positive carry candidates — short that perp and collect funding. Negative rates below **-54.75%** favor longs.

### 5. Entry decision

```bash
# Example: short high-funding perp (validate first)
bitmex order sell XBTUSD 100 --order-type Limit \
  --price <markPrice * 1.0001> --validate -o json
```

## Notes

- Cross-check volume in step 1: avoid illiquid symbols where the spread erodes carry.
- Funding rates shift rapidly near settlement (top of each 8 h window). Re-scan every 30 minutes.
- Combine with a delta hedge on spot to isolate the carry without directional risk.
