---
name: bitmex-recipe-launch-grid-bot
description: Deploy a grid trading bot with testnet validation.
---

# Grid Trading Bot

Place a ladder of limit orders above and below the current price. Buy orders fill on dips; sell orders fill on rallies. Each filled buy is replaced with a sell one grid level higher, and vice versa.

Always validate on testnet first: prefix commands with `bitmex --testnet` until the strategy behaves as expected.

## Prerequisites

- `BITMEX_API_KEY` and `BITMEX_API_SECRET` set.
- `jq` and `bc` installed.
- Configure: `SYMBOL`, `GRID_LEVELS` (e.g. 5), `GRID_SPACING_PCT` (e.g. 0.5), `QTY_PER_LEVEL`.

## Phase 1: Testnet Validation

```bash
MID=$(bitmex --testnet market instrument --symbol XBTUSD -o json | jq '.[0].lastPrice')

for i in 1 2 3 4 5; do
  BUY_PRICE=$(echo "scale=0; $MID * (1 - $i * 0.005) / 1" | bc)
  SELL_PRICE=$(echo "scale=0; $MID * (1 + $i * 0.005) / 1" | bc)

  bitmex --testnet order buy XBTUSD 10 --order-type Limit --price $BUY_PRICE --yes -o json \
    | jq '{side: "Buy", price, orderID}'

  bitmex --testnet order sell XBTUSD 10 --order-type Limit --price $SELL_PRICE --yes -o json \
    | jq '{side: "Sell", price, orderID}'
done
```

Verify the full grid appears in:

```bash
bitmex --testnet order list -o json | jq '[.[] | {side, price, orderQty, ordStatus}]'
```

Also confirm the order book:

```bash
bitmex --testnet market orderbook XBTUSD --depth 10 -o json
```

## Phase 2: Live Deployment

Replace `--testnet` with live credentials and repeat the loop above.

## Phase 3: Monitor and Replace Filled Levels

```bash
bitmex ws --auth execution -o json | jq 'select(.data[].ordStatus == "Filled") | .data[]
  | {symbol, side, price, orderQty}'
```

When a buy fills at level N, place a new sell at level N+1. When a sell fills, place a new buy at level N-1.

## Shutdown

Cancel the entire grid cleanly:

```bash
bitmex order cancel-all --yes -o json
```

## Notes

- Grids perform best in ranging markets. In strong trends, one side fills completely while the other side sits idle.
- Start with `QTY_PER_LEVEL` equal to 1% of available margin per level.
