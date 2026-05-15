---
name: bitmex-recipe-subaccount-capital-rotation
description: Rotate capital between subaccounts based on strategy performance.
---

# Subaccount Capital Rotation

Evaluate performance across multiple BitMEX subaccounts and transfer capital from underperforming strategies to outperforming ones.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set (master account credentials).
- `jq` and `bc` installed.

## Steps

### 1. List transfer-eligible subaccounts

```bash
bitmex subaccount transfer-accounts -o json \
  | jq '[.[] | {id, username, entitlementRoleName}]'
```

Note the `id` for each subaccount you want to evaluate.

### 2. Check wallet balance per subaccount

Use each subaccount's own API credentials, or query from the master account if permitted:

```bash
# wallet balance returns {amount, currency, ...}; use account margin for walletBalance/availableMargin
bitmex wallet balance --currency XBt -o json 2>/dev/null | jq '{currency, amount}'
bitmex account margin --currency XBt -o json 2>/dev/null | jq '{walletBalance, availableMargin, marginBalance}'
```

Record balances across accounts for comparison.

### 3. Evaluate recent performance

Pull 30-day trade history for each account:

```bash
bitmex execution trade-history --reverse --count 500 -o json \
  | jq '
    (map(.realisedPnl // 0) | add) as $total_pnl |
    (map(.commission // 0) | add) as $total_fees |
    {totalPnl: $total_pnl, totalFees: $total_fees, netPnl: ($total_pnl - $total_fees)}
  '
```

Rank accounts by `netPnl`.

### 4. Decide rotation amount

```bash
# Example: rotate 20% of an underperformer's balance to the top performer
SOURCE_BALANCE=50000000  # satoshis
ROTATION_AMOUNT=$(echo "scale=0; $SOURCE_BALANCE * 0.20 / 1" | bc)
echo "Rotating $ROTATION_AMOUNT XBt"
```

### 5. Execute the transfer

```bash
bitmex wallet transfer \
  --currency XBt \
  --amount $ROTATION_AMOUNT \
  --to-user-id <target_subaccount_user_id> \
  --yes -o json \
  | jq '{transferID, amount, currency, status}'
```

### 6. Confirm balances after transfer

```bash
bitmex account margin --currency XBt -o json 2>/dev/null | jq '{walletBalance, availableMargin}'
bitmex wallet history --currency XBt --count 5 -o json 2>/dev/null \
  | jq '[.[] | {timestamp, amount, transactType}]'
```

## Notes

- Transfers settle immediately on BitMEX (no blockchain confirmation required for internal transfers).
- Always leave sufficient margin in source accounts — don't transfer so much that open positions are at liquidation risk.
- Run this workflow monthly or when strategy performance diverges by more than 15%.
