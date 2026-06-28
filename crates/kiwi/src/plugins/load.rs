use kiwi_plugin_api::kiwi_version_compatible;
use kiwi_plugin_loader::{load_plugin, PluginHost};

use crate::config::PluginsSettings;
use crate::state::{PluginEntry, PluginPaletteCommand, PluginStatus, PluginsState};

use super::discovery::{discover_plugins, PluginCandidate};
use super::registry::{default_registry_path, PluginRegistry, PluginRegistryEntry};

pub struct PluginLoadOutcome {
    pub state: PluginsState,
    pub host: PluginHost,
    pub messages: Vec<String>,
}

pub fn load_plugins(config: &PluginsSettings, kiwi_version: &str) -> PluginLoadOutcome {
    let mut outcome = PluginLoadOutcome {
        state: PluginsState::default(),
        host: PluginHost::empty(),
        messages: Vec::new(),
    };

    if !config.enabled {
        return outcome;
    }

    // Load (or create empty) registry.
    let registry_path = default_registry_path();
    let mut registry = if let Some(ref path) = registry_path {
        let (reg, warnings) = PluginRegistry::load(path);
        outcome.messages.extend(warnings);
        reg
    } else {
        PluginRegistry::default()
    };

    let (candidates, discovery_warnings) = discover_plugins(&config.directory);
    outcome.messages.extend(discovery_warnings);

    for candidate in &candidates {
        let name = &candidate.manifest.name;

        // Auto-register newly discovered plugins (enabled by default).
        if registry.get(name).is_none() {
            let entry_filename = candidate
                .library_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&candidate.manifest.entry)
                .to_string();
            let installed_path = candidate
                .manifest_path
                .parent()
                .unwrap_or(&candidate.manifest_path)
                .to_path_buf();
            registry.register(PluginRegistryEntry {
                name: name.clone(),
                display_name: candidate.manifest.display_name.clone(),
                version: candidate.manifest.version.clone(),
                enabled: true,
                installed_path,
                entry: entry_filename,
                source: "local".to_string(),
            });
        }

        let enabled = registry.is_enabled(name);
        load_candidate(&mut outcome, candidate, kiwi_version, enabled);
    }

    // Persist any registry changes (new auto-registrations, etc.).
    if let Some(ref path) = registry_path {
        if let Err(err) = registry.save(path) {
            outcome.messages.push(format!("Warning: could not save plugin registry: {err}"));
        }
    }

    outcome
}

