//! MCP tool and configuration overview for the Tool Activity panel.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config_host::resolve_config_relative;

use nest_error::{NestError, NestResult};
use nest_mcp::{load_mcp_config, McpHub};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// Default MCP config beside `config.toml` (`~/.config/kiwi/mcp.json` when installed).
const DEFAULT_MCP_CONFIG: &str = "mcp.json";

/// Default MCP servers for Nest memory tools.
const DEFAULT_MCP_SERVERS: &[&str] = &[
    "nest-memory",
    "nest-knowledge",
    "nest-context-memory",
];

/// `[agent]` MCP-related fields from `config.toml`.
#[derive(Debug, Clone, Deserialize)]
struct AgentMcpSection {
    #[serde(default = "default_mcp_config")]
    mcp_config: String,
    #[serde(default = "default_mcp_servers")]
    mcp_servers: Vec<String>,
    #[serde(default)]
    disabled_mcp_servers: Vec<String>,
    #[serde(default = "default_max_steps")]
    max_steps: u32,
    #[serde(default)]
    agent_mode: bool,
    #[serde(default)]
    allow_save_context: bool,
    #[serde(default)]
    allow_file_writes: bool,
}

impl Default for AgentMcpSection {
    fn default() -> Self {
        Self {
            mcp_config: default_mcp_config(),
            mcp_servers: default_mcp_servers(),
            disabled_mcp_servers: Vec::new(),
            max_steps: default_max_steps(),
            agent_mode: false,
            allow_save_context: false,
            allow_file_writes: false,
        }
    }
}

/// Agent MCP options exposed to the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAgentOptions {
    pub mcp_servers: Vec<String>,
    pub disabled_mcp_servers: Vec<String>,
    pub max_steps: u32,
    pub agent_mode: bool,
    pub allow_save_context: bool,
    pub allow_file_writes: bool,
}

/// One MCP tool with JSON Schema arguments.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolInfo {
    pub name: String,
    pub qualified_name: String,
    pub description: String,
    pub input_schema: Value,
}

/// One MCP server entry from `mcp.json` plus live tool list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub configured: bool,
    pub enabled: bool,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: HashMap<String, String>,
    pub tools: Vec<McpToolInfo>,
    pub error: Option<String>,
}

/// Full MCP overview for the Tool Activity panel.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpOverview {
    pub mcp_config_path: String,
    pub agent: McpAgentOptions,
    pub servers: Vec<McpServerInfo>,
}

/// Loads MCP servers, agent options, and tool schemas for the Tool Activity panel.
pub async fn overview(config_path: Option<&Path>) -> NestResult<McpOverview> {
    let agent = load_agent_section(config_path)?;
    let mcp_config_path = resolve_mcp_config_path(config_path, &agent.mcp_config);

    let mcp_file = load_mcp_config(&mcp_config_path).map_err(|error| {
        NestError::validation(format!(
            "failed to load MCP config {}: {error}",
            mcp_config_path.display()
        ))
    })?;

    let resolved_configs = McpHub::server_configs_from_file(&mcp_config_path, None)?;
    let config_by_name: HashMap<String, _> = resolved_configs
        .into_iter()
        .map(|config| (config.name.clone(), config))
        .collect();

    let mut names: Vec<String> = mcp_file.mcp_servers.keys().cloned().collect();
    for name in &agent.mcp_servers {
        if !names.iter().any(|entry| entry == name) {
            names.push(name.clone());
        }
    }
    names.sort();

    let mut servers = Vec::new();
    for name in names {
        let configured = agent.mcp_servers.iter().any(|entry| entry == &name);
        let enabled =
            configured && !agent.disabled_mcp_servers.iter().any(|disabled| disabled == &name);

        let (command, args, cwd, env) = if let Some(config) = config_by_name.get(&name) {
            (
                config.command.display().to_string(),
                config.args.clone(),
                config.cwd.as_ref().map(|path| path.display().to_string()),
                config.env.clone(),
            )
        } else {
            (String::new(), Vec::new(), None, HashMap::new())
        };

        let mut server = McpServerInfo {
            name: name.clone(),
            configured,
            enabled,
            command,
            args,
            cwd,
            env,
            tools: Vec::new(),
            error: None,
        };

        if !config_by_name.contains_key(&name) {
            server.error = Some(format!(
                "server `{name}` is listed in config.toml but missing from mcp.json"
            ));
        } else if enabled {
            match list_server_tools(&mcp_config_path, &name).await {
                Ok(tools) => server.tools = tools,
                Err(error) => server.error = Some(error.to_string()),
            }
        }

        servers.push(server);
    }

    Ok(McpOverview {
        mcp_config_path: mcp_config_path.display().to_string(),
        agent: McpAgentOptions {
            mcp_servers: agent.mcp_servers,
            disabled_mcp_servers: agent.disabled_mcp_servers,
            max_steps: agent.max_steps,
            agent_mode: agent.agent_mode,
            allow_save_context: agent.allow_save_context,
            allow_file_writes: agent.allow_file_writes,
        },
        servers,
    })
}

async fn list_server_tools(path: &Path, server: &str) -> NestResult<Vec<McpToolInfo>> {
    let configs = McpHub::server_configs_from_file(path, Some(&[server.to_string()]))?;
    let mut hub = McpHub::connect_all(configs).await?;
    let tools = hub.list_tools().await?;
    Ok(tools
        .into_iter()
        .map(|tool| McpToolInfo {
            name: tool.name,
            qualified_name: tool.qualified_name,
            description: tool.description,
            input_schema: tool.input_schema,
        })
        .collect())
}

fn load_agent_section(config_path: Option<&Path>) -> NestResult<AgentMcpSection> {
    let Some(path) = config_path else {
        return Ok(AgentMcpSection::default());
    };
    let text = std::fs::read_to_string(path)
        .map_err(|error| NestError::io(format!("failed to read {}: {error}", path.display())))?;
    let root: toml::Value = text
        .parse()
        .map_err(|error| NestError::config(format!("failed to parse {}: {error}", path.display())))?;
    let section = root
        .get("agent")
        .cloned()
        .unwrap_or(toml::Value::Table(toml::map::Map::new()));
    section
        .try_into()
        .map_err(|error| NestError::config(format!("invalid [agent] section: {error}")))
}

fn resolve_mcp_config_path(config_path: Option<&Path>, mcp_config: &str) -> PathBuf {
    let path = Path::new(mcp_config);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if let Some(config_path) = config_path {
        return resolve_config_relative(config_path, mcp_config);
    }
    if mcp_config == DEFAULT_MCP_CONFIG {
        return crate::config_host::kiwi_home_mcp_path();
    }
    PathBuf::from(mcp_config)
}

fn default_mcp_config() -> String {
    DEFAULT_MCP_CONFIG.into()
}

fn default_mcp_servers() -> Vec<String> {
    DEFAULT_MCP_SERVERS
        .iter()
        .map(|name| (*name).to_string())
        .collect()
}

fn default_max_steps() -> u32 {
    10
}
