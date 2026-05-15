#!/usr/bin/env bash
# Paper Trading Workflow
# Demonstrates: init, buy, check status, limit sell, check orders, reset
# Requirements: bitmex (on PATH), jq, bc
set -euo pipefail

PAIR="BTCUSD"
INITIAL_BALANCE=10000
BUY_VOLUME="0.01"

echo "=== Paper Trading Workflow ==="

# Step 1: Initialize
echo "Step 1: Initialize paper account with \$${INITIAL_BALANCE}"
INIT=$(bitmex paper init --balance "$INITIAL_BALANCE" -o json 2>/dev/null)
echo "$INIT" | jq '.'

# Step 2: Get current price
echo "Step 2: Get current $PAIR price"
TICKER=$(bitmex ticker "$PAIR" -o json 2>/dev/null)
PRICE=$(echo "$TICKER" | jq -r ".\"$PAIR\".c[0] // .\"$PAIR\".last // \"unknown\"")
echo "  Current price: \$$PRICE"

# Step 3: Market buy
echo "Step 3: Paper buy $BUY_VOLUME $PAIR at market"
BUY_RESULT=$(bitmex paper buy "$PAIR" "$BUY_VOLUME" -o json 2>/dev/null)
echo "$BUY_RESULT" | jq '.'

# Step 4: Check portfolio
echo "Step 4: Check portfolio status"
bitmex paper status -o json 2>/dev/null | jq '.'

# Step 5: Check balances
echo "Step 5: Check balances"
bitmex paper balance -o json 2>/dev/null | jq '.'

# Step 6: Place limit sell at 5% above market
echo "Step 6: Place limit sell at 5% above market"
if command -v bc >/dev/null 2>&1 && [ "$PRICE" != "unknown" ]; then
  SELL_PRICE=$(echo "$PRICE * 1.05" | bc -l | xargs printf "%.1f")
  echo "  Sell price: \$$SELL_PRICE"
  bitmex paper sell "$PAIR" "0.005" --type limit --price "$SELL_PRICE" -o json 2>/dev/null | jq '.'
else
  echo "  Skipping (bc not available or price unknown)"
fi

# Step 7: Check open orders
echo "Step 7: Check open orders"
bitmex paper orders -o json 2>/dev/null | jq '.'

# Step 8: Trade history
echo "Step 8: Check trade history"
bitmex paper history -o json 2>/dev/null | jq '.'

# Step 9: Reset
echo "Step 9: Reset paper account"
bitmex paper reset -o json 2>/dev/null | jq '.'

echo "=== Workflow complete ==="
