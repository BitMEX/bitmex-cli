/// Tool registry: merges clap command metadata with the tool catalog
/// to produce MCP tool definitions.
use std::collections::HashMap;
use std::sync::Arc;

use rmcp::model::{Tool, ToolAnnotations};
use serde_json::Value;

use super::schema::clap_command_to_schema;
use crate::errors::{BitmexError, Result};

#[derive(Debug, Clone)]
pub(crate) struct ArgMeta {
    pub(crate) id: String,
    /// Long flag name (e.g., "count", "asset-class"). None for positional args.
    pub(crate) long: Option<String>,
    /// True for SetTrue/SetFalse/Count actions that emit as presence flags.
    pub(crate) is_bool_flag: bool,
    /// 0-based position index for positional args. None for flag args.
    pub(crate) positional_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct ToolEntry {
    pub(crate) tool: Tool,
    pub(crate) canonical_key: String,
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) group: String,
    pub(crate) dangerous: bool,
    pub(crate) clap_args: Vec<ArgMeta>,
}

#[derive(Debug, Clone)]
pub(crate) struct ToolRegistry {
    tools: Vec<ToolEntry>,
    by_name: HashMap<String, usize>,
}

impl ToolRegistry {
    #[cfg(test)]
    pub(crate) fn build(active_services: &[String]) -> Result<Self> {
        Self::build_with_options(active_services, false)
    }

    pub(crate) fn build_with_options(
        active_services: &[String],
        allow_dangerous: bool,
    ) -> Result<Self> {
        let catalog = load_catalog()?;
        let clap_root = crate::Cli::command();

        let catalog_index = build_catalog_index(&catalog)?;
        let mut tools = Vec::new();
        let mut by_name = HashMap::new();

        collect_clap_tools(
            &clap_root,
            &[],
            &catalog_index,
            active_services,
            allow_dangerous,
            &mut tools,
        )?;

        for (i, entry) in tools.iter().enumerate() {
            by_name.insert(entry.tool.name.to_string(), i);
        }

        if tools.is_empty() {
            return Err(BitmexError::Validation {
                message: "No tools available after service filtering. Ensure at least one \
                 REST-eligible service group is specified."
                    .into(),
            });
        }

        Ok(Self { tools, by_name })
    }

    pub(crate) fn tools(&self) -> &[ToolEntry] {
        &self.tools
    }

    pub(crate) fn get_by_name(&self, name: &str) -> Option<&ToolEntry> {
        self.by_name.get(name).map(|&i| &self.tools[i])
    }

    pub(crate) fn tool_definitions(&self) -> Vec<Tool> {
        self.tools.iter().map(|e| e.tool.clone()).collect()
    }
}

use clap::CommandFactory;

struct CatalogEntry {
    group: String,
    dangerous: bool,
    description: String,
    /// Raw `parameters` array from the catalog, used for richer schema generation
    /// (integer types, explicit required flags) that clap introspection cannot express.
    parameters: Option<Vec<Value>>,
}

fn load_catalog() -> Result<Value> {
    let catalog_bytes = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/agents/tool-catalog.json"
    ));
    serde_json::from_str(catalog_bytes).map_err(|e| BitmexError::Parse { message: e.to_string() })
}

fn build_catalog_index(catalog: &Value) -> Result<HashMap<String, CatalogEntry>> {
    let commands = catalog
        .get("commands")
        .and_then(|c| c.as_array())
        .ok_or_else(|| BitmexError::Parse { message: "Catalog missing 'commands' array".into() })?;

    let mut index = HashMap::new();
    for cmd in commands {
        let raw_command = cmd
            .get("command")
            .and_then(|c| c.as_str())
            .unwrap_or_default();
        let key = canonical_key_from_catalog(raw_command);
        if key.is_empty() {
            continue;
        }
        let group = cmd
            .get("group")
            .and_then(|g| g.as_str())
            .unwrap_or("unknown")
            .to_string();
        let dangerous = cmd
            .get("dangerous")
            .and_then(|d| d.as_bool())
            .unwrap_or(false);
        let description = cmd
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();
        let parameters = cmd
            .get("parameters")
            .and_then(|p| p.as_array())
            .map(|arr| arr.to_vec());
        index.insert(
            key,
            CatalogEntry {
                group,
                dangerous,
                description,
                parameters,
            },
        );
    }
    Ok(index)
}

