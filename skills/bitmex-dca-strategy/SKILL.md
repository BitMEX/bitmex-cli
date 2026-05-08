---
name: bitmex-dca-strategy
version: 1.0.0
description: "Dollar cost averaging on bitmex-cli: testnet-first, fixed qty per interval, limit orders, and position cap enforcement."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared", "bitmex-order-execution", "bitmex-position-risk", "bitmex-fee-optimization"]
---

# bitmex-dca-strategy

DCA on BitMEX perpetuals: buy a fixed quantity at regular intervals regardless of price, using limit orders to potentially reduce costs with maker orders.

**Always test with `--testnet` first for at least 5 cycles before using real funds.**

## Strategy Parameters

| Parameter | Example | Description |
|---|---|---|
| SYMBOL | XBTUSD | Perpetual contract |
| QTY_PER_INTERVAL | 100 | Contracts per DCA slice |
| INTERVAL_SECONDS | 3600 | 1 hour between buys |
| MAX_POSITION | 5000 | Hard cap on total long exposure |
| STOP_LOSS_PCT | 0.15 | Close if unrealised loss exceeds 15% |

## Testnet First

```bash
export BITMEX_API_KEY="testnet-key"
export BITMEX_API_SECRET="testnet-secret"

# Run 3 testnet cycles to verify logic
for i in 1 2 3; do
  PRICE=$(bitmex --testnet market orderbook XBTUSD --depth 1 -o json 2>/dev/null | \
    jq '[.[] | select(.side == "Buy")] | .[0].price')
  bitmex --testnet order buy XBTUSD 100 --price "$PRICE" \
    --order-type Limit --exec-inst ParticipateDoNotInitiate \
    -o json 2>/dev/null | jq '{orderID, price, ordStatus}'
  sleep 5
done
```

## DCA Loop

```bash
SYMBOL="XBTUSD"
QTY=100
MAX_POSITION=5000
INTERVAL=3600  # seconds

while true; do
  # 1. Check position cap
  CURRENT=$(bitmex position list --symbol $SYMBOL -o json 2>/dev/null | \
    jq -r '.[0].currentQty // 0')
  if [ "$CURRENT" -ge "$MAX_POSITION" ]; then
    echo "Position cap reached ($CURRENT >= $MAX_POSITION), skipping"
    sleep $INTERVAL
    continue
  fi

  # 2. Check margin health
  AVAIL=$(bitmex account margin --currency XBt -o json 2>/dev/null | \
    jq -r '.availableMargin // 0')
  if [ "$AVAIL" -lt 100000 ]; then
    echo "Insufficient margin ($AVAIL sats), skipping"
    sleep $INTERVAL
    continue
  fi

  # 3. Place limit order at best bid
  BID=$(bitmex market orderbook $SYMBOL --depth 1 -o json 2>/dev/null | \
    jq '[.[] | select(.side == "Buy")] | .[0].price')
  echo "DCA buy $QTY $SYMBOL @ $BID"
  bitmex order buy $SYMBOL $QTY \
    --price "$BID" \
    --order-type Limit \
    --exec-inst ParticipateDoNotInitiate \
    -o json 2>/dev/null | jq '{orderID, price, ordStatus}'

  sleep $INTERVAL
done
```

## Track Average Entry Price

> For **inverse** instruments (e.g. XBTUSD), the correct average entry is a cost-weighted harmonic mean: `total_qty / sum(qty/price)`. Standard VWAP (`sum(qty*price) / total_qty`) overstates the average slightly and diverges from the exchange-reported figure.
> For **linear** instruments (e.g. XBTUSDT), standard VWAP is correct.

```bash
# Inverse instruments (XBTUSD, etc.)
bitmex execution trade-history --symbol XBTUSD --reverse --count 100 -o json 2>/dev/null | \
  jq '
    [.[] | select(.side == "Buy" and .execType == "Trade")] |
    {
      count: length,
      total_qty: (map(.lastQty) | add),
      avg_entry: (
        (map(.lastQty) | add) /
        (map(.lastQty / .lastPx) | add)
        | (. * 100 | round) / 100
      )
    }
  '
```

## Stop Loss Enforcement

```bash
STOP_LOSS_PCT=0.15

POS=$(bitmex position list --symbol XBTUSD -o json 2>/dev/null | jq '.[0]')
QTY=$(echo "$POS" | jq -r '.currentQty // 0')
AVG_ENTRY=$(echo "$POS" | jq -r '.avgEntryPrice // 0')
MARK=$(echo "$POS" | jq -r '.markPrice // 0')

if [ "$QTY" -gt 0 ] && [ "$AVG_ENTRY" != "0" ]; then
  LOSS_PCT=$(echo "scale=4; ($AVG_ENTRY - $MARK) / $AVG_ENTRY" | bc -l)
  if [ "$(echo "$LOSS_PCT > $STOP_LOSS_PCT" | bc -l)" = "1" ]; then
    echo "Stop loss triggered: loss $LOSS_PCT > $STOP_LOSS_PCT — closing position"
    bitmex order close-position XBTUSD -o json 2>/dev/null
  fi
fi
```

## Promote to Live

Only after confirming on testnet:
1. Average fill prices are consistent with market
2. Position cap is enforced correctly
3. Stop loss triggers and closes position fully
4. Margin checks prevent over-allocation

Switch by updating env vars to live API keys. Start with `QTY=10` (10% of intended size) for the first live session.
