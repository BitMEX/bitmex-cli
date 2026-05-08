# Skills Index

49 agent skills for `bitmex-cli`, organized by category.

## Core

Shared runtime contract, safety rules, autonomy progression, and MCP integration.

| Skill | Description |
|-------|-------------|
| [bitmex-shared](./bitmex-shared/SKILL.md) | Shared runtime contract for bitmex-cli: auth, invocation, parsing, and safety. |
| [bitmex-autonomy-levels](./bitmex-autonomy-levels/SKILL.md) | Autonomy progression for agents: from read-only market data to autonomous trading. |
| [bitmex-mcp-integration](./bitmex-mcp-integration/SKILL.md) | MCP client setup: Claude Desktop, Cursor, service filtering, and tool naming. |
| [bitmex-rate-limits](./bitmex-rate-limits/SKILL.md) | REST rate limit handling: 300 req/5min budget, headers, backoff, and WebSocket alternatives. |
| [bitmex-order-types](./bitmex-order-types/SKILL.md) | Order types, execInst modifiers, and TIF options with practical examples. |
| [bitmex-error-recovery](./bitmex-error-recovery/SKILL.md) | Error category handling, duplicate order prevention, retry logic, and partial fill management. |

## Market Data

Instruments, order books, trades, funding rates, alerts, and streaming.

| Skill | Description |
|-------|-------------|
| [bitmex-market-intel](./bitmex-market-intel/SKILL.md) | Market data commands: instruments, order books, trades, candles, funding, and quotes. |
| [bitmex-multi-pair](./bitmex-multi-pair/SKILL.md) | Multi-symbol screening, funding rate comparison, and WebSocket multi-subscribe. |
| [bitmex-alert-patterns](./bitmex-alert-patterns/SKILL.md) | Price, funding, liquidation, and balance alerts using polling and WebSocket. |
| [bitmex-ws-streaming](./bitmex-ws-streaming/SKILL.md) | WebSocket streaming: topics, auth, NDJSON output, and piping to jq. |

## Trading

Order execution, position management, stops, and fee optimization.

| Skill | Description |
|-------|-------------|
| [bitmex-order-execution](./bitmex-order-execution/SKILL.md) | Safe order execution flow: validate, place, monitor, cancel, and dead man's switch. |
| [bitmex-stop-take-profit](./bitmex-stop-take-profit/SKILL.md) | Stop-loss and take-profit orders: Stop, StopLimit, bracket orders, and ReduceOnly. |
| [bitmex-fee-optimization](./bitmex-fee-optimization/SKILL.md) | Minimize trading fees: maker vs taker, post-only orders, commission tiers. |
| [bitmex-position-risk](./bitmex-position-risk/SKILL.md) | Position risk management: leverage, margin, funding costs, risk limits, and close procedures. |
| [bitmex-risk-operations](./bitmex-risk-operations/SKILL.md) | Operational risk controls: pre-flight checks, dead man's switch, mass cancel, emergency close. |
| [bitmex-liquidation-guard](./bitmex-liquidation-guard/SKILL.md) | Liquidation prevention: margin health, emergency flatten, dead man's switch, and real-time monitoring. |

## Testnet

Strategy testing and promotion to live trading.

| Skill | Description |
|-------|-------------|
| [bitmex-testnet-strategy](./bitmex-testnet-strategy/SKILL.md) | Strategy testing on BitMEX testnet: setup, limitations, validation workflow, and PnL review. |
| [bitmex-testnet-to-live](./bitmex-testnet-to-live/SKILL.md) | Promotion from testnet to live: checklist, small-size ramp-up, safety controls, and rollback. |

## Advanced Strategies

Basis trading, funding carry, DCA, grid, rebalancing, and TWAP.

| Skill | Description |
|-------|-------------|
| [bitmex-basis-trading](./bitmex-basis-trading/SKILL.md) | Delta-neutral basis trading between perpetuals and fixed-date futures. |
| [bitmex-funding-carry](./bitmex-funding-carry/SKILL.md) | Earn funding payments on perpetuals: scan rates, entry, monitoring, yield, and exit. |
| [bitmex-dca-strategy](./bitmex-dca-strategy/SKILL.md) | Dollar cost averaging: testnet-first, fixed qty per interval, limit orders, position cap. |
| [bitmex-grid-trading](./bitmex-grid-trading/SKILL.md) | Grid trading on perpetuals: setup, order placement, fill monitoring, and shutdown. |
| [bitmex-rebalancing](./bitmex-rebalancing/SKILL.md) | Rebalancing positions: check allocations, calculate deltas, place orders, validate on testnet. |
| [bitmex-twap-execution](./bitmex-twap-execution/SKILL.md) | TWAP execution for large orders: slicing, rate-limit awareness, fill tracking, and abort. |

## Wallet & Staking

Deposits, withdrawals, transfers, staking, and exports.

