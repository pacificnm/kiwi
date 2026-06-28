use serde::Deserialize;

/// Metadata from a plugin's `plugin.toml` manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub min_kiwi_version: String,
    /// Dynamic-library entry symbol. Defaults to [`crate::PLUGIN_INIT_SYMBOL`].
    #[serde(default = "default_entry_symbol")]
    pub entry: String,

    // Optional rich metadata — all fields below are backward-compatible.
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub capabilities: Option<PluginCapabilities>,
    /// If set, this plugin provides an AI agent command for the Agent panel.
    #[serde(default)]
    pub agent: Option<AgentPluginConfig>,
}

/// Agent command declared by a plugin in its `plugin.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct AgentPluginConfig {
    /// The executable to run (e.g. `"cursor"`, `"aider"`).
    pub command: String,
    /// Arguments passed to the executable on spawn.
    pub args: Vec<String>,
}

impl PluginManifest {
    /// Returns `display_name` if set, otherwise falls back to `name`.
    #[must_use]
    pub fn effective_display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }
}

/// Optional capability flags declared by a plugin in its `plugin.toml`.
///
/// These are informational — Kiwi does not enforce them at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct PluginCapabilities {
    pub commands: bool,
    pub panels: bool,
    pub tabs: bool,
    pub events: bool,
    pub mcp: bool,
}

fn default_entry_symbol() -> String {
    crate::PLUGIN_INIT_SYMBOL.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_manifest_with_defaults() {
        let manifest: PluginManifest = toml::from_str(
            r#"
            name = "hello"
            version = "0.1.0"
            min_kiwi_version = "0.1.0"
            "#,
        )
        .expect("manifest");
        assert_eq!(manifest.name, "hello");
        assert_eq!(manifest.entry, "kiwi_plugin_init");
        assert!(manifest.display_name.is_none());
        assert!(manifest.description.is_none());
        assert!(manifest.author.is_none());
        assert!(manifest.capabilities.is_none());
    }

    #[test]
    fn deserializes_rich_manifest() {
        let manifest: PluginManifest = toml::from_str(
            r#"
            name = "ollama"
            display_name = "Ollama"
            version = "0.1.0"
            min_kiwi_version = "0.1.0"
            description = "Local Ollama integration"
            author = "PacificNM"

            [capabilities]
            commands = true
            panels = true
            "#,
        )
        .expect("manifest");
        assert_eq!(manifest.display_name.as_deref(), Some("Ollama"));
        assert_eq!(manifest.description.as_deref(), Some("Local Ollama integration"));
        assert_eq!(manifest.author.as_deref(), Some("PacificNM"));
        let caps = manifest.capabilities.expect("capabilities");
        assert!(caps.commands);
        assert!(caps.panels);
        assert!(!caps.tabs);
        assert!(!caps.mcp);
    }

    #[test]
    fn effective_display_name_falls_back_to_name() {
        let manifest: PluginManifest = toml::from_str(
            r#"name = "hello"
            version = "0.1.0"
            min_kiwi_version = "0.1.0""#,
        )
        .expect("manifest");
        assert_eq!(manifest.effective_display_name(), "hello");
    }

    #[test]
    fn effective_display_name_uses_display_name_when_set() {
        let manifest: PluginManifest = toml::from_str(
            r#"name = "hello"
            display_name = "Hello Plugin"
            version = "0.1.0"
            min_kiwi_version = "0.1.0""#,
        )
        .expect("manifest");
        assert_eq!(manifest.effective_display_name(), "Hello Plugin");
    }
}
