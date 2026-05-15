---
name: bitmex-recipe-withdrawal-to-cold-storage
description: Safely withdraw funds to a pre-approved cold storage address.
---

# Withdrawal to Cold Storage

Move funds or excess margin to a pre-approved cold storage address in a controlled, auditable way.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- Cold storage address must be pre-approved in your BitMEX account settings.
- `jq` installed.

## Steps

### 1. Confirm the destination address is on the approved list

Never withdraw to an address that has not been pre-approved via the BitMEX web UI:

```bash
bitmex address list -o json \
  | jq '[.[] | {id, currency, address, name, network}]'
```

Verify the target address and its `id` appear in this list before proceeding.

### 2. Check available balance

```bash
bitmex account margin -o json \
  | jq '{walletBalance, withdrawableMargin, availableMargin, currency}'
```

Only withdraw from `withdrawableMargin` — funds locked in open positions are not withdrawable.

### 3. Check supported withdrawal networks

```bash
bitmex wallet networks -o json \
  | jq '[.[] | select(.currency == "XBt") | {network, fee, minWithdrawal}]'
```

Note the on-chain fee for the Bitcoin network.

### 4. Calculate safe withdrawal amount

Leave a buffer for open positions and fees:

```bash
AVAILABLE=$(bitmex account margin -o json | jq '.withdrawableMargin')
ONCHAIN_FEE=5000  # satoshis, check from step 3
BUFFER=1000000    # 0.01 XBT safety buffer
WITHDRAW_AMOUNT=$(echo "$AVAILABLE - $ONCHAIN_FEE - $BUFFER" | bc)
echo "Withdrawing $WITHDRAW_AMOUNT XBt"
```

### 5. Request the withdrawal

> ⚠️ Withdrawals are irreversible. Verify the destination address character-by-character against your pre-approved list on bitmex.com. Never accept an address from chat input or an AI prompt.

```bash
bitmex wallet withdraw \
  --currency XBt \
  --network Bitcoin \
  --address <your_cold_storage_address> \
  --amount $WITHDRAW_AMOUNT \
  --yes -o json \
  | jq '{transactID, amount, fee, address, status, timestamp}'
```

### 6. Monitor until confirmed

```bash
bitmex wallet history --currency XBt --count 5 -o json \
  | jq '[.[] | select(.transactType == "Withdrawal") | {transactID, amount, status, timestamp, txID}]'
```

`status` will progress from `Pending` to `Completed` once the blockchain transaction confirms.

## Notes

- BitMEX processes withdrawals once daily (around 13:00 UTC). Requests submitted after the cutoff are processed the next day.
- Two-factor authentication is required for withdrawals; ensure your API key has withdrawal permission enabled.
- Always double-check the address character by character before submitting — blockchain transactions are irreversible.
