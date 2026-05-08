---
name: bitmex-staking
version: 1.0.0
description: "Staking on bitmex-cli: check status, instruments, tiers, unstake, and pending unstake operations."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-staking

BitMEX offers staking for select assets. Staked assets earn yield based on tier. Unstaking has a lock-up period.

## Check Staking Status

View currently staked positions:

```bash
bitmex staking status -o json 2>/dev/null | \
  jq '[.[] | {
    symbol,
    stakingType,
    stakedAmount,
    currentTier,
    estimatedApr
  }]'
```

## Available Staking Instruments

```bash
bitmex staking instruments -o json 2>/dev/null | \
  jq '[.[] | {
    symbol,
    currency,
    apyBps,
    minSizePerClient,
    maxSizePerClient,
    status,
    expiryTime
  }]'
```

## Reward Tiers

Note: the `/staking/tiers` endpoint returns 404 on current BitMEX mainnet. Use `staking instruments` for APY rates instead (`apyBps` = basis points, divide by 100 for percent).

```bash
# APY from instruments (apyBps / 100 = APY %)
bitmex staking instruments -o json 2>/dev/null | \
  jq '[.[] | {symbol, apyBps, apyPct: (.apyBps / 100)}]'
```

## Estimate Yield

```bash
# Estimate annual yield for a given staked amount (using apyBps from instruments)
STAKED=1000000  # satoshis
RATE_BPS=$(bitmex staking instruments -o json 2>/dev/null | \
  jq -r '[.[] | select(.currency == "BMEx")] | first | .apyBps // 0')
RATE_PCT=$(echo "scale=4; $RATE_BPS / 100" | bc -l)
YIELD=$(echo "scale=4; $STAKED * $RATE_PCT / 100" | bc -l)
echo "Estimated annual yield at ${RATE_PCT}% APR: ${YIELD} units"
```

## Unstake

**Dangerous — initiates an unstaking period. Assets are locked until the period ends.**

```bash
# Check staking instruments before unstaking
bitmex staking instruments -o json 2>/dev/null | \
  jq '[.[] | {symbol, status, expiryTime}]'

# Unstake (requires explicit user confirmation)
bitmex staking unstake XBt 500000 -o json 2>/dev/null | \
  jq '{symbol, amount, unstakingEndDate, status}'
```

## Pending Unstake

Note: `bitmex staking pending-unstake` currently returns an API error on mainnet (the endpoint requires a `status` query parameter that the CLI does not yet pass). Use `bitmex staking status` to see current staking positions instead.

```bash
# View all pending unstake requests (may return API error on mainnet)
bitmex staking pending-unstake -o json 2>/dev/null
```

## Cancel Unstake

If you change your mind before the unstaking period ends:

```bash
# List pending unstakes to get ID (may return API error on mainnet — see Pending Unstake note above)
bitmex staking pending-unstake -o json 2>/dev/null

# Cancel a specific unstake request
bitmex staking cancel-unstake <id> -o json 2>/dev/null | \
  jq '{status, symbol, amount}'
```

## Staking Workflow

```bash
# 1. Check what's available and what you already have staked
bitmex staking instruments -o json 2>/dev/null | jq '[.[] | select(.status == "ACTIVE")]'
bitmex staking status -o json 2>/dev/null

# 2. Review APY rates from instruments (staking tiers endpoint is not available)
bitmex staking instruments -o json 2>/dev/null | jq '[.[] | {symbol, apyBps, apyPct: (.apyBps / 100)}]'

# 3. Check wallet balance before unstaking
bitmex wallet balance -o json 2>/dev/null | \
  jq '{amount, currency}'

# 4. Unstake (after user approval)
bitmex staking unstake XBt 500000 -o json 2>/dev/null

# 5. Monitor unstaking completion (may return API error on mainnet — see Pending Unstake note above)
bitmex staking pending-unstake -o json 2>/dev/null
```
