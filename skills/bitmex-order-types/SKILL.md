---
name: bitmex-order-types
version: 1.0.0
description: "BitMEX order types, execInst modifiers, and TIF options with practical examples."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-order-types

BitMEX supports seven order types, several `execInst` modifiers, and four time-in-force values. Always use `--validate` before live submission.

## Order Types

### Limit (default)

Posts to the order book. Earns maker rebate when filled passively.

```bash
bitmex order buy XBTUSD 100 --price 50000 --order-type Limit --validate -o json 2>/dev/null
bitmex order buy XBTUSD 100 --price 50000 --order-type Limit -o json 2>/dev/null
```

### Market

Fills immediately at best available price. Pays taker fee. Avoid when possible.

```bash
bitmex order buy XBTUSD 100 --order-type Market --validate -o json 2>/dev/null
```

### Stop

Triggered market order. Submits a market order when `stopPx` is reached.

```bash
# Stop loss: sell if price drops to 48000
bitmex order sell XBTUSD 100 --order-type Stop --stop-px 48000 \
  --exec-inst ReduceOnly --validate -o json 2>/dev/null
```

### StopLimit

Triggered limit order. When `stopPx` is touched, places a limit at `--price`.

```bash
bitmex order sell XBTUSD 100 --order-type StopLimit \
  --stop-px 48000 --price 47900 --exec-inst ReduceOnly \
  --validate -o json 2>/dev/null
```

### MarketIfTouched

Triggered market order when price moves to `stopPx` (in the favorable direction).

```bash
# Buy if price drops to 49000 (entry on dip)
bitmex order buy XBTUSD 100 --order-type MarketIfTouched --stop-px 49000 \
  --validate -o json 2>/dev/null
```

### LimitIfTouched

Triggered limit order when price touches `stopPx`.

```bash
bitmex order buy XBTUSD 100 --order-type LimitIfTouched \
  --stop-px 49000 --price 49100 --validate -o json 2>/dev/null
```

### Pegged

Tracks best bid or ask with an offset. Used for market-making.

> **Note:** The CLI does not currently expose `--peg-offset-value` or `--peg-price-type` flags.
> Pegged orders must be submitted via the REST API directly or via `bitmex order buy --order-type Pegged` without peg parameters (which will be rejected by the exchange until support is added).

```bash
# Validate that Pegged order type is accepted by the CLI (peg params set server-side or via raw API)
bitmex order buy XBTUSD 100 --order-type Pegged \
  --validate -o json 2>/dev/null
```

## execInst Modifiers

Multiple modifiers can be combined with commas.

| Modifier | Effect |
|---|---|
| `ParticipateDoNotInitiate` | Post-only: cancel if would take |
| `ReduceOnly` | Never increase position size |
| `Close` | Close entire position |
| `LastPrice` | Trigger based on last trade price |
| `IndexPrice` | Trigger based on index price |
| `MarkPrice` | Trigger based on mark price (default for stops) |

```bash
# Post-only limit (maker rebate guaranteed or cancel)
bitmex order buy XBTUSD 100 --price 50000 --order-type Limit \
  --exec-inst ParticipateDoNotInitiate -o json 2>/dev/null

# ReduceOnly stop: cannot flip position
bitmex order sell XBTUSD 100 --order-type Stop --stop-px 48000 \
  --exec-inst ReduceOnly,MarkPrice -o json 2>/dev/null
```

## Time-In-Force

| TIF | Behavior |
|---|---|
| `GoodTillCancel` | Default; remains open until filled or cancelled |
| `ImmediateOrCancel` | Fill whatever is available immediately, cancel rest |
| `FillOrKill` | Fill entirely or cancel entirely |
| `Day` | Expires at end of trading session |

```bash
bitmex order buy XBTUSD 100 --price 50000 --order-type Limit \
  --tif ImmediateOrCancel --validate -o json 2>/dev/null
```

## Contingency / Linked Orders

BitMEX supports paired order logic via `contingencyType` and a shared `clOrdLinkID`. The CLI does not currently expose these as flags; use the REST API for these order types.

| Type | Behavior |
|---|---|
| `OneCancelsTheOther` | OCO: when one order fills or triggers, the linked order is cancelled |
| `OneTriggersTheOther` | OTO: when the first order fills, the linked order becomes active |
| `OneUpdatesTheOtherAbsolute` | OUO: amending one order propagates the change to the linked order |

Typical use: a stop-loss + take-profit pair as OCO so that one filling automatically cancels the other. These must currently be submitted directly via the BitMEX REST API with matching `clOrdLinkID` values.

## Amending Orders

Change price or qty on an open order without cancelling and re-placing:

```bash
ORDER_ID=$(bitmex order list --reverse --symbol XBTUSD -o json 2>/dev/null | \
  jq -r '[.[] | select(.ordStatus=="New")][0].orderID')

bitmex order amend --order-id "$ORDER_ID" --price 50500 -o json 2>/dev/null
bitmex order amend --order-id "$ORDER_ID" --qty 200 -o json 2>/dev/null
```
