# Disclaimer

## Experimental Software

This software is experimental and under active development. It is provided "AS IS" without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose, and non-infringement. Use it at your own risk.

## Not Financial Advice

This tool is a command-line interface for the BitMEX cryptocurrency derivatives exchange. It does not provide financial advice, trading recommendations, or investment guidance. It executes commands exactly as instructed. No information provided by this tool, including market data, funding rates, or strategy workflows, constitutes a recommendation to buy, sell, or hold any financial instrument.

Use of this tool does not create an advisory, fiduciary, or client relationship between you and HDR Global Trading Limited or any of its affiliates.

## Real Money, Real Risk

Commands executed through this CLI interact with the live BitMEX exchange and can result in real financial transactions. Orders, withdrawals, and transfers are irreversible once processed by the exchange. Incorrect commands, software bugs, network failures, or agent errors can result in financial loss, including the total loss of deposited funds.

## API Key Security

Your API key and secret grant access to your BitMEX account. Treat them like passwords:
- Never share them in public repositories, logs, or chat messages.
- Never pass `--api-secret` on the command line in shared environments (use environment variables or `--api-secret-stdin`).
- Rotate keys regularly.
- Use the most restrictive permissions possible for your use case.

Before using this tool with real funds:

- Test your workflows using `--testnet`, which targets the BitMEX testnet with no real money at risk.
- Validate orders with `--validate` before submitting.
- Use restricted API keys with only the permissions you need.
- Start with small amounts.

## BitMEX Terms of Service

Use of this tool to interact with the BitMEX exchange is subject to the [BitMEX Terms of Service](https://www.bitmex.com/terms). BitMEX services are not available in all jurisdictions. You are solely responsible for determining whether your use of this tool and the BitMEX exchange complies with the laws and regulations applicable in your jurisdiction.

## AI Agent and Automated Use

When used by AI agents, large language models, or other automated systems, the same risks apply — and may be amplified. AI agents may misinterpret instructions, hallucinate parameters, or take actions that differ from your intent. The CLI executes commands as received; neither the CLI nor any AI agent validates whether a trade is financially sound.

If you grant an AI agent access to your API credentials:

- **You are responsible for all actions the agent takes on your behalf, regardless of whether those actions match your intent.**
- The agent can place orders, cancel orders, and (if permitted by the API key) withdraw funds.
- AI agents are third-party software not developed, controlled, or endorsed by HDR Global Trading Limited. We make no representations about the behaviour, reliability, or safety of any AI agent.
- Use the `dangerous` field in `agents/tool-catalog.json` to identify high-risk commands.
- Use BitMEX's API key permission system to restrict what the agent can do.
- Enable the dead man's switch (`bitmex order cancel-after`) for unattended or autonomous sessions.
- Autonomous operation without active human monitoring significantly increases risk. Dead man's switch failures, position limit misconfiguration, or network outages can result in unprotected positions or runaway order accumulation.

## Limitation of Liability

To the maximum extent permitted by applicable law, in no event shall HDR Global Trading Limited, its affiliates, or the authors and contributors of this software be liable for any direct, indirect, incidental, special, consequential, or exemplary damages (including but not limited to loss of profits, loss of data, loss of funds, business interruption, or procurement of substitute services) arising out of or in connection with the use of or inability to use this software, whether based in contract, tort, negligence, strict liability, or any other legal theory, even if advised of the possibility of such damages.

This limitation applies whether the software is used manually or by an automated agent.

## Indemnification

By using this software, you agree to indemnify, defend, and hold harmless HDR Global Trading Limited, its affiliates, officers, directors, employees, and contributors from and against any claims, liabilities, damages, losses, and expenses (including reasonable legal fees) arising out of or in any way connected with your use of this software or any actions taken through it, whether by you or by an automated agent acting on your behalf.

## Data Handling and Credential Security

This CLI processes API credentials locally on your machine. Depending on how you configure the CLI, API secrets may be stored in your operating system's native keychain where available, while API keys may be stored in a local configuration file. Credentials may also be provided through environment variables, stdin, or command-line flags and may not be persisted by the CLI. By default, the CLI transmits credentials only to `*.bitmex.com` endpoints for the purpose of authenticating your requests. If you override the API base URL (for example with `--api-url` or `BITMEX_API_URL`), credentials may be transmitted to that configured host instead. No credentials are intentionally transmitted to unrelated third parties by default.

You are responsible for securing the environment in which this tool runs.

In particular:

- Never share API keys or secrets in public repositories, logs, or messages.
- Never pass `--api-secret` on the command line in shared or multi-user environments. Use environment variables or `--api-secret-stdin`.
- Rotate keys regularly.
- Use the most restrictive API key permissions possible for your use case.
- When using subprocess-mode MCP integration, be aware that credentials may be stored in MCP client configuration files. Use HTTP mode to keep credentials out of config files.

## No Warranty of Accuracy

Market data, account information, and other data returned by this tool is provided for informational purposes only and is sourced from the BitMEX API. It may be delayed, incomplete, or inaccurate. Do not rely on it as the sole basis for trading decisions. Verify critical information through the BitMEX web interface at [https://www.bitmex.com](https://www.bitmex.com).

## Open Source License

This tool is open-sourced under the [MIT License](LICENSE) by HDR Global Trading Limited. The MIT License applies to the software code. This Disclaimer, and the [BitMEX Terms of Service](https://www.bitmex.com/terms), impose additional conditions on the use of this software to interact with the BitMEX exchange.

## Support and Responsible Disclosure

Bug reports and feature requests are handled through [GitHub Issues](https://github.com/BitMEX/bitmex-cli/issues).

If you discover a security vulnerability, do not open a public issue. Report it to [security@bitmex.com](mailto:security@bitmex.com) with details of the vulnerability and steps to reproduce. You will receive an acknowledgement within 5 business days.

For exchange account support, visit [https://www.bitmex.com/support](https://www.bitmex.com/support).
