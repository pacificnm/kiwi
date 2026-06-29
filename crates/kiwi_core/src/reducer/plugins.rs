use std::path::PathBuf;

use crate::events::{AgentEffect, SideEffect};
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
    state.plugins.install_job.start(format!(
        "Installing plugin from {}",
        src_path.display()
    ));
    state.set_dirty();
    vec![SideEffect::PluginInstall { src_path }]
}

pub(super) fn reduce_plugin_remove(state: &mut ReduceView<'_>, name: String) -> Vec<SideEffect> {
    state.set_dirty();
    vec![SideEffect::PluginRemove { name }]
}

pub(super) fn reduce_plugin_reinstall(
    state: &mut ReduceView<'_>,
    src_path: PathBuf,
) -> Vec<SideEffect> {
    state.plugins.install_job.start(format!(
        "Reinstalling plugin from {}",
        src_path.display()
    ));
    state.set_dirty();
    vec![SideEffect::PluginReinstall { src_path }]
}

pub(super) fn reduce_plugin_install_progress(
    state: &mut ReduceView<'_>,
    message: String,
    step: u32,
    total: u32,
) -> Vec<SideEffect> {
    state.plugins.install_job.apply_progress(message, step, total);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_plugin_install_finished(
    state: &mut ReduceView<'_>,
    result: Option<crate::plugins::PluginInstallResult>,
    error: Option<String>,
) -> Vec<SideEffect> {
    let success = result.is_some();
    let summary = if let Some(ref install) = result {
        install
            .messages
            .last()
            .cloned()
            .unwrap_or_else(|| "Plugin installed.".to_string())
    } else {
        error
            .clone()
            .unwrap_or_else(|| "Plugin install failed.".to_string())
    };

    state.plugins.install_job.finish(success, Some(summary.clone()));
    if success {
        state.logs.push_info(summary.clone());
    } else {
        state.notifications.show_toast(summary.clone());
        state.logs.push_info(summary.clone());
    }
    state.set_dirty();

    if let Some(install_result) = result {
        vec![SideEffect::PluginInstallRegister {
            result: install_result,
        }]
    } else {
        vec![SideEffect::PluginInstallFailed]
    }
}

pub(super) fn reduce_set_agent(
    state: &mut ReduceView<'_>,
    command: String,
    args: Vec<String>,
) -> Vec<SideEffect> {
    state.config.agent.command = command.clone();
    state.config.agent.args = args.clone();
    let id = state.agent_manager.active_id();
    state.notifications.show_toast(format!("Switching agent to `{command}`…"));
    state.set_dirty();
    vec![
        SideEffect::PersistAgentConfig { command, args },
        SideEffect::Agent(AgentEffect::Restart(id)),
    ]
}
