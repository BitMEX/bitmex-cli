---
name: bitmex-subaccount-ops
version: 1.0.0
description: "Subaccount management on bitmex-cli: create, list transfer accounts, fund allocation, and independent accounts."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-wallet-ops"]
---

# bitmex-subaccount-ops

BitMEX supports subaccounts for strategy isolation, risk separation, and multi-strategy fund management. Each subaccount has independent margin.

## Create a Subaccount

Standard subaccount — shares wallet with parent:

```bash
# Create (requires user confirmation)
bitmex subaccount add "strategy-dca" -o json 2>/dev/null | \
  jq '{name, accountId}'
```

Independent subaccount — fully separated balance:

```bash
bitmex subaccount create-independent "strategy-grid" -o json 2>/dev/null | \
  jq '{name, accountId}'
```

## Update Subaccount Name

```bash
bitmex subaccount update <account-id> --account-name "strategy-grid-v2" -o json 2>/dev/null
```

## List Transfer Accounts

Shows all accounts available for internal transfers (main + subaccounts):

```bash
bitmex subaccount transfer-accounts -o json 2>/dev/null | \
  jq '[.[] | {id, username, entitlementRoleName}]'
```

## Fund a Subaccount

Transfer satoshis from main account to a subaccount:

```bash
# Check available balance first
bitmex wallet balance --currency XBt -o json 2>/dev/null | \
  jq '{withdrawableAmount}'

# Transfer 1,000,000 satoshis (0.01 BTC) to subaccount
bitmex wallet transfer \
  --currency XBt \
  --amount 1000000 \
  -o json 2>/dev/null | \
  jq '{transactStatus, amount, currency}'
```

Note: use `--target-account-id` if the CLI supports targeting a specific subaccount.

## Check Subaccount Margin

Each subaccount requires separate API keys. Generate keys per subaccount in the BitMEX UI, then:

```bash
export BITMEX_API_KEY="subaccount-key"
export BITMEX_API_SECRET="subaccount-secret"

bitmex account margin --currency XBt -o json 2>/dev/null | \
  jq '{marginBalance, availableMargin, unrealisedPnl}'

bitmex position list -o json 2>/dev/null | \
  jq '[.[] | select(.isOpen == true) | {symbol, currentQty, unrealisedPnl}]'
```

## Multi-Strategy Allocation Pattern

```bash
# Main account overview
MAIN_BALANCE=$(bitmex wallet balance --currency XBt -o json 2>/dev/null | jq -r '.amount')
echo "Main balance: $MAIN_BALANCE sats"

# List all subaccounts
bitmex subaccount transfer-accounts -o json 2>/dev/null | \
  jq '[.[] | {id, username, entitlementRoleName}]'

# Allocate 30% to each of two strategies
ALLOC=$(echo "$MAIN_BALANCE * 0.30 / 1" | bc)
echo "Allocating $ALLOC sats per strategy"

# Transfer to each (requires confirmation per transfer)
bitmex wallet transfer --currency XBt --amount $ALLOC -o json 2>/dev/null
```

## Affiliate and Commission Info

```bash
# View affiliate stats
bitmex account affiliate -o json 2>/dev/null
```

## Margining Mode

Set the margining mode for the account. Valid values: `REGULAR_MARGIN` or `ISOLATED_MARGIN`.
Current mode is visible via `bitmex account me -o json 2>/dev/null | jq '{selectedMarginingMode, marginingMode}'`.

```bash
# Read current margining mode (non-mutating)
bitmex account me -o json 2>/dev/null | jq '{selectedMarginingMode, marginingMode}'

# Set margining mode (mutating — requires user confirmation)
bitmex account margining-mode REGULAR_MARGIN -o json 2>/dev/null
```
