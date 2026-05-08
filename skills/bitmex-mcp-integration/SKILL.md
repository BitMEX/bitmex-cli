---
name: bitmex-mcp-integration
version: 1.0.0
description: "MCP client setup for bitmex-cli: Claude Desktop, Cursor, service filtering, and tool naming."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bitmex"]
  depends: ["bitmex-shared"]
---

# bitmex-mcp-integration

Configures `bitmex-cli` as an MCP tool server for Claude Desktop, Cursor, and other MCP clients.

## Claude Desktop Config

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "bitmex": {
      "command": "bitmex",
      "args": ["mcp"],
      "env": {
        "BITMEX_API_KEY": "your-key",
        "BITMEX_API_SECRET": "your-secret"
      }
    }
  }
}
```

For testnet, add `"--testnet"` to `args`:

```json
{
  "mcpServers": {
    "bitmex-testnet": {
      "command": "bitmex",
      "args": ["mcp", "--testnet"],
      "env": {
        "BITMEX_API_KEY": "testnet-key",
        "BITMEX_API_SECRET": "testnet-secret"
      }
    }
  }
}
```

## Cursor Config

Edit `.cursor/mcp.json` in your project root:

```json
{
  "mcpServers": {
    "bitmex": {
      "command": "bitmex",
      "args": ["mcp", "-s", "market,account,order,position,wallet"],
      "env": {
        "BITMEX_API_KEY": "${env:BITMEX_API_KEY}",
        "BITMEX_API_SECRET": "${env:BITMEX_API_SECRET}"
      }
    }
  }
}
```

## Service Filtering

Restrict which command groups are exposed as MCP tools using `-s`:

```bash
# Read-only: market data and account state only
bitmex mcp -s market,account,position,wallet

# Trading: add order management
bitmex mcp -s market,account,order,position,wallet

# Full access (requires --allow-dangerous)
bitmex mcp -s market,account,order,position,wallet,staking,subaccount \
  --allow-dangerous
```

Recommended for most agents: `market,account,position` (no order placement without explicit upgrade).

## Dangerous Tool Exposure

By default, order placement and withdrawal tools are hidden. To expose them:

```bash
bitmex mcp --allow-dangerous -s market,account,order,position,wallet
```

Always combine `--allow-dangerous` with a system prompt that requires user confirmation before calling any order or withdrawal tool.

## Tool Naming Convention

MCP tools follow the pattern `bitmex_<group>_<subcommand>`:

| CLI Command | MCP Tool Name |
|---|---|
| `bitmex market orderbook XBTUSD` | `bitmex_market_orderbook` |
| `bitmex market funding` | `bitmex_market_funding` |
| `bitmex order buy XBTUSD 100` | `bitmex_order_buy` |
| `bitmex order cancel` | `bitmex_order_cancel` |
| `bitmex position list` | `bitmex_position_list` |
| `bitmex position leverage` | `bitmex_position_leverage` |
| `bitmex wallet balance` | `bitmex_wallet_balance` |
| `bitmex account margin` | `bitmex_account_margin` |

## Verifying the MCP Server

```bash
# Start the MCP server in stdio mode and confirm it launches without error
bitmex mcp -s market,account 2>/dev/null &
MCP_PID=$!
sleep 1
kill $MCP_PID 2>/dev/null

# To test a tool call, connect via an MCP client (e.g. Claude Desktop) or use the --port HTTP mode:
# bitmex mcp --port 3000 --token mytoken -s market,account &
# curl -s -H "Authorization: Bearer mytoken" \
#   -H "Content-Type: application/json" \
#   -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"bitmex_market_instrument","arguments":{"symbol":"XBTUSD"}}}' \
#   http://localhost:3000/mcp
```

## System Prompt Guidance

When using the MCP server in an agent, include in the system prompt:

> Before calling any `bitmex_order_*` or `bitmex_wallet_withdraw` tool, you MUST present the full parameters to the user and receive explicit written approval. Never use `--allow-dangerous` tools autonomously.
