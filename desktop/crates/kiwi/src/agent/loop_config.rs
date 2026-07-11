//! Agent loop configuration from `[agent]` config section.

use std::path::{Path, PathBuf};

use std::time::Duration;

use nest_agent::AgentConfig;
use nest_config::ConfigService;
use nest_error::{NestError, NestResult};
use serde::Deserialize;

/// Default MCP config path relative to the Kiwi config file.
pub const DEFAULT_MCP_CONFIG: &str = "../../../.cursor/mcp.json";

/// Default MCP servers for Nest memory tools.
pub const DEFAULT_MCP_SERVERS: &[&str] = &[
    "nest-memory",
    "nest-knowledge",
    "nest-context-memory",
];

/// `[agent]` section in `config.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentSection {
    /// Model id for tool-using agent runs (falls back to `[ai].model`).
    pub model: Option<String>,
    /// Path to Cursor MCP config, relative to the config file unless absolute.
    #[serde(default = "default_mcp_config")]
    pub mcp_config: String,
    /// MCP server ids to connect (must exist in `mcp_config`).
    #[serde(default = "default_mcp_servers")]
    pub mcp_servers: Vec<String>,
    /// MCP servers disabled in the Agent sidebar (subset of `mcp_servers`).
    #[serde(default)]
    pub disabled_mcp_servers: Vec<String>,
    /// Maximum model ↔ tool iterations.
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
    /// When true, chat Send uses the MCP agent loop by default.
    #[serde(default)]
    pub agent_mode: bool,
    /// When true, `save_context_memory` may auto-run without per-call approval.
    #[serde(default)]
    pub allow_save_context: bool,
    /// When true, Nest file write/delete tools may auto-run without per-call approval.
    #[serde(default)]
    pub allow_file_writes: bool,
}

impl Default for AgentSection {
    fn default() -> Self {
        Self {
            model: None,
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

/// Resolved agent loop settings for CLI and GUI.
#[derive(Debug, Clone)]
pub struct AgentLoopConfig {
    /// Model id for agent runs.
    pub model: String,
    /// Resolved absolute or cwd-relative MCP config path.
    pub mcp_config_path: PathBuf,
    /// MCP server ids configured in `[agent].mcp_servers`.
    pub mcp_servers: Vec<String>,
    /// MCP servers disabled in the Agent sidebar.
    pub disabled_mcp_servers: Vec<String>,
    /// Maximum loop steps.
    pub max_steps: u32,
    /// Default Agent checkbox state in the chat panel.
    #[allow(dead_code)]
    pub agent_mode: bool,
    /// Whether `save_context_memory` may auto-run.
    pub allow_save_context: bool,
    /// Whether Nest file write/delete tools may auto-run.
    pub allow_file_writes: bool,
}

impl AgentLoopConfig {
    /// Loads agent loop settings from config, falling back to `[ai]` model.
    pub fn from_config_service(service: &ConfigService) -> NestResult<Self> {
        let section = service
            .section::<AgentSection>("agent")
            .unwrap_or_default();

        let ai_model = service
            .section::<nest_ai_ollama::AiSection>("ai")
            .map(|ai| ai.model)
            .unwrap_or_else(|_| "qwen2.5-coder:3b".into());

        let model = section
            .model
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(ai_model);

        let mcp_config_path = resolve_mcp_config_path(service.path(), &section.mcp_config);

        Ok(Self {
            model,
            mcp_config_path,
            mcp_servers: section.mcp_servers,
            disabled_mcp_servers: section.disabled_mcp_servers,
            max_steps: section.max_steps,
            agent_mode: section.agent_mode,
            allow_save_context: section.allow_save_context,
            allow_file_writes: section.allow_file_writes,
        })
    }

    /// MCP servers enabled for agent runs.
    pub fn enabled_mcp_servers(&self) -> Vec<String> {
        self.mcp_servers
            .iter()
            .filter(|name| !self.disabled_mcp_servers.iter().any(|disabled| disabled == *name))
            .cloned()
            .collect()
    }

    /// Builds [`AgentConfig`] for [`nest_agent::AgentLoop`].
    pub fn agent_config(&self, workspace_root: Option<std::path::PathBuf>) -> AgentConfig {
        let mut config = AgentConfig::default()
            .with_max_steps(self.max_steps)
            .with_allow_save_context(self.allow_save_context)
            .with_allow_file_writes(self.allow_file_writes)
            .with_tool_timeout(Duration::from_secs(180));
        if let Some(root) = workspace_root {
            config = config.with_workspace_root(root);
        }
        config
    }
}

/// Resolves an MCP config path relative to the Kiwi config file directory.
pub fn resolve_mcp_config_path(config_path: Option<&Path>, mcp_config: &str) -> PathBuf {
    let path = Path::new(mcp_config);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    let base = config_path
        .and_then(|value| value.parent())
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    base.join(path)
}

/// Formats MCP config path errors for UI and CLI.
pub fn mcp_config_error(path: &Path, error: &NestError) -> String {
    format!(
        "failed to load MCP config {}: {error}",
        path.display()
    )
}
