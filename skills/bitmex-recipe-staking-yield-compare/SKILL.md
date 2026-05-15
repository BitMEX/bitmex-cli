---
name: bitmex-recipe-staking-yield-compare
description: Compare staking yields across instruments to find the best rate.
---

# Staking Yield Comparison

Survey available BitMEX staking instruments, compare APY across assets and tiers, then rebalance staked positions toward the highest-yielding options.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` installed.

## Steps

### 1. List all available staking instruments

```bash
# Actual fields: symbol, currency, apyBps, minSizePerClient, maxSizePerClient, status, expiryTime
bitmex staking instruments -o json \
  | jq '[.[] | {symbol, currency, apyBps, apyPct: (.apyBps / 100), minSizePerClient, status}]'
```

`apyBps` is basis points (100 bps = 1%). `status` is "ACTIVE" or "INACTIVE".

### 2. List yield tiers

Note: `bitmex staking tiers` returns 404 on mainnet. Use `apyBps` from instruments instead.

```bash
bitmex staking instruments -o json \
  | jq '[.[] | {symbol, apyBps, apyPct: (.apyBps / 100)}]'
```

### 3. Determine which tier you qualify for

```bash
# wallet balance is a single object (not an array)
BALANCE=$(bitmex wallet balance -o json | jq '.amount')

bitmex staking instruments -o json \
  | jq --argjson bal "$BALANCE" \
      '[.[] | select(.minSizePerClient <= $bal) | {symbol, apyBps, apyPct: (.apyBps / 100), minSizePerClient}]
       | sort_by(.apyBps) | reverse | .[0]'
```

### 4. Check current staking positions

```bash
bitmex staking status -o json \
  | jq '[.[] | {symbol, stakedAmount}]'
```

### 5. Compare and identify the best opportunity

```bash
bitmex staking instruments -o json \
  | jq '[.[] | {symbol, apyBps, apyPct: (.apyBps / 100)}] | sort_by(.apyBps) | reverse | .[0:3]'
```

### 6. Unstake lower-yield (no `stake` subcommand exists — only `unstake`)

Note: `bitmex staking pending-unstake` returns an API error on mainnet (the endpoint requires a `status` param the CLI doesn't pass). Skip or ignore errors from that step.

```bash
# Check pending unstakes (may return API error on mainnet)
bitmex staking pending-unstake -o json

# Unstake (positional args: SYMBOL AMOUNT)
bitmex staking unstake <low_yield_symbol> <qty> --yes -o json
```

## Notes

- Re-run this comparison weekly — APRs change as liquidity pools are rebalanced.
- Check `lockPeriodDays` before unstaking: early withdrawal may forfeit accrued yield.
- Factor in the opportunity cost of locked capital against open trading margin requirements.
