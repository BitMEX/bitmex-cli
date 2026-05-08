---
name: bitmex-fee-optimization
version: 1.0.0
description: "Minimize trading fees on bitmex-cli: maker vs taker, post-only orders, commission tiers, and fee audit."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-types"]
---

# bitmex-fee-optimization

BitMEX charges taker fees and pays maker rebates. The difference is meaningful at scale. Default to limit orders with `ParticipateDoNotInitiate` for passive entries. For aggressive fills where immediate execution matters more than fee savings, use a limit order at the ask/bid price without this flag — see *Avoid Market Orders* below.

## Fee Structure

| Role | Fee Type | Direction |
|---|---|---|
| Maker | Rebate | You earn (negative fee) |
| Taker | Fee | You pay |

Check your current fee schedule (returns a map of symbol → fees):

```bash
# All symbols
bitmex account commission -o json 2>/dev/null

# Single symbol
bitmex account commission -o json 2>/dev/null | \
  jq '.XBTUSD | {makerFee, takerFee, settlementFee}'
```

## Default: Post-Only Limit Orders

`ParticipateDoNotInitiate` rejects the order if it would immediately match (become a taker). This guarantees the maker rebate or no fill at all.

```bash
# Post-only buy — earns rebate or does not fill
bitmex order buy XBTUSD 100 \
  --price 50000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  --validate -o json 2>/dev/null

bitmex order buy XBTUSD 100 \
  --price 50000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null

# Post-only sell
bitmex order sell XBTUSD 100 \
  --price 50500 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null
```

## Avoid Market Orders

Market orders always pay taker fees. Use limit orders at aggressive prices (inside the spread) instead:

```bash
# Instead of market buy, use aggressive limit:
ASK=$(bitmex market orderbook XBTUSD --depth 1 -o json 2>/dev/null | jq '[.[] | select(.side=="Sell") | .price][0]')
bitmex order buy XBTUSD 100 \
  --price "$ASK" \
  --order-type Limit \
  -o json 2>/dev/null
# Note: no ParticipateDoNotInitiate here — we want to take liquidity at ask
```

## ImmediateOrCancel for Partial Maker

Fill at existing prices only, no remainder:

```bash
bitmex order buy XBTUSD 100 \
  --price 50000 \
  --order-type Limit \
  --tif ImmediateOrCancel \
  -o json 2>/dev/null
```

## Volume Tier Review

Higher 30-day trading volume unlocks lower fees:

```bash
# Check your 30-day volume (returns array; first element has the totals)
bitmex account volume -o json 2>/dev/null | \
  jq '.[0] | {advUsd, advUsdContract, advUsdSpot}'

# Cross-reference with commission tiers (returns map of symbol → fees)
bitmex account commission -o json 2>/dev/null | jq '.XBTUSD'
```

## Fee Audit from Trade History

```bash
# Sum fees paid over last 500 executions
bitmex execution trade-history --reverse --count 500 -o json 2>/dev/null | \
  jq '
    {
      total_fee: (map(.commission // 0) | add),
      total_volume: (map(.lastQty // 0) | add),
      trade_count: length,
      avg_fee_per_trade: ((map(.commission // 0) | add) / length | round / 100)
    }
  '
```

## Maker vs Taker Split

```bash
bitmex execution trade-history --reverse --count 500 -o json 2>/dev/null | \
  jq '
    {
      maker_count: (map(select(.commission < 0)) | length),
      taker_count: (map(select(.commission > 0)) | length),
      maker_rebate: (map(select(.commission < 0) | .commission) | add // 0),
      taker_paid: (map(select(.commission > 0) | .commission) | add // 0)
    }
  '
```

## Validate Order Role Before Submission

```bash
# --validate returns the expected execInst, confirming post-only
bitmex order buy XBTUSD 100 \
  --price 50000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  --validate -o json 2>/dev/null | \
  jq '{ordType, execInst, price}'
```

If `execInst` in the validate response does not include `ParticipateDoNotInitiate`, do not submit.
