use std::path::PathBuf;

use crate::agent::{AgentStatus, ChatSession};
use crate::config::AgentMode;
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
    mode: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    api_key_env: Option<String>,
    api_url: Option<String>,
    api_key: Option<String>,
) -> Vec<SideEffect> {
    let is_api = mode.as_deref() == Some("api");
    let new_mode = if is_api { AgentMode::Api } else { AgentMode::Pty };

    state.config.agent.command = command.clone();
    state.config.agent.args = args.clone();
    state.config.agent.mode = new_mode;

    let id = state.agent_manager.active_id();
    state.set_dirty();

    if is_api {
        let model_str = model.clone().unwrap_or_else(|| state.config.agent.model.clone());
        let provider_str = provider.clone().unwrap_or_else(|| "claude".to_string());

        // Update per-provider settings in the resolved config map.
        let entry = state.config.agent.providers
            .entry(provider_str.clone())
            .or_insert_with(|| crate::config::ProviderSettings {
                api_key_env: "ANTHROPIC_API_KEY".to_string(),
                api_key: None,
                model: model_str.clone(),
                api_url: None,
            });
        entry.model = model_str.clone();
        if let Some(ref env) = api_key_env { entry.api_key_env = env.clone(); }
        if let Some(ref url) = api_url     { entry.api_url = Some(url.clone()); }
        // User-supplied key from Settings UI — update immediately so it's available for the session.
        if let Some(ref key) = api_key     { entry.api_key = Some(key.clone()); }

        state.config.agent.active_provider = Some(provider_str.clone());
        // Keep legacy flat fields in sync.
        state.config.agent.model = model_str.clone();

        let agent_provider = match provider_str.as_str() {
            "ollama" => crate::agent::AgentProvider::Ollama,
            "openai" => crate::agent::AgentProvider::OpenAI,
            _ => crate::agent::AgentProvider::Claude,
        };

        if let Some(pty) = state.agent_manager.pty_mut(id) {
            pty.chat = Some(ChatSession {
                model: model_str.clone(),
                provider: agent_provider,
                status: AgentStatus::Idle,
                ..ChatSession::default()
            });
        }

        let display = provider.as_deref().unwrap_or("API");
        state.notifications.show_toast(format!("Switched to {display} agent (native chat)."));

        // Carry forward any api_key already stored for this provider so
        // switching agents never drops a key that was previously persisted.
        let existing_api_key = state.config.agent.providers
            .get(&provider_str)
            .and_then(|p| p.api_key.clone());

        vec![SideEffect::PersistAgentMode {
            provider: provider_str,
            model: model_str,
            api_key_env,
            api_url,
            api_key: existing_api_key,
        }]
    } else {
        if let Some(pty) = state.agent_manager.pty_mut(id) {
            pty.chat = None;
        }
        state.notifications.show_toast(format!("Switching agent to `{command}`…"));
        vec![
            SideEffect::PersistAgentConfig { command, args },
            SideEffect::Agent(AgentEffect::Restart(id)),
        ]
    }
}
