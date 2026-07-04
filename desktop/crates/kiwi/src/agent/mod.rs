//! Agent configuration load/save for Kiwi.

mod loop_config;

use std::fs;
use std::path::Path;

use nest_ai_ollama::{AiSection, OllamaConfig, OllamaSharedConfig};
use nest_config::ConfigService;
use nest_error::{NestError, NestResult};
use toml::Value;

pub use loop_config::{mcp_config_error, AgentLoopConfig};

/// Editable agent settings shown in the Agent sidebar.
#[derive(Debug, Clone)]
pub struct AgentSettings {
    /// Ollama host IP or hostname.
    pub host: String,
    /// Ollama HTTP port.
    pub port: String,
    /// Active chat model id.
    pub model: String,
    /// Available model ids.
    pub models: Vec<String>,
    /// Draft text for adding a model.
    pub new_model: String,
    /// Transient status after apply/save.
    pub status: Option<String>,
    /// MCP server connection summary for the sidebar.
    pub mcp_status: Option<String>,
    /// Tool count from the last MCP hub probe.
    pub mcp_tool_count: Option<usize>,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            host: "192.168.88.10".into(),
            port: "11434".into(),
            model: "qwen2.5-coder:3b".into(),
            models: vec![
                "nomic-embed-text:latest".into(),
                "llama3.2:3b".into(),
                "qwen2.5-coder:3b".into(),
            ],
            new_model: String::new(),
            status: None,
            mcp_status: None,
            mcp_tool_count: None,
        }
    }
}

impl AgentSettings {
    /// Loads agent settings from `[ai]` config, falling back to defaults.
    pub fn from_config_service(service: &ConfigService) -> NestResult<Self> {
        let Ok(section) = service.section::<AiSection>("ai") else {
            return Ok(Self::default());
        };

        let (host, port) = if let Some(host) = section.host.filter(|h| !h.trim().is_empty()) {
            (host, section.port.to_string())
        } else {
            let (host, port) = OllamaConfig::host_port_from_base_url(&section.base_url);
            (host, port.to_string())
        };

        let models = if section.models.is_empty() {
            vec![section.model.clone()]
        } else {
            section.models
        };

        Ok(Self {
            host,
            port,
            model: section.model,
            models,
            new_model: String::new(),
            status: None,
            mcp_status: None,
            mcp_tool_count: None,
        })
    }

    /// Parses the port, defaulting to Ollama's standard port.
    pub fn port_u16(&self) -> u16 {
        self.port.parse().unwrap_or(11434)
    }

    /// Builds the Ollama HTTP base URL from host and port.
    pub fn base_url(&self) -> String {
        OllamaConfig::base_url_from_host(&self.host, self.port_u16())
    }

    /// Applies settings to the runtime Ollama config handle.
    pub fn apply_runtime(&self, shared: &OllamaSharedConfig) {
        shared.set(OllamaConfig::new(self.base_url(), &self.model));
    }

    /// Adds a model from the draft field when non-empty and unique.
    pub fn add_model_from_draft(&mut self) {
        let name = self.new_model.trim().to_string();
        if name.is_empty() {
            return;
        }
        if !self.models.iter().any(|model| model == &name) {
            self.models.push(name);
        }
        self.new_model.clear();
    }

    /// Removes a model by index and adjusts the active model if needed.
    pub fn remove_model(&mut self, index: usize) {
        if index >= self.models.len() {
            return;
        }
        let removed = self.models.remove(index);
        if self.model == removed {
            self.model = self
                .models
                .first()
                .cloned()
                .unwrap_or_else(|| "smollm2:360m".into());
        }
    }

    /// Persists agent settings to the config file, preserving other sections.
    pub fn save_to_config_path(&self, path: &Path) -> NestResult<()> {
        let content = fs::read_to_string(path).map_err(|error| {
            NestError::io(format!("failed to read {}: {error}", path.display()))
        })?;
        let mut root: Value = content.parse().map_err(|error| {
            NestError::config(format!("failed to parse {}: {error}", path.display()))
        })?;

        let table = root
            .as_table_mut()
            .ok_or_else(|| NestError::config("config root must be a table"))?;

        let ai = table
            .entry("ai")
            .or_insert_with(|| Value::Table(toml::map::Map::new()));

        let ai_table = ai
            .as_table_mut()
            .ok_or_else(|| NestError::config("[ai] must be a table"))?;

        ai_table.insert("enabled".into(), Value::Boolean(true));
        ai_table.insert("provider".into(), Value::String("ollama".into()));
        ai_table.insert(
            "host".into(),
            Value::String(self.host.trim().to_string()),
        );
        ai_table.insert(
            "port".into(),
            Value::Integer(i64::from(self.port_u16())),
        );
        ai_table.insert(
            "base_url".into(),
            Value::String(self.base_url()),
        );
        ai_table.insert("model".into(), Value::String(self.model.clone()));
        ai_table.insert(
            "models".into(),
            Value::Array(
                self.models
                    .iter()
                    .map(|model| Value::String(model.clone()))
                    .collect(),
            ),
        );

        let serialized = toml::to_string_pretty(&root).map_err(|error| {
            NestError::config(format!("failed to serialize config: {error}"))
        })?;
        fs::write(path, serialized).map_err(|error| {
            NestError::io(format!("failed to write {}: {error}", path.display()))
        })?;
        Ok(())
    }
}
