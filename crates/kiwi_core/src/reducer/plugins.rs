use std::path::PathBuf;

use crate::events::SideEffect;
use crate::state::{PluginStatus, ReduceView};

pub(super) fn reduce_plugin_set_enabled(
    state: &mut ReduceView<'_>,
    name: String,
    enabled: bool,
) -> Vec<SideEffect> {
    if let Some(entry) = state.plugins.entries.iter_mut().find(|e| e.name == name) {
        entry.enabled = enabled;
        if !enabled {
            entry.status = PluginStatus::Disabled;
        } else if matches!(entry.status, PluginStatus::Disabled) {
            entry.status = PluginStatus::Loaded;
        }
    }
    state.notifications.show_toast(format!(
        "Plugin `{name}` {}. Restart to apply.",
        if enabled { "enabled" } else { "disabled" }
    ));
    state.set_dirty();
    vec![SideEffect::PluginSetEnabled { name, enabled }]
}

pub(super) fn reduce_plugin_install(
    state: &mut ReduceView<'_>,
    src_path: PathBuf,
) -> Vec<SideEffect> {
    state.plugins.install_path_input.clear();
    state.set_dirty();
    vec![SideEffect::PluginInstall { src_path }]
}

pub(super) fn reduce_set_agent(
    state: &mut ReduceView<'_>,
    command: String,
    args: Vec<String>,
) -> Vec<SideEffect> {
    state.config.agent.command = command.clone();
    state.config.agent.args = args.clone();
    state.notifications.show_toast(
        format!("Agent set to `{command}`. Restart the Agent panel to apply.")
    );
    state.set_dirty();
    vec![SideEffect::PersistAgentConfig { command, args }]
}
