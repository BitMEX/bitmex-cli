---
name: bitmex-recipe-portfolio-snapshot-csv
description: Export portfolio snapshot with balances and positions to CSV.
---

# Portfolio Snapshot to CSV

Export a timestamped CSV of wallet balances and open positions for record-keeping, audit, or external analysis.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` installed.

## Steps

### 1. Set output paths

```bash
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
OUTDIR="$HOME/bitmex-snapshots"
mkdir -p "$OUTDIR"
WALLET_CSV="$OUTDIR/wallet_${TIMESTAMP}.csv"
POSITIONS_CSV="$OUTDIR/positions_${TIMESTAMP}.csv"
COMBINED_CSV="$OUTDIR/snapshot_${TIMESTAMP}.csv"
```

### 2. Export wallet balances

```bash
# wallet balance returns a single object (not array); use account margin for walletBalance/availableMargin
echo "timestamp,currency,amount,walletBalance,availableMargin,marginBalance" > "$WALLET_CSV"
MARGIN=$(bitmex account margin --currency XBt -o json 2>/dev/null)
bitmex wallet balance --currency XBt -o json 2>/dev/null \
  | jq -r --arg ts "$TIMESTAMP" \
      --argjson m "$MARGIN" \
      '[$ts, .currency, (.amount|tostring),
        ($m.walletBalance|tostring),
        ($m.availableMargin|tostring),
        ($m.marginBalance|tostring)] | @csv' \
  >> "$WALLET_CSV"
```

### 3. Export open positions

```bash
echo "timestamp,symbol,currentQty,avgEntryPrice,markPrice,unrealisedPnl,liquidationPrice,leverage" \
  > "$POSITIONS_CSV"
bitmex position list -o json \
  | jq -r --arg ts "$TIMESTAMP" \
      '.[] | [$ts, .symbol, (.currentQty|tostring), (.avgEntryPrice|tostring),
              (.markPrice|tostring), (.unrealisedPnl|tostring),
              (.liquidationPrice|tostring), (.leverage|tostring)] | @csv' \
  >> "$POSITIONS_CSV"
```

### 4. Combine into a single snapshot file

```bash
{
  echo "=== WALLET ==="
  cat "$WALLET_CSV"
  echo ""
  echo "=== POSITIONS ==="
  cat "$POSITIONS_CSV"
} > "$COMBINED_CSV"

echo "Snapshot saved: $COMBINED_CSV"
```

### 5. Optional: append to a running daily ledger

```bash
DAILY="$OUTDIR/daily_$(date -u +%Y-%m-%d).csv"
cat "$POSITIONS_CSV" >> "$DAILY"
```

## Tips

- Schedule via cron every hour: `0 * * * * /path/to/snapshot.sh`.
- Import the daily CSV into a spreadsheet to graph equity curve over time.
- BitMEX balances are in satoshis (XBt); divide by `100000000` in your spreadsheet for XBT.