fn canonical_key_from_catalog(command_str: &str) -> String {
    command_str
        .split_whitespace()
        .skip(1) // skip "bitmex"
        .filter(|t| !t.starts_with('<') && !t.ends_with('>'))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
        .replace('_', "-")
}

fn canonical_key_from_clap(path: &[&str]) -> String {
    path.to_vec().join(" ").to_lowercase().replace('_', "-")
}

fn tool_name_from_key(key: &str) -> String {
    format!("bitmex.{}", key.replace([' ', '-'], "."))
}

fn collect_clap_tools(
    cmd: &clap::Command,
    parent_path: &[&str],
    catalog_index: &HashMap<String, CatalogEntry>,
    active_services: &[String],
    allow_dangerous: bool,
    out: &mut Vec<ToolEntry>,
) -> Result<()> {
    let subs: Vec<_> = cmd.get_subcommands().collect();

    if subs.is_empty() && !parent_path.is_empty() {
        let key = canonical_key_from_clap(parent_path);
        if let Some(catalog_entry) = catalog_index.get(&key) {
            if !active_services.contains(&catalog_entry.group) {
                return Ok(());
            }
            if super::schema::is_mcp_excluded_command(&key) {
                return Ok(());
            }
            let name = tool_name_from_key(&key);
            let description = build_description(cmd, catalog_entry);
            // Prefer catalog-driven schema (supports integer types, examples, oneOf)
            // over runtime clap introspection which can only produce "string" for numerics.
            let mut input_schema = if let Some(params) = &catalog_entry.parameters {
                super::schema::catalog_parameters_to_schema(params)
            } else {
                clap_command_to_schema(cmd)
            };
            if catalog_entry.dangerous && !allow_dangerous {
                super::schema::inject_dangerous_confirmation(&mut input_schema);
            }
            let input_schema = input_schema;
            let schema_obj: serde_json::Map<String, Value> =
                serde_json::from_value(input_schema).unwrap_or_default();

            let mut tool = Tool::new(name.clone(), description, Arc::new(schema_obj));

            if catalog_entry.dangerous {
                tool = tool.with_annotations(ToolAnnotations::from_raw(
                    None,
                    None,
                    Some(true),
                    None,
                    None,
                ));
            }

            let clap_args = extract_clap_arg_meta(cmd);

            out.push(ToolEntry {
                tool,
                canonical_key: key,
                group: catalog_entry.group.clone(),
                dangerous: catalog_entry.dangerous,
                clap_args,
            });
        }
        return Ok(());
    }

    for sub in subs {
        let sub_name = sub.get_name();
        if sub_name == "help" {
            continue;
        }
        let mut path = parent_path.to_vec();
        path.push(sub_name);
        collect_clap_tools(
            sub,
            &path,
            catalog_index,
            active_services,
            allow_dangerous,
            out,
        )?;
    }

    Ok(())
}

fn build_description(cmd: &clap::Command, catalog_entry: &CatalogEntry) -> String {
    let base = cmd
        .get_about()
        .map(|a| a.to_string())
        .or_else(|| {
            if !catalog_entry.description.is_empty() {
                Some(catalog_entry.description.clone())
            } else {
                None
            }
        })
        .unwrap_or_default();

    if catalog_entry.dangerous {
        format!("[DANGEROUS: requires human confirmation] {base}")
    } else {
        base
    }
}

