---
name: bitmex-autonomy-levels
version: 1.0.0
description: "Autonomy progression for bitmex-cli agents: from read-only market data to autonomous fund management."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-autonomy-levels

Defines five autonomy levels for BitMEX agent operations. Always start at the lowest applicable level and promote only after explicit user approval.

## Level 1: Read-Only

No credentials required for market data. Account commands require auth but make no changes.

```bash
# Market data — no auth
bitmex market instrument --active -o json 2>/dev/null
bitmex market orderbook XBTUSD --depth 10 -o json 2>/dev/null
bitmex market funding --symbol XBTUSD -o json 2>/dev/null

# Account state — auth required, read-only
bitmex account me -o json 2>/dev/null
bitmex position list -o json 2>/dev/null
bitmex order list --reverse -o json 2>/dev/null
bitmex wallet balance -o json 2>/dev/null
```

Safe for: dashboards, monitoring, reporting, alerting.

## Level 2: Testnet

All commands with `--testnet`. Uses `testnet.bitmex.com`. Requires separate testnet API keys.

```bash
export BITMEX_API_KEY="testnet-key"
export BITMEX_API_SECRET="testnet-secret"

bitmex --testnet order buy XBTUSD 100 --price 50000 --validate -o json 2>/dev/null
bitmex --testnet order buy XBTUSD 100 --price 50000 -o json 2>/dev/null
bitmex --testnet position list -o json 2>/dev/null
bitmex --testnet order cancel-all -o json 2>/dev/null
```

Safe for: strategy development, parameter tuning, onboarding new logic.

## Level 3: Supervised Live

Live credentials. Human confirmation required before every order. Small position sizes. Always validate first.

```bash
# Step 1: validate
bitmex order buy XBTUSD 100 --price 50000 --order-type Limit --validate -o json 2>/dev/null

# Step 2: present to user and wait for explicit "yes"
# Step 3: execute only after confirmation
bitmex order buy XBTUSD 100 --price 50000 --order-type Limit -o json 2>/dev/null

# Confirm fill
bitmex order list --reverse --symbol XBTUSD -o json 2>/dev/null | jq '.[] | {orderID, ordStatus, price, cumQty}'
```

Constraints: max qty per order, no compounding without approval, stop loss required.

## Level 4: Autonomous

> ⚠️ Even at this level, active monitoring is required. Dead man's switch failure, position cap misconfiguration, or network outages can result in unprotected positions or runaway accumulation. You remain responsible for all outcomes.

Pre-approved strategy parameters. Position limits enforced in code. Dead man's switch active on every session.

```bash
# Enable dead man's switch at session start (60 seconds — extend periodically)
bitmex order cancel-after 60000 -o json 2>/dev/null

# Enforce position cap before ordering
CURRENT=$(bitmex position list --symbol XBTUSD -o json 2>/dev/null | jq '.[0].currentQty // 0')
MAX_QTY=1000
if [ "$(echo "$CURRENT >= $MAX_QTY" | bc)" = "1" ]; then
  echo "Position cap reached, skipping order"
  exit 0
fi

# Place order within approved parameters
bitmex order buy XBTUSD 100 --price 50000 --order-type Limit \
  --exec-inst ParticipateDoNotInitiate -o json 2>/dev/null
```

Requirements: strategy approved in writing, position limits in config, automated stop loss, daily PnL alert.

## Level 5: Fund Management

> ⚠️ Even at this level, active monitoring is required. Dead man's switch failure, position cap misconfiguration, or network outages can result in unprotected positions or runaway accumulation. You remain responsible for all outcomes.

Multi-account, risk budget per strategy. Each strategy has an independent margin allocation.

```bash
# Check all subaccount positions
bitmex subaccount transfer-accounts -o json 2>/dev/null

# Transfer capital between accounts
bitmex wallet transfer --currency XBt --amount 100000 -o json 2>/dev/null

# Per-strategy risk budget check
bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '{availableMargin, marginBalance, unrealisedPnl}'

# Commission summary for fee audit
bitmex account commission -o json 2>/dev/null
```

Requirements: multi-sig withdrawal policy, daily reconciliation, independent risk oversight, regulator-compliant audit trail.

## Promotion Checklist

Before advancing a level, confirm:
- [ ] No unexpected positions remain from prior testing
- [ ] Stop loss logic tested and triggered correctly
- [ ] Dead man's switch verified
- [ ] Error recovery tested (network drop, rate limit, auth failure)
- [ ] Maximum position size reviewed by user
