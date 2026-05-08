---
name: bitmex-recipe-fee-tier-progress
description: Track trading volume and commission rates.
---

# Fee Tier Progress

Check your current maker/taker commission rates, your 30-day trading volume, and calculate how much volume you need to reach the next fee tier.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` and `bc` installed.

## Steps

### 1. Check current commission rates

```bash
# Returns a map of symbol → fees; index by symbol to get rates
bitmex account commission -o json \
  | jq '.XBTUSD | {makerFee, takerFee, settlementFee}'
```

Rates are expressed as decimals (e.g. `-0.00025` = -0.025% maker rebate, `0.00075` = 0.075% taker fee).

### 2. Check 30-day trading volume

```bash
# Returns an array; first element has the totals
bitmex account volume -o json \
  | jq '.[0] | {advUsd, advUsdContract, advUsdSpot}'
```

`advUsd` is the 30-day average daily volume in USD; multiply by 30 for total monthly volume.

```bash
ADV_USD=$(bitmex account volume -o json | jq '.[0].advUsd')
MONTHLY_VOL=$(echo "scale=2; $ADV_USD * 30" | bc)
echo "Estimated 30-day volume: \$${MONTHLY_VOL}"
```

### 3. Reference BitMEX fee tiers

Typical tier thresholds (verify current values at bitmex.com/wallet/fees):

| Tier | 30-day Volume (USD) | Maker    | Taker    |
|------|---------------------|----------|----------|
| 1    | < $1M               | -0.025%  | 0.075%   |
| 2    | $1M – $5M           | -0.025%  | 0.060%   |
| 3    | $5M – $10M          | -0.025%  | 0.050%   |
| 4    | > $10M              | -0.025%  | 0.040%   |

### 4. Calculate distance to next tier

```bash
NEXT_TIER_USD=5000000
GAP=$(echo "scale=2; $NEXT_TIER_USD - $MONTHLY_VOL" | bc)
echo "Volume needed for next tier: \$${GAP}"
```

### 5. Compare maker vs taker cost

On a 100,000 USD trade at current rates:

```bash
TRADE_SIZE=100000
MAKER_RATE=$(bitmex account commission -o json | jq '.XBTUSD.makerFee')
TAKER_RATE=$(bitmex account commission -o json | jq '.XBTUSD.takerFee')

MAKER_COST=$(echo "scale=4; $TRADE_SIZE * $MAKER_RATE" | bc)
TAKER_COST=$(echo "scale=4; $TRADE_SIZE * $TAKER_RATE" | bc)

echo "Maker cost/rebate on \$${TRADE_SIZE}: \$${MAKER_COST}"
echo "Taker cost on \$${TRADE_SIZE}: \$${TAKER_COST}"
```

A negative maker cost means BitMEX pays you a rebate for providing liquidity.

## Notes

- Always prefer limit orders when timing allows — the difference between maker rebate and taker fee compounds significantly on high-frequency strategies.
- Fee tiers reset monthly. Monitor volume in the final week if you are close to a tier boundary.