fn extract_clap_arg_meta(cmd: &clap::Command) -> Vec<ArgMeta> {
    let mut meta = Vec::new();
    let mut positional_idx = 0usize;

    for arg in cmd.get_arguments() {
        let id = arg.get_id().as_str();
        if id == "help" || id == "version" {
            continue;
        }
        if super::schema::is_mcp_excluded_arg(id) {
            continue;
        }

        let long = arg.get_long().map(|s| s.to_string());
        let is_bool_flag = !arg.get_action().takes_values();
        let is_positional = long.is_none() && arg.get_short().is_none();

        let positional_index = if is_positional {
            let idx = positional_idx;
            positional_idx += 1;
            Some(idx)
        } else {
            None
        };

        meta.push(ArgMeta {
            id: id.to_string(),
            long,
            is_bool_flag,
            positional_index,
        });
    }

    meta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_key_from_catalog_strips_bitmex_and_placeholders() {
        assert_eq!(
            canonical_key_from_catalog("bitmex market stats"),
            "market stats"
        );
        assert_eq!(
            canonical_key_from_catalog("bitmex order buy <symbol> <qty>"),
            "order buy"
        );
        assert_eq!(
            canonical_key_from_catalog("bitmex market orderbook <symbol>"),
            "market orderbook"
        );
        assert_eq!(
            canonical_key_from_catalog("bitmex wallet balance"),
            "wallet balance"
        );
    }

    #[test]
    fn canonical_key_from_clap_joins_path() {
        assert_eq!(canonical_key_from_clap(&["market", "stats"]), "market stats");
        assert_eq!(canonical_key_from_clap(&["order", "buy"]), "order buy");
    }

    #[test]
    fn tool_name_generation() {
        assert_eq!(tool_name_from_key("market stats"), "bitmex.market.stats");
        assert_eq!(tool_name_from_key("order buy"), "bitmex.order.buy");
        assert_eq!(
            tool_name_from_key("market orderbook"),
            "bitmex.market.orderbook"
        );
    }

    #[test]
    fn registry_builds_for_market() {
        let registry = ToolRegistry::build(&["market".into()]).unwrap();
        assert!(!registry.tools().is_empty());
        for entry in registry.tools() {
            assert_eq!(entry.group, "market");
        }
    }

    #[test]
    fn registry_rejects_empty_after_filter() {
        let err = ToolRegistry::build(&[]).unwrap_err().to_string();
        assert!(err.contains("No tools available"));
    }

    #[test]
    fn dangerous_tools_have_annotation() {
        let registry = ToolRegistry::build(&["order".into()]).unwrap();
        let dangerous_tools: Vec<_> = registry.tools().iter().filter(|e| e.dangerous).collect();
        for entry in &dangerous_tools {
            let desc = entry.tool.description.as_deref().unwrap_or("");
            assert!(
                desc.contains("[DANGEROUS"),
                "Tool {} missing danger prefix in description",
                entry.tool.name
            );
            assert!(
                entry
                    .tool
                    .annotations
                    .as_ref()
                    .and_then(|a| a.destructive_hint)
                    .unwrap_or(false),
                "Tool {} missing destructive_hint annotation",
                entry.tool.name
            );
        }
    }

    #[test]
    fn market_tools_not_dangerous() {
        let registry = ToolRegistry::build(&["market".into()]).unwrap();
        for entry in registry.tools() {
            assert!(
                !entry.dangerous,
                "Market tool {} should not be dangerous",
                entry.tool.name
            );
        }
    }

    #[test]
    fn registry_filters_by_service() {
        let market = ToolRegistry::build(&["market".into()]).unwrap();
        let all = ToolRegistry::build(&[
            "market".into(),
            "account".into(),
            "order".into(),
            "wallet".into(),
            "staking".into(),
            "subaccount".into(),
            "auth".into(),
        ])
        .unwrap();
        assert!(all.tools().len() > market.tools().len());
    }

    #[test]
    fn tool_lookup_by_name() {
        let registry = ToolRegistry::build(&["market".into()]).unwrap();
        let orderbook = registry.get_by_name("bitmex.market.orderbook");
        assert!(orderbook.is_some(), "Should find bitmex.market.orderbook tool");
    }

    #[test]
    fn account_position_mode_registered_and_dangerous() {
        let registry =
            ToolRegistry::build(&["account".into(), "position".into()]).unwrap();
        let entry = registry
            .get_by_name("bitmex.account.position.mode")
            .expect("account position-mode should register as an MCP tool");
        assert!(entry.dangerous, "position-mode must be marked dangerous");
        // The `position mode` alias is intentionally CLI-only (no catalog entry),
        // so it must not surface as a second, duplicate MCP tool even when the
        // position group is loaded.
        assert!(
            registry.get_by_name("bitmex.position.mode").is_none(),
            "position mode alias should not be an MCP tool"
        );
    }

    #[test]
    fn auth_excluded_commands() {
        let registry = ToolRegistry::build(&["auth".into()]).unwrap();
        assert!(
            registry.get_by_name("bitmex.auth.set").is_none(),
            "auth set should be excluded from MCP registration"
        );
        assert!(
            registry.get_by_name("bitmex.auth.reset").is_none(),
            "auth reset should be excluded from MCP registration"
        );
        assert!(
            registry.get_by_name("bitmex.auth.show").is_some(),
            "auth show should remain registered"
        );
    }

    #[test]
    fn registry_registers_peg_and_chaser_tools() {
        let registry = ToolRegistry::build(&["order".into()]).unwrap();
        let by_name = |n: &str| registry.tools().iter().find(|e| e.tool.name == n);

        let chase = by_name("bitmex.order.chase").expect("order.chase tool should be registered");
        assert!(chase.dangerous);
        let props = chase
            .tool
            .input_schema
            .get("properties")
            .and_then(|p| p.as_object())
            .expect("chase schema should have properties");
        for k in ["symbol", "side", "qty", "offset", "bothways"] {
            assert!(props.contains_key(k), "chase schema missing `{k}`");
        }
        // `side` should be constrained to Buy/Sell.
        assert!(props["side"].get("oneOf").is_some(), "side should expose oneOf");

        // tool_name_from_key replaces '-' with '.', so trailing-stop -> trailing.stop.
        let trailing = by_name("bitmex.order.trailing.stop")
            .expect("order.trailing-stop tool should be registered");
        assert!(trailing.dangerous);
        let tprops = trailing
            .tool
            .input_schema
            .get("properties")
            .and_then(|p| p.as_object())
            .expect("trailing-stop schema should have properties");
        for k in ["symbol", "side", "qty", "offset", "limit_price", "trigger"] {
            assert!(tprops.contains_key(k), "trailing-stop schema missing `{k}`");
        }
    }

    #[test]
    fn buy_tool_exposes_peg_parameters() {
        let registry = ToolRegistry::build(&["order".into()]).unwrap();
        let buy = registry
            .tools()
            .iter()
            .find(|e| e.tool.name == "bitmex.order.buy")
            .expect("order.buy tool should be registered");
        let props = buy
            .tool
            .input_schema
            .get("properties")
            .and_then(|p| p.as_object())
            .unwrap();
        assert!(props.contains_key("peg_price_type"));
        assert!(props.contains_key("peg_offset_value"));
        assert!(props["peg_price_type"].get("oneOf").is_some());
    }

    #[test]
    fn dangerous_tools_have_acknowledged_in_schema() {
        let registry = ToolRegistry::build(&["order".into()]).unwrap();
        let dangerous_tools: Vec<_> = registry.tools().iter().filter(|e| e.dangerous).collect();
        assert!(
            !dangerous_tools.is_empty(),
            "order group should have dangerous tools"
        );
        for entry in &dangerous_tools {
            let schema = &entry.tool.input_schema;
            let props = schema
                .get("properties")
                .and_then(|p| p.as_object())
                .expect("schema should have properties");
            assert!(
                props.contains_key("acknowledged"),
                "Dangerous tool {} missing acknowledged in schema",
                entry.tool.name
            );
        }
    }

    #[test]
    fn websocket_tools_excluded() {
        let services = super::super::apply_exclusions(&["websocket".into()]);
        if services.is_empty() {
            return;
        }
        let registry = ToolRegistry::build(&services);
        if let Ok(r) = registry {
            for entry in r.tools() {
                assert_ne!(entry.group, "websocket");
                assert_ne!(entry.group, "futures-ws");
            }
        }
    }
}
