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
    use std::path::PathBuf;

    use super::*;

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
}
