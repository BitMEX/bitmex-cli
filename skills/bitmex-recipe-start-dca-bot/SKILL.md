---
name: bitmex-recipe-start-dca-bot
description: Set up and run a DCA bot from testnet validation to live.
---

# DCA Bot

Dollar-cost average into a position by placing limit buy orders at a fixed discount to the current market price on a recurring schedule.

Always validate on testnet first: prefix commands with `bitmex --testnet` until the strategy behaves as expected.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set for live.
- `jq` and `bc` installed.
- Configure: `SYMBOL`, `DCA_QTY`, `DISCOUNT_PCT`, `MAX_QTY`, `INTERVAL_SECONDS`.

## Phase 1: Validate on Testnet

```bash
# Get current mid price on testnet
MID=$(bitmex --testnet market instrument --symbol XBTUSD -o json \
  | jq '.[0].lastPrice')

BID=$(echo "scale=0; $MID * (1 - 0.005) / 1" | bc)

bitmex --testnet order buy XBTUSD 10 --order-type Limit \
  --price $BID --validate -o json
```

Run 3–5 test sessions on testnet. Confirm fills appear in:

```bash
bitmex --testnet execution trade-history --count 10 -o json
```

## Phase 2: Go Live at Reduced Size

Start at 10% of intended quantity:

```bash
DCA_QTY=10      # full size would be 100
DISCOUNT_PCT=0.5
MAX_QTY=500
```

## Phase 3: DCA Loop

```bash
while true; do
  # Check current position
  CURRENT_QTY=$(bitmex position list -o json \
    | jq '[.[] | select(.symbol == "XBTUSD")] | .[0].currentQty // 0')

  if (( $(echo "$CURRENT_QTY >= $MAX_QTY" | bc -l) )); then
    echo "Max position $MAX_QTY reached, pausing DCA"
    sleep $INTERVAL_SECONDS
    continue
  fi

  # Check available margin
  MARGIN=$(bitmex account margin -o json | jq '.availableMargin')
  if (( $(echo "$MARGIN < 1000000" | bc -l) )); then
    echo "Low margin, halting DCA"
    exit 1
  fi

  # Place limit buy
  MID=$(bitmex market instrument --symbol XBTUSD -o json | jq '.[0].lastPrice')
  BID=$(echo "scale=0; $MID * (1 - $DISCOUNT_PCT / 100) / 1" | bc)
  bitmex order buy XBTUSD $DCA_QTY --order-type Limit --price $BID --yes -o json \
    | jq '{orderID, price, orderQty}'

  sleep $INTERVAL_SECONDS
done
```

## Safety Checks

- Hard cap on `MAX_QTY` prevents runaway accumulation.
- Margin floor check exits before a margin call can occur.
- Trip the drawdown circuit breaker (see `recipe-drawdown-circuit-breaker`) before each iteration.
