# bitmex-cli Skills

49 goal-oriented skill packages for AI agents that operate `bitmex-cli`.

See the full [Skills Index](INDEX.md) for the complete categorized list.

## Skill Types

- **Core** (6): Shared contract, autonomy levels, MCP integration, rate limits, order types, error recovery
- **Market Data** (4): Instruments, multi-pair screening, alerts, WebSocket streaming
- **Trading** (6): Order execution, stops, fees, position risk, risk operations, liquidation guard
- **Testnet** (2): Strategy testing, testnet-to-live promotion
- **Advanced Strategies** (6): Basis trading, funding carry, DCA, grid, rebalancing, TWAP
- **Wallet & Staking** (3): Deposits/withdrawals, staking, trade exports
- **Portfolio & Account** (2): Balance analysis, subaccount management
- **Recipes** (20): Multi-step workflows combining multiple skills

## OpenClaw Setup

From the repository root, symlink all skills:

```bash
cd /path/to/bitmex-cli
for skill in skills/bitmex-* skills/recipe-*; do
  ln -s "$(pwd)/$skill" ~/.openclaw/skills/
done
```

Install specific skills:

```bash
ln -s "$(pwd)/skills/bitmex-shared" ~/.openclaw/skills/
ln -s "$(pwd)/skills/recipe-morning-market-brief" ~/.openclaw/skills/
```

## Claude Desktop / MCP

Skills are loaded automatically when using the MCP server:

```bash
bitmex mcp -s market,account,order,position,wallet
```

See [bitmex-mcp-integration](./bitmex-mcp-integration/SKILL.md) for full configuration.

## Skill Structure

Each skill directory contains:

```
skills/bitmex-<name>/
  SKILL.md       # Frontmatter + full skill content
```

Frontmatter fields: `name`, `version`, `description`, `dependencies` (optional).