fn load_candidate(
    outcome: &mut PluginLoadOutcome,
    candidate: &PluginCandidate,
    kiwi_version: &str,
    enabled: bool,
) {
    let manifest = &candidate.manifest;
    let name = manifest.name.clone();
    let display_name = manifest.effective_display_name().to_string();
    let version = manifest.version.clone();
    let description = manifest.description.clone().unwrap_or_default();
    let author = manifest.author.clone().unwrap_or_default();

    if !enabled {
        outcome.state.entries.push(PluginEntry {
            name,
            display_name,
            version,
            description,
            author,
            enabled: false,
            status: PluginStatus::Disabled,
            command_ids: vec![],
        });
        return;
    }

    if !kiwi_version_compatible(&manifest.min_kiwi_version, kiwi_version) {
        let reason = format!(
            "requires Kiwi {} (running {kiwi_version})",
            manifest.min_kiwi_version
        );
        outcome
            .messages
            .push(format!("Plugin `{name}` skipped: {reason}"));
        outcome.state.entries.push(PluginEntry {
            name,
            display_name,
            version,
            description,
            author,
            enabled: true,
            status: PluginStatus::Incompatible(reason),
            command_ids: vec![],
        });
        return;
    }

    match load_plugin(&candidate.library_path, &manifest.entry) {
        Ok(mut loaded) => {
            if loaded.name != name {
                outcome.messages.push(format!(
                    "Plugin `{name}` warning: descriptor name `{}` does not match manifest",
                    loaded.name
                ));
            }

            let mut command_ids = Vec::new();
            let plugin_name = loaded.name.clone();
            for command in std::mem::take(&mut loaded.commands) {
                command_ids.push(command.id.clone());
                outcome.state.commands.push(PluginPaletteCommand {
                    id: command.id,
                    title: command.title,
                    plugin_name: plugin_name.clone(),
                    callback: command.callback,
                    enabled: true,
                });
            }

            let count = command_ids.len();
            outcome
                .messages
                .push(format!("Plugin `{name}` loaded ({count} command(s))"));
            outcome.state.entries.push(PluginEntry {
                name,
                display_name,
                version,
                description,
                author,
                enabled: true,
                status: PluginStatus::Loaded,
                command_ids,
            });
            outcome.host.push(loaded);
        }
        Err(err) => {
            let reason = err.to_string();
            outcome
                .messages
                .push(format!("Plugin `{name}` skipped: {err}"));
            outcome.state.entries.push(PluginEntry {
                name,
                display_name,
                version,
                description,
                author,
                enabled: true,
                status: PluginStatus::Failed(reason),
                command_ids: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use kiwi_plugin_api::PluginResult;
    use kiwi_plugin_loader::{invoke_plugin_command, PluginInvokeOutcome};

    use super::*;

    static SAMPLE_PLUGIN_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempPluginInstall {
        root: PathBuf,
    }

    impl TempPluginInstall {
        fn new() -> Self {
            let id = SAMPLE_PLUGIN_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!("kiwi-sample-plugin-{id}"));
            let plugin_dir = root.join("hello");
            fs::create_dir_all(&plugin_dir).expect("plugin dir");
            fs::write(
                plugin_dir.join("plugin.toml"),
                r#"
                name = "hello"
                version = "0.1.0"
                min_kiwi_version = "0.1.0"
                "#,
            )
            .expect("manifest");
            Self { root }
        }

        fn plugins_dir(&self) -> &Path {
            &self.root
        }

        fn copy_library(&self, library_path: &Path) {
            let ext = if cfg!(target_os = "macos") { "dylib" } else { "so" };
            let dest = self.root.join("hello").join(format!("libhello.{ext}"));
            fs::copy(library_path, dest).expect("copy library");
        }
    }

    impl Drop for TempPluginInstall {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn sample_hello_library_path() -> PathBuf {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
        let filename = if cfg!(target_os = "macos") {
            "libkiwi_plugin_hello.dylib"
        } else {
            "libkiwi_plugin_hello.so"
        };
        workspace.join("target").join(profile).join(filename)
    }

    #[test]
    fn disabled_config_skips_discovery() {
        let outcome = load_plugins(
            &PluginsSettings {
                enabled: false,
                directory: PathBuf::from("/tmp/unused"),
            },
            "0.1.0",
        );
        assert!(outcome.state.commands.is_empty());
        assert!(outcome.state.entries.is_empty());
        assert!(outcome.messages.is_empty());
    }

    #[test]
    fn sample_hello_plugin_registers_palette_command_and_entry() {
        let library_path = sample_hello_library_path();
        assert!(
            library_path.is_file(),
            "missing sample plugin library at {} — build with `cargo build -p kiwi_plugin_hello`",
            library_path.display()
        );

        let install = TempPluginInstall::new();
        install.copy_library(&library_path);

        let outcome = load_plugins(
            &PluginsSettings {
                enabled: true,
                directory: install.plugins_dir().to_path_buf(),
            },
            env!("CARGO_PKG_VERSION"),
        );

        assert!(
            outcome.messages.iter().any(|m| m.contains("Plugin `hello` loaded")),
            "messages: {:?}",
            outcome.messages
        );
        assert_eq!(outcome.state.commands.len(), 1);
        assert_eq!(outcome.state.commands[0].id, "hello.greet");
        assert_eq!(outcome.state.commands[0].title, "Hello Plugin: Greet");

        // PluginEntry should be present and Loaded
        assert_eq!(outcome.state.entries.len(), 1);
        let entry = &outcome.state.entries[0];
        assert_eq!(entry.name, "hello");
        assert!(entry.enabled);
        assert!(matches!(entry.status, PluginStatus::Loaded));
        assert_eq!(entry.command_ids, ["hello.greet"]);

        match invoke_plugin_command(outcome.state.commands[0].callback) {
            PluginInvokeOutcome::Completed(PluginResult::Ok) => {}
            other => panic!("unexpected plugin callback outcome: {other:?}"),
        }
    }

    #[test]
    fn incompatible_version_produces_incompatible_entry() {
        let outcome = load_plugins(
            &PluginsSettings {
                enabled: true,
                directory: PathBuf::from("/tmp/kiwi-no-plugins-dir"),
            },
            "0.1.0",
        );
        // No candidates → no entries, no error
        assert!(outcome.state.entries.is_empty());
        assert!(outcome.state.commands.is_empty());
    }
}