| Skill | Description |
|-------|-------------|
| [bitmex-wallet-ops](./bitmex-wallet-ops/SKILL.md) | Wallet operations: balance, deposits, withdrawals, transfers, and supported networks. |
| [bitmex-staking](./bitmex-staking/SKILL.md) | Staking: check status, instruments, tiers, unstake, and pending unstake operations. |
| [bitmex-trade-export](./bitmex-trade-export/SKILL.md) | Export trade, execution, and wallet data: history fetch, date filtering, CSV conversion. |

## Portfolio & Account

Balance analysis, P&L tracking, and subaccount management.

| Skill | Description |
|-------|-------------|
| [bitmex-portfolio-intel](./bitmex-portfolio-intel/SKILL.md) | Portfolio analysis: balance, positions, trade history, margin state, and volume. |
| [bitmex-subaccount-ops](./bitmex-subaccount-ops/SKILL.md) | Subaccount management: create, list transfer accounts, fund allocation. |

## Recipes

Multi-step workflows combining multiple skills.

### Strategy Recipes

| Skill | Description |
|-------|-------------|
| [bitmex-recipe-start-dca-bot](./bitmex-recipe-start-dca-bot/SKILL.md) | Set up and run a DCA bot from testnet validation to live. |
| [bitmex-recipe-launch-grid-bot](./bitmex-recipe-launch-grid-bot/SKILL.md) | Deploy a grid trading bot with testnet validation. |
| [bitmex-recipe-trailing-stop-runner](./bitmex-recipe-trailing-stop-runner/SKILL.md) | Ride a trend with a trailing stop that locks in profits on reversal. |
| [bitmex-recipe-basis-trade-entry](./bitmex-recipe-basis-trade-entry/SKILL.md) | Enter a perp-futures basis trade when premium exceeds threshold. |
| [bitmex-recipe-futures-hedge-spot](./bitmex-recipe-futures-hedge-spot/SKILL.md) | Hedge a long position with a short perpetual. |
| [bitmex-recipe-funding-rate-scan](./bitmex-recipe-funding-rate-scan/SKILL.md) | Scan perpetual contracts for funding rate carry opportunities. |
| [bitmex-recipe-testnet-strategy-backtest](./bitmex-recipe-testnet-strategy-backtest/SKILL.md) | Validate a strategy across multiple testnet sessions before going live. |

### Portfolio Recipes

| Skill | Description |
|-------|-------------|
| [bitmex-recipe-weekly-rebalance](./bitmex-recipe-weekly-rebalance/SKILL.md) | Weekly rebalance to maintain target position allocations. |
| [bitmex-recipe-daily-pnl-report](./bitmex-recipe-daily-pnl-report/SKILL.md) | Daily realised P&L summary from trades and wallet history. |
| [bitmex-recipe-portfolio-snapshot-csv](./bitmex-recipe-portfolio-snapshot-csv/SKILL.md) | Export portfolio snapshot with balances and positions to CSV. |
| [bitmex-recipe-subaccount-capital-rotation](./bitmex-recipe-subaccount-capital-rotation/SKILL.md) | Rotate capital between subaccounts based on strategy performance. |
| [bitmex-recipe-fee-tier-progress](./bitmex-recipe-fee-tier-progress/SKILL.md) | Track trading volume and commission rates. |

### Market Data Recipes

| Skill | Description |
|-------|-------------|
| [bitmex-recipe-morning-market-brief](./bitmex-recipe-morning-market-brief/SKILL.md) | Morning summary: prices, funding, positions, and wallet. |
| [bitmex-recipe-multi-pair-breakout-watch](./bitmex-recipe-multi-pair-breakout-watch/SKILL.md) | Monitor multiple symbols for price breakouts. |
| [bitmex-recipe-track-orderbook-depth](./bitmex-recipe-track-orderbook-depth/SKILL.md) | Monitor order book depth and bid-ask imbalance for liquidity signals. |
| [bitmex-recipe-price-level-alerts](./bitmex-recipe-price-level-alerts/SKILL.md) | Set up price level alerts for key levels. |

### Risk Recipes

| Skill | Description |
|-------|-------------|
| [bitmex-recipe-emergency-flatten](./bitmex-recipe-emergency-flatten/SKILL.md) | Cancel all orders and close all positions immediately. |
| [bitmex-recipe-drawdown-circuit-breaker](./bitmex-recipe-drawdown-circuit-breaker/SKILL.md) | Halt trading when portfolio drawdown exceeds threshold. |

### Funding Recipes

| Skill | Description |
|-------|-------------|
| [bitmex-recipe-withdrawal-to-cold-storage](./bitmex-recipe-withdrawal-to-cold-storage/SKILL.md) | Safely withdraw funds to a pre-approved cold storage address. |
| [bitmex-recipe-staking-yield-compare](./bitmex-recipe-staking-yield-compare/SKILL.md) | Compare staking yields across instruments to find the best rate. |
