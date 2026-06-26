use kiwi_plugin_api::kiwi_version_compatible;
use kiwi_plugin_loader::{load_plugin, PluginHost};

use crate::config::PluginsSettings;
use crate::state::{PluginPaletteCommand, PluginsState};

use super::discovery::{discover_plugins, PluginCandidate};

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

    let (candidates, discovery_warnings) = discover_plugins(&config.directory);
    outcome.messages.extend(discovery_warnings);

    for candidate in candidates {
        load_candidate(&mut outcome, &candidate, kiwi_version);
    }

    outcome
}

fn load_candidate(
    outcome: &mut PluginLoadOutcome,
    candidate: &PluginCandidate,
    kiwi_version: &str,
) {
    let name = &candidate.manifest.name;
    if !kiwi_version_compatible(&candidate.manifest.min_kiwi_version, kiwi_version) {
        outcome.messages.push(format!(
            "Plugin `{name}` skipped: requires Kiwi {} (running {kiwi_version})",
            candidate.manifest.min_kiwi_version
        ));
        return;
    }

    match load_plugin(&candidate.library_path, &candidate.manifest.entry) {
        Ok(mut loaded) => {
            if loaded.name != *name {
                outcome.messages.push(format!(
                    "Plugin `{name}` warning: descriptor name `{}` does not match manifest",
                    loaded.name
                ));
            }

            let command_count = loaded.commands.len();
            let plugin_name = loaded.name.clone();
            for command in std::mem::take(&mut loaded.commands) {
                outcome.state.commands.push(PluginPaletteCommand {
                    id: command.id,
                    title: command.title,
                    plugin_name: plugin_name.clone(),
                    callback: command.callback,
                    enabled: true,
                });
            }

            outcome.messages.push(format!(
                "Plugin `{name}` loaded ({command_count} command(s))"
            ));
            outcome.host.push(loaded);
        }
        Err(err) => outcome
            .messages
            .push(format!("Plugin `{name}` skipped: {err}")),
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
            let ext = if cfg!(target_os = "macos") {
                "dylib"
            } else {
                "so"
            };
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
        assert!(outcome.messages.is_empty());
    }

    #[test]
    fn sample_hello_plugin_registers_palette_command() {
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
            outcome
                .messages
                .iter()
                .any(|message| message.contains("Plugin `hello` loaded")),
            "messages: {:?}",
            outcome.messages
        );
        assert_eq!(outcome.state.commands.len(), 1);
        assert_eq!(outcome.state.commands[0].id, "hello.greet");
        assert_eq!(outcome.state.commands[0].title, "Hello Plugin: Greet");

        match invoke_plugin_command(outcome.state.commands[0].callback) {
            PluginInvokeOutcome::Completed(PluginResult::Ok) => {}
            other => panic!("unexpected plugin callback outcome: {other:?}"),
        }
    }
}
