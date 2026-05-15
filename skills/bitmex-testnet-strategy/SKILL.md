---
name: bitmex-testnet-strategy
version: 1.0.0
description: "Strategy testing on BitMEX testnet: setup, limitations, validation workflow, and PnL review."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-testnet-strategy

BitMEX testnet (`testnet.bitmex.com`) is the correct environment for strategy development and validation. Use `--testnet` on every command. **There is no paper trading mode — testnet is the equivalent.**

## Setup

1. Register at `https://testnet.bitmex.com`
2. Create API keys at `https://testnet.bitmex.com/app/apiKeys`
3. Set env vars:

```bash
export BITMEX_API_KEY="testnet-api-key"
export BITMEX_API_SECRET="testnet-api-secret"
```

Or use the config file:

```bash
bitmex --testnet auth set --api-key <key> --api-secret <secret>
```

## Basic Connectivity Test

```bash
bitmex --testnet market instrument --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {symbol, lastPrice, markPrice}'

bitmex --testnet account me -o json 2>/dev/null | \
  jq '{username, id}'
```

## Place Test Orders

```bash
# Validate first
bitmex --testnet order buy XBTUSD 100 \
  --price 50000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  --validate -o json 2>/dev/null

# Place
bitmex --testnet order buy XBTUSD 100 \
  --price 50000 \
  --order-type Limit \
  --exec-inst ParticipateDoNotInitiate \
  -o json 2>/dev/null | jq '{orderID, ordStatus, price, ordQty}'
```

## Monitor Testnet Positions

```bash
bitmex --testnet position list --symbol XBTUSD -o json 2>/dev/null | \
  jq '.[0] | {currentQty, avgEntryPrice, markPrice, unrealisedPnl, liquidationPrice}'
```

## Test Stop Loss Trigger

```bash
# Enter long position
bitmex --testnet order buy XBTUSD 100 --order-type Market -o json 2>/dev/null

# Place stop immediately
bitmex --testnet order sell XBTUSD 100 \
  --order-type Stop --stop-px 45000 \
  --exec-inst ReduceOnly,MarkPrice \
  -o json 2>/dev/null

# Verify stop is in open orders
bitmex --testnet order list --symbol XBTUSD -o json 2>/dev/null | \
  jq '[.[] | select(.ordType == "Stop") | {orderID, stopPx, execInst}]'
```

## Test Dead Man's Switch

```bash
# Enable DMS
bitmex --testnet order cancel-after 10000 -o json 2>/dev/null
# Wait 10 seconds without renewing
sleep 15
# Verify orders were cancelled
bitmex --testnet order list --symbol XBTUSD -o json 2>/dev/null | \
  jq '[.[] | select(.ordStatus == "New")] | length'
# Expected: 0
```

## Testnet Limitations

| Limitation | Detail |
|---|---|
| Prices may diverge | Testnet prices are independent of mainnet |
| Periodic resets | Testnet balances and positions are reset periodically |
| Liquidity differs | Order fills may behave differently than production |
| WebSocket differences | Some testnet WebSocket events may lag |

## PnL Review After Testing

```bash
bitmex --testnet execution trade-history --count 100 -o json 2>/dev/null | \
  jq '
    {
      trades: length,
      realised_pnl: (map(.realisedPnl // 0) | add),
      total_fees: (map(.commission // 0) | add),
      win_count: (map(select((.realisedPnl // 0) > 0)) | length),
      loss_count: (map(select((.realisedPnl // 0) < 0)) | length)
    }
  '
```

## Validation Checklist Before Promoting to Live

- [ ] Strategy ran for 5+ complete cycles on testnet
- [ ] Stop losses triggered correctly and closed position fully
- [ ] Dead man's switch verified to cancel orders on timeout
- [ ] Average fill prices are within expected slippage
- [ ] No unexpected open positions left after session end
- [ ] Error recovery logic tested (simulate network error by killing process)
- [ ] Position cap enforced — orders rejected when limit reached
