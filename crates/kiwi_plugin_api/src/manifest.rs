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
    /// How `kiwi plugin install` builds and stages this plugin.
    #[serde(default)]
    pub install: Option<PluginInstallConfig>,
}

/// Install instructions declared in `plugin.toml` for `kiwi plugin install`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PluginInstallConfig {
    /// `cargo` — build via `cargo build`; `copy` — manifest-only; `artifact` — pre-built library.
    #[serde(default = "default_install_kind")]
    pub kind: PluginInstallKind,
    /// Cargo package name when `kind = "cargo"`. Inferred from `Cargo.toml` when omitted.
    #[serde(default)]
    pub package: Option<String>,
    /// `release` or `debug`. Defaults to `release`.
    #[serde(default = "default_install_profile")]
    pub profile: String,
    /// Native library stem (`lib{library}.so`). Inferred from the crate `[lib]` name when omitted.
    #[serde(default)]
    pub library: Option<String>,
    /// Executables to copy onto the user's PATH after install.
    #[serde(default)]
    pub binaries: Vec<String>,
    /// Destination for `binaries`; defaults to `~/.local/bin`.
    #[serde(default)]
    pub bin_dir: Option<String>,
    /// Files to copy when `kind = "copy"`. Defaults to `["plugin.toml"]`.
    #[serde(default)]
    pub files: Vec<String>,
    /// Additional workspace packages to build and install alongside this plugin.
    #[serde(default)]
    pub extra_packages: Vec<PluginExtraPackage>,
}

/// Build and install binaries from another workspace package during plugin install.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PluginExtraPackage {
    pub package: String,
    #[serde(default)]
    pub binaries: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginInstallKind {
    #[default]
    Cargo,
    Copy,
    Artifact,
}

fn default_install_kind() -> PluginInstallKind {
    PluginInstallKind::Cargo
}

fn default_install_profile() -> String {
    "release".to_string()
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

    #[test]
    fn deserializes_install_config() {
        let manifest: PluginManifest = toml::from_str(
            r#"
            name = "ollama"
            version = "0.1.0"
            min_kiwi_version = "0.1.0"

            [install]
            kind = "cargo"
            package = "kiwi_plugin_ollama"
            profile = "release"
            library = "kiwi_plugin_ollama"
            binaries = ["kiwi-ollama"]

            [[install.extra_packages]]
            package = "kiwi_mcp_memory"
            binaries = ["kiwi-mcp-memory"]
            "#,
        )
        .expect("manifest");
        let install = manifest.install.expect("install");
        assert_eq!(install.kind, PluginInstallKind::Cargo);
        assert_eq!(install.package.as_deref(), Some("kiwi_plugin_ollama"));
        assert_eq!(install.profile, "release");
        assert_eq!(install.binaries, vec!["kiwi-ollama".to_string()]);
        assert_eq!(install.extra_packages.len(), 1);
        assert_eq!(install.extra_packages[0].package, "kiwi_mcp_memory");
        assert_eq!(
            install.extra_packages[0].binaries,
            vec!["kiwi-mcp-memory".to_string()]
        );
    }
}
