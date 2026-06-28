//! Plugin registry — persists installed-plugin metadata to
//! `~/.config/kiwi/plugin-registry.toml`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// One entry in `plugin-registry.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRegistryEntry {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub version: String,
    pub enabled: bool,
    pub installed_path: PathBuf,
    /// Filename of the native library (e.g. `libkiwi_plugin_ollama.so`).
    pub entry: String,
    /// Where the plugin came from — always `"local"` for now.
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "local".to_string()
}

/// TOML root: `[plugins.<name>] ...`
#[derive(Debug, Default, Serialize, Deserialize)]
struct RegistryFile {
    #[serde(default)]
    plugins: HashMap<String, PluginRegistryEntry>,
}

/// In-memory view of `plugin-registry.toml`.
#[derive(Debug, Default)]
pub struct PluginRegistry {
    entries: HashMap<String, PluginRegistryEntry>,
}

impl PluginRegistry {
    /// Load from `path`. A missing file is not an error — returns an empty registry.
    pub fn load(path: &Path) -> (Self, Vec<String>) {
        let mut warnings = Vec::new();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return (Self::default(), warnings);
            }
            Err(err) => {
                warnings.push(format!(
                    "Plugin registry unreadable ({}): {err}",
                    path.display()
                ));
                return (Self::default(), warnings);
            }
        };

        match toml::from_str::<RegistryFile>(&content) {
            Ok(file) => (Self { entries: file.plugins }, warnings),
            Err(err) => {
                warnings.push(format!(
                    "Plugin registry corrupt ({}): {err} — starting with empty registry",
                    path.display()
                ));
                (Self::default(), warnings)
            }
        }
    }

    /// Write current state to `path`, creating parent directories as needed.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!("Failed to create registry directory {}: {e}", parent.display())
            })?;
        }
        let file = RegistryFile { plugins: self.entries.clone() };
        let content = toml::to_string_pretty(&file)
            .map_err(|e| format!("Failed to serialize registry: {e}"))?;
        fs::write(path, content)
            .map_err(|e| format!("Failed to write registry {}: {e}", path.display()))
    }

    /// Returns `true` if the plugin is enabled (defaults to `true` when not in registry).
    #[must_use]
    pub fn is_enabled(&self, name: &str) -> bool {
        self.entries.get(name).map_or(true, |e| e.enabled)
    }

    /// Returns the registry entry for `name`, if present.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&PluginRegistryEntry> {
        self.entries.get(name)
    }

    /// All entries, sorted by name.
    #[must_use]
    pub fn entries_sorted(&self) -> Vec<&PluginRegistryEntry> {
        let mut v: Vec<_> = self.entries.values().collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    /// Add or replace an entry.
    pub fn register(&mut self, entry: PluginRegistryEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    /// Enable a plugin by name. Returns `false` if the name is not in the registry.
    pub fn enable(&mut self, name: &str) -> bool {
        if let Some(entry) = self.entries.get_mut(name) {
            entry.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a plugin by name. Returns `false` if the name is not in the registry.
    pub fn disable(&mut self, name: &str) -> bool {
        if let Some(entry) = self.entries.get_mut(name) {
            entry.enabled = false;
            true
        } else {
            false
        }
    }

    /// Remove an entry. Returns `Some(entry)` if it existed, `None` otherwise.
    pub fn remove(&mut self, name: &str) -> Option<PluginRegistryEntry> {
        self.entries.remove(name)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Default registry path: `~/.config/kiwi/plugin-registry.toml`.
#[must_use]
pub fn default_registry_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("kiwi")
            .join("plugin-registry.toml")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_registry_path() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("kiwi-registry-test-{id}.toml"))
    }

    fn sample_entry(name: &str, enabled: bool) -> PluginRegistryEntry {
        PluginRegistryEntry {
            name: name.to_string(),
            display_name: Some(format!("{name} Plugin")),
            version: "0.1.0".to_string(),
            enabled,
            installed_path: PathBuf::from(format!("/tmp/plugins/{name}")),
            entry: format!("lib{name}.so"),
            source: "local".to_string(),
        }
    }

    #[test]
    fn missing_file_returns_empty_registry() {
        let (registry, warnings) =
            PluginRegistry::load(Path::new("/tmp/kiwi-registry-nonexistent.toml"));
        assert!(registry.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn round_trip_save_and_load() {
        let path = temp_registry_path();
        let mut registry = PluginRegistry::default();
        registry.register(sample_entry("hello", true));
        registry.register(sample_entry("ollama", false));
        registry.save(&path).expect("save");

        let (loaded, warnings) = PluginRegistry::load(&path);
        assert!(warnings.is_empty());
        assert_eq!(loaded.len(), 2);
        assert!(loaded.is_enabled("hello"));
        assert!(!loaded.is_enabled("ollama"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn is_enabled_defaults_to_true_for_unknown_plugin() {
        let registry = PluginRegistry::default();
        assert!(registry.is_enabled("unknown"));
    }

    #[test]
    fn enable_disable_update_entry() {
        let mut registry = PluginRegistry::default();
        registry.register(sample_entry("hello", true));

        assert!(registry.disable("hello"));
        assert!(!registry.is_enabled("hello"));

        assert!(registry.enable("hello"));
        assert!(registry.is_enabled("hello"));
    }

    #[test]
    fn enable_disable_return_false_for_unknown() {
        let mut registry = PluginRegistry::default();
        assert!(!registry.enable("ghost"));
        assert!(!registry.disable("ghost"));
    }

    #[test]
    fn remove_returns_entry_and_shrinks_registry() {
        let mut registry = PluginRegistry::default();
        registry.register(sample_entry("hello", true));
        let removed = registry.remove("hello");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "hello");
        assert!(registry.is_empty());
    }

    #[test]
    fn entries_sorted_returns_alphabetical_order() {
        let mut registry = PluginRegistry::default();
        registry.register(sample_entry("zebra", true));
        registry.register(sample_entry("alpha", true));
        registry.register(sample_entry("mango", true));
        let sorted: Vec<_> = registry.entries_sorted().iter().map(|e| e.name.as_str()).collect();
        assert_eq!(sorted, ["alpha", "mango", "zebra"]);
    }

    #[test]
    fn corrupt_registry_file_produces_warning_and_empty_registry() {
        let path = temp_registry_path();
        std::fs::write(&path, "not valid toml ][[[").expect("write");
        let (registry, warnings) = PluginRegistry::load(&path);
        assert!(registry.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("corrupt"));
        let _ = std::fs::remove_file(&path);
    }
}
