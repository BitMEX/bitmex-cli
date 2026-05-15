---
name: bitmex-wallet-ops
version: 1.0.0
description: "Wallet operations on bitmex-cli: balance, deposits, withdrawals, transfers, and supported networks."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-wallet-ops

All wallet operations require authentication. Withdrawals and transfers are dangerous — always require explicit user confirmation and use `--validate` where available.

## Check Balance

```bash
# XBt (satoshi) balance
bitmex wallet balance --currency XBt -o json 2>/dev/null | \
  jq '{currency, amount, withdrawableAmount, pendingDebit, pendingCredit}'

# Default currency balance (returns single object, not array)
bitmex wallet balance -o json 2>/dev/null | \
  jq 'select(.amount > 0) | {currency, amount, withdrawableAmount}'

# Wallet summary (returns array of wallet event rows; grab the Total row)
bitmex wallet summary -o json 2>/dev/null | \
  jq '[.[] | select(.transactType == "Total") | {currency, walletBalance, marginBalance, unrealisedPnl}]'
```

## Deposit Address

```bash
# Get deposit address for Bitcoin (--network is required; find valid networks with wallet networks)
bitmex wallet networks -o json 2>/dev/null | jq '[.[] | select(.currency == "XBt") | .network]'
bitmex wallet deposit --currency XBt --network btc -o json 2>/dev/null | \
  jq '{address: .}'
```

## Supported Assets and Networks

```bash
# All supported currencies
bitmex wallet assets -o json 2>/dev/null | \
  jq '[.[] | {currency, name, enabled}]'

# Supported withdrawal networks
bitmex wallet networks -o json 2>/dev/null | \
  jq '[.[] | {currency, network, enabled}]'
```

## Withdrawal

**Dangerous — requires user confirmation. Cannot be undone once submitted.**

```bash
# Step 1: Check supported networks
bitmex wallet networks -o json 2>/dev/null | \
  jq '[.[] | select(.currency == "XBt") | {network, currency, enabled}]'

# Step 2: Validate withdrawal
bitmex wallet withdraw \
  --currency XBt \
  --network btc \
  --address bc1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx \
  --amount 1000000 \
  --validate -o json 2>/dev/null | \
  jq '{currency, network, address, amount, fee}'

# Step 3: Submit (only after explicit user approval)
bitmex wallet withdraw \
  --currency XBt \
  --network Bitcoin \
  --address bc1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx \
  --amount 1000000 \
  -o json 2>/dev/null | \
  jq '{transactID, transactStatus, amount, fee}'
```

## Confirm or Cancel Withdrawal

BitMEX sends an email with a confirmation token for withdrawals:

```bash
# Confirm with token from email
bitmex wallet confirm-withdraw <token> -o json 2>/dev/null

# Cancel before confirmation
bitmex wallet cancel-withdraw <token> -o json 2>/dev/null
```

## Internal Transfer (Between Subaccounts)

```bash
# Check available balance first
bitmex wallet balance --currency XBt -o json 2>/dev/null | \
  jq '{withdrawableAmount}'

# Transfer 500,000 satoshis to a subaccount
bitmex wallet transfer \
  --currency XBt \
  --amount 500000 \
  -o json 2>/dev/null | \
  jq '{transactStatus, amount, currency}'
```

## Wallet Transaction History

```bash
# All transactions
bitmex wallet history --currency XBt -o json 2>/dev/null | \
  jq '[.[] | {timestamp, transactType, amount, fee, address, transactStatus}]'

# Recent deposits only
bitmex wallet history --currency XBt -o json 2>/dev/null | \
  jq '[.[] | select(.transactType == "Deposit") | {timestamp, amount, transactStatus}]'

# Recent withdrawals
bitmex wallet history --currency XBt -o json 2>/dev/null | \
  jq '[.[] | select(.transactType == "Withdrawal") | {timestamp, amount, fee, address, transactStatus}]'
```

## Saved Addresses

```bash
# List saved withdrawal addresses
bitmex address list -o json 2>/dev/null | \
  jq '[.[] | {currency, network, address, name}]'

# Add a new saved address (requires 2FA in practice)
bitmex address add XBt Bitcoin bc1qxxxxxxxxxx -o json 2>/dev/null
```

## Safety Reminders

- Withdrawal amounts are in satoshis (1 BTC = 100,000,000 XBt)
- Network fees are deducted from the withdrawal amount
- Always verify the destination address on a separate device
- BitMEX withdrawal confirmations are sent by email — never skip them
