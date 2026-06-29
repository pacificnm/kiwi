//! Background service wiring and side-effect execution for the GUI.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use arboard::Clipboard;

use kiwi_core::agent::{
    execute_tool, spawn_claude_stream, ExecutionResult, KiwiTool, StreamCancelHandle,
    ToolParseError,
};
use kiwi_core::diff::spawn_file_diff_load;
use kiwi_core::editor::{
    launch_gui_editor, prepare_editor_launch, run_terminal_editor, EditorLaunchMode,
};
use kiwi_core::events::{
    AgentEffect, AppEvent, EventChannel, FsEffect, GitEffect, GitHubEffect, SearchEffect,
    ShellEffect, SideEffect,
};
use kiwi_core::file_tree::spawn_directory_load;
use kiwi_core::git::spawn_git_refresh;
use kiwi_core::github::{
    spawn_github_auth_check, spawn_github_issue_comment, spawn_github_issue_create,
    spawn_github_issue_create_branch,
    spawn_github_issue_detail_load, spawn_github_issue_label_apply, spawn_github_issue_list_load,
    spawn_github_issue_milestone_assign, spawn_github_open_browser, spawn_github_pr_create,
    spawn_github_pr_detail_load, spawn_github_pr_list_load, spawn_github_pr_merge,
    spawn_github_repo_labels_load, spawn_github_repo_milestones_load,
};
use kiwi_core::preview::spawn_preview_load;
use kiwi_core::config::persist_user_theme;
use kiwi_core::search::{spawn_search, DebounceTimer, SearchCancelHandle, SearchJob};
use kiwi_core::state::{AppState, ReduceView};
use kiwi_core::workspace::{try_merge_save_gui, try_save_from_reduce_view, GuiWorkspaceSnapshot};

use crate::pty::PtyRuntime;

/// Maximum events processed per frame to avoid stalling egui.
pub const MAX_EVENTS_PER_FRAME: usize = 256;

/// Debounce/cancel state for background search jobs (SPEC-007).
#[derive(Debug)]
pub struct SearchRuntime {
    pub debounce: DebounceTimer,
    pub cancel: SearchCancelHandle,
    pub live_generation: Arc<AtomicU64>,
    /// Generation for which the debounce timer was last armed (avoids re-arming every frame).
    armed_generation: Option<u64>,
}

impl Default for SearchRuntime {
    fn default() -> Self {
        Self {
            debounce: DebounceTimer::default(),
            cancel: SearchCancelHandle::default(),
            live_generation: Arc::new(AtomicU64::new(0)),
            armed_generation: None,
        }
    }
}

impl SearchRuntime {
    pub fn sync_debounce(&mut self, state: &AppState) {
        self.live_generation
            .store(state.search.generation, Ordering::Relaxed);
        if state.search.debounce_scheduled {
            if self.armed_generation != Some(state.search.generation) {
                let debounce = Duration::from_millis(state.config.search.debounce_ms);
                self.debounce.schedule(debounce);
                self.armed_generation = Some(state.search.generation);
            }
        } else {
            self.armed_generation = None;
        }
    }

    pub fn poll_debounce(&mut self) -> bool {
        self.debounce.poll_ready()
    }

    pub fn debounce_pending(&self) -> bool {
        self.debounce.remaining().is_some()
    }

    pub fn clear_debounce(&mut self) {
        self.debounce.clear();
        self.armed_generation = None;
    }
}

pub struct ServiceContext<'a> {
    pub state: &'a mut AppState,
    pub events: &'a EventChannel,
    pub pty: &'a mut PtyRuntime,
    pub search: &'a mut SearchRuntime,
    /// Dock layout snapshot used by SaveWorkspace to persist GUI state alongside core state.
    /// `None` in tests or when the service layer is invoked without a live dock.
    pub dock_snapshot: Option<GuiWorkspaceSnapshot>,
}

/// Execute a batch of reducer side effects. Returns `true` when the app should quit.
pub fn execute_gui_effects(ctx: &mut ServiceContext<'_>, effects: Vec<SideEffect>) -> bool {
    for effect in effects {
        if execute_gui_effect(ctx, effect) {
            return true;
        }
    }
    false
}

fn execute_gui_effect(ctx: &mut ServiceContext<'_>, effect: SideEffect) -> bool {
    match effect {
        SideEffect::Quit => {
            ctx.pty.shutdown();
            return true;
        }
        // Both core state and dock layout are saved here, matching KiwiApp::save_workspace
        // (the 30-second timer path). The dock_snapshot carries the live egui_dock state;
        // it is None only in tests / headless contexts (#276).
        SideEffect::SaveWorkspace => {
            let mut view = ReduceView::from_app_state(ctx.state);
            try_save_from_reduce_view(&mut view);
            if let Some(ref snapshot) = ctx.dock_snapshot {
                try_merge_save_gui(
                    &ctx.state.repo_root.clone(),
                    ctx.state.config.workspace.persist,
                    snapshot,
                    &mut ctx.state.logs,
                );
            }
        }
        SideEffect::Git(effect) => match effect {
            GitEffect::SpawnRefresh => {
                if ctx.state.workspace_meta.is_git_repo {
                    spawn_git_refresh(
                        ctx.state.repo_root.clone(),
                        ctx.state.config.git.show_untracked,
                        ctx.events.sender(),
                    );
                }
            }
            GitEffect::SpawnBranchList => {
                if ctx.state.workspace_meta.is_git_repo {
                    kiwi_core::git::spawn_branch_list(
                        ctx.state.repo_root.clone(),
                        ctx.events.sender(),
                    );
                }
            }
            GitEffect::SpawnBranchCheckout { name } => {
                if ctx.state.workspace_meta.is_git_repo {
                    kiwi_core::git::spawn_branch_checkout(
                        ctx.state.repo_root.clone(),
                        name,
                        ctx.events.sender(),
                    );
                }
            }
            GitEffect::SpawnBranchDetail { name } => {
                if ctx.state.workspace_meta.is_git_repo {
                    kiwi_core::git::spawn_branch_detail(
                        ctx.state.repo_root.clone(),
                        name,
                        ctx.events.sender(),
                    );
                }
            }
            _ => {} // #[non_exhaustive] forward-compat
        },
        SideEffect::GitHub(effect) => match effect {
            GitHubEffect::SpawnRefresh => {
                // The reducer never emits SpawnRefresh for GitHubRefresh commands
                // (it emits SpawnAuthCheck directly). This arm exists only for
                // #[non_exhaustive] forward-compat should the variant gain meaning later.
            }
            GitHubEffect::SpawnAuthCheck => {
                spawn_github_auth_check(
                    ctx.state.config.github.command.clone(),
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnIssueList => {
                spawn_github_issue_list_load(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnPrList => {
                spawn_github_pr_list_load(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnIssueDetail { number } => {
                spawn_github_issue_detail_load(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    number,
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnPrDetail { number } => {
                spawn_github_pr_detail_load(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    number,
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnIssueComment { number, body } => {
                spawn_github_issue_comment(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    number,
                    body,
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnIssueCreate { request } => {
                spawn_github_issue_create(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    request,
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnIssueCreateBranch { number } => {
                spawn_github_issue_create_branch(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    number,
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnRepoLabels => {
                spawn_github_repo_labels_load(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnIssueLabelApply { number, labels } => {
                spawn_github_issue_label_apply(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    number,
                    labels,
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnRepoMilestones => {
                spawn_github_repo_milestones_load(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnIssueMilestoneAssign {
                number,
                milestone_title,
            } => {
                spawn_github_issue_milestone_assign(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    number,
                    milestone_title,
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnOpenBrowser { target } => {
                spawn_github_open_browser(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    target,
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnPrCreate { request } => {
                spawn_github_pr_create(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    request,
                    ctx.events.sender(),
                );
            }
            GitHubEffect::SpawnPrMerge { number } => {
                spawn_github_pr_merge(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.github.command.clone(),
                    number,
                    ctx.events.sender(),
                );
            }
            _ => {} // #[non_exhaustive] forward-compat
        },
        SideEffect::Shell(effect) => match effect {
            ShellEffect::Write(data) => {
                let _ = ctx.pty.write_shell(&data);
            }
            ShellEffect::Resize { cols, rows } => {
                if ctx.pty.resize_shell(cols, rows) {
                    ctx.state.shell.apply_resize(cols, rows);
                }
            }
            _ => {} // #[non_exhaustive] forward-compat
        },
        SideEffect::Agent(effect) => match effect {
            AgentEffect::StreamRequest(id) => {
                spawn_claude_stream_effect(ctx, id);
            }
            AgentEffect::CancelStream(id) => {
                ctx.pty.cancel_stream(id);
            }
            AgentEffect::ExecuteTool { agent_id, tool_use_id, tool_name, input_json } => {
                handle_execute_tool(ctx, agent_id, tool_use_id, &tool_name, &input_json);
            }
            _ => {} // #[non_exhaustive] forward-compat
        },
        SideEffect::Fs(effect) => match effect {
            FsEffect::LoadDirectoryChildren(path) => {
                spawn_directory_load(path, ctx.events.sender());
            }
            FsEffect::LoadPreviewFile(path) => {
                spawn_preview_load(
                    path,
                    ctx.state.config.preview.max_size_bytes,
                    ctx.events.sender(),
                );
            }
            FsEffect::LoadFileDiff { path, source } => {
                spawn_file_diff_load(
                    ctx.state.repo_root.clone(),
                    path,
                    source,
                    ctx.state.config.diff.context_lines,
                    ctx.events.sender(),
                );
            }
            FsEffect::LaunchEditor { path, line } => {
                spawn_editor_launch(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.editor.clone(),
                    path,
                    line,
                    ctx.events.sender(),
                );
            }
            _ => {} // #[non_exhaustive] forward-compat
        },
        SideEffect::Search(effect) => match effect {
            SearchEffect::Cancel => {
                ctx.search.cancel.cancel();
                ctx.search.clear_debounce();
                ctx.search
                    .live_generation
                    .store(ctx.state.search.generation, Ordering::Relaxed);
            }
            SearchEffect::Run { mode, query, generation } => {
                ctx.search.cancel.clear();
                spawn_search(
                    SearchJob {
                        mode,
                        query,
                        generation,
                        repo_root: ctx.state.repo_root.clone(),
                        rg_command: ctx.state.config.search.command.clone(),
                    },
                    ctx.events.sender(),
                    ctx.search.live_generation.clone(),
                    ctx.search.cancel.clone(),
                );
            }
            _ => {} // #[non_exhaustive] forward-compat
        },
        SideEffect::CopyToClipboard(text) => {
            match Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
                Ok(()) => {}
                Err(err) => {
                    ctx.state.notifications.show_toast(format!("Copy failed: {err}"));
                    ctx.state.dirty = true;
                }
            }
        }
        // arboard reads the OS clipboard directly — egui::Context is not required (#273).
        SideEffect::PasteFromClipboard => {
            match Clipboard::new().and_then(|mut cb| cb.get_text()) {
                Ok(text) => {
                    let effects = kiwi_core::reducer::reduce(
                        &mut ReduceView::from_app_state(ctx.state),
                        kiwi_core::events::AppEvent::Command(
                            kiwi_core::events::AppCommand::PasteText(text),
                        ),
                    );
                    // PasteText only produces Shell/Agent write effects; execute inline.
                    for inner in effects {
                        execute_gui_effect(ctx, inner);
                    }
                }
                Err(err) => {
                    ctx.state.notifications.show_toast(format!("Paste failed: {err}"));
                    ctx.state.dirty = true;
                }
            }
        }
        SideEffect::PluginSetEnabled { name, enabled } => {
            if let Some(path) = kiwi_core::plugins::default_registry_path() {
                let (mut registry, _) = kiwi_core::plugins::PluginRegistry::load(&path);
                // Auto-register plugins that are on disk but not yet in the registry.
                kiwi_core::plugins::ensure_registered(&mut registry, &name);
                if enabled {
                    registry.enable(&name);
                } else {
                    registry.disable(&name);
                }
                if let Err(e) = registry.save(&path) {
                    ctx.state
                        .notifications
                        .show_toast(format!("Failed to save plugin registry: {e}"));
                    ctx.state.dirty = true;
                } else {
                    ctx.state.notifications.show_toast(format!(
                        "Plugin `{name}` {}. Restart to apply.",
                        if enabled { "enabled" } else { "disabled" }
                    ));
                    refresh_available_plugins(ctx);
                }
            }
        }
        SideEffect::PluginInstall { src_path } => {
            spawn_plugin_install(ctx, src_path, false);
        }
        SideEffect::PluginRemove { name } => {
            handle_plugin_remove(ctx, &name);
        }
        SideEffect::PluginReinstall { src_path } => {
            spawn_plugin_install(ctx, src_path, true);
        }
        SideEffect::PluginInstallRegister { result } => {
            register_installed_plugin(ctx, result, "Plugin installed.");
        }
        SideEffect::PluginInstallFailed => {
            refresh_available_plugins(ctx);
        }
        SideEffect::PersistAgentMode { provider, model, api_key_env, api_url, api_key } => {
            let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
            if let Some(home) = home {
                if let Err(e) = kiwi_core::config::persist_user_agent_mode(
                    &home, &provider, &model,
                    api_key_env.as_deref(), api_url.as_deref(), api_key.as_deref(),
                ) {
                    ctx.state.notifications.show_toast(format!("Failed to save agent mode: {e}"));
                    ctx.state.dirty = true;
                }
            }
        }
        SideEffect::PersistAgentConfig { command, args } => {
            let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
            match home {
                Some(home) => {
                    if let Err(e) = kiwi_core::config::persist_user_agent(&home, &command, &args) {
                        ctx.state
                            .notifications
                            .show_toast(format!("Failed to save agent config: {e}"));
                        ctx.state.dirty = true;
                    }
                }
                None => {
                    ctx.state
                        .notifications
                        .show_toast("Cannot save agent config: HOME not set");
                    ctx.state.dirty = true;
                }
            }
        }
        SideEffect::PersistUserTheme { name } => {
            let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
            match home {
                Some(home) => match persist_user_theme(&home, &name) {
                    Ok(()) => {}
                    Err(err) => {
                        ctx.state
                            .notifications
                            .show_toast(format!("Failed to save theme to config: {err}"));
                        ctx.state.dirty = true;
                    }
                },
                None => {
                    ctx.state
                        .notifications
                        .show_toast("Cannot save theme: HOME not set");
                    ctx.state.dirty = true;
                }
            }
        }
        // SideEffect is #[non_exhaustive]; future variants added to kiwi_core are unhandled here
        // until explicitly implemented above.
        _ => {}
    }
    false
}

fn handle_execute_tool(
    ctx: &mut ServiceContext<'_>,
    agent_id: kiwi_core::agent::AgentId,
    tool_use_id: String,
    tool_name: &str,
    input_json: &str,
) {
    let input: serde_json::Value = serde_json::from_str(input_json).unwrap_or_default();

    let tool = match KiwiTool::from_tool_use(tool_name, &input) {
        Ok(t) => t,
        Err(ToolParseError(msg)) => {
            let _ = ctx.events.sender().send(AppEvent::AgentToolResult {
                agent_id,
                tool_use_id,
                content: format!("Unknown tool '{tool_name}': {msg}"),
                is_error: true,
            });
            return;
        }
    };

    // RunBash is handled inline — it needs PTY write access before spawning a thread.
    if let KiwiTool::RunBash { ref command } = tool {
        let bytes: Vec<u8> = format!("{command}\n").into_bytes();
        let _ = ctx.pty.write_shell(&bytes);
        let content = format!(
            "Command sent to Terminal panel: `{command}`\nSwitch to the Terminal tab to see output."
        );
        let _ = ctx.events.sender().send(AppEvent::AgentToolResult {
            agent_id,
            tool_use_id,
            content,
            is_error: false,
        });
        return;
    }

    // All other tools run on a background thread (may involve disk I/O or subprocesses).
    let repo_root = ctx.state.repo_root.clone();
    let sender = ctx.events.sender();
    std::thread::spawn(move || {
        let (content, is_error) = match execute_tool(&tool, &repo_root) {
            ExecutionResult::Done { content, is_error } => (content, is_error),
            ExecutionResult::RunBash { .. } => unreachable!("RunBash handled above"),
        };
        let _ = sender.send(AppEvent::AgentToolResult {
            agent_id,
            tool_use_id,
            content,
            is_error,
        });
    });
}

fn spawn_claude_stream_effect(ctx: &mut ServiceContext<'_>, agent_id: kiwi_core::agent::AgentId) {
    use kiwi_core::agent::AgentProvider;

    // Snapshot provider, model, messages, and api_url before any mutable borrows.
    let snapshot = ctx
        .state
        .agent_manager
        .pty(agent_id)
        .and_then(|s| s.chat.as_ref())
        .map(|chat| (chat.provider.clone(), chat.messages.clone(), chat.model.clone()));

    let Some((provider, messages, model)) = snapshot else { return };
    if messages.is_empty() {
        return;
    }

    // Clear any previous error now that we are starting a new stream.
    if let Some(pty) = ctx.state.agent_manager.pty_mut(agent_id) {
        if let Some(chat) = &mut pty.chat {
            chat.error = None;
        }
    }

    let cancel = StreamCancelHandle::default();
    ctx.pty.register_stream(agent_id, cancel.clone());
    let sender = ctx.events.sender();

    // Look up per-provider settings; fall back to legacy flat fields.
    let active_name = ctx.state.config.agent.active_provider.as_deref().unwrap_or("claude");
    let provider_settings = ctx.state.config.agent.providers.get(active_name).cloned();

    let resolve_api_key = |default_env: &str| -> String {
        let env_var = provider_settings.as_ref().map(|p| p.api_key_env.as_str())
            .unwrap_or(default_env);
        let config_key = provider_settings.as_ref().and_then(|p| p.api_key.clone())
            .or_else(|| ctx.state.config.agent.api_key.clone());
        std::env::var(env_var)
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .or_else(|| config_key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty()))
            .unwrap_or_default()
    };

    match provider {
        AgentProvider::Ollama => {
            let api_url = provider_settings
                .as_ref()
                .and_then(|p| p.api_url.clone())
                .or_else(|| ctx.state.config.agent.api_url.clone())
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            kiwi_core::agent::spawn_ollama_stream(agent_id, api_url, model, messages, cancel, sender);
        }
        AgentProvider::OpenAI => {
            let api_key = resolve_api_key("OPENAI_API_KEY");
            if api_key.is_empty() {
                let _ = sender.send(AppEvent::AgentApiError {
                    agent_id,
                    message: "API key not found. Export OPENAI_API_KEY in your shell, \
                              or set api_key under [agent.providers.openai] in config.toml"
                        .to_string(),
                });
                return;
            }
            kiwi_core::agent::spawn_openai_stream(agent_id, api_key, model, messages, cancel, sender);
        }
        _ => {
            let api_key = resolve_api_key("ANTHROPIC_API_KEY");
            if api_key.is_empty() {
                let _ = sender.send(AppEvent::AgentApiError {
                    agent_id,
                    message: "API key not found. Export ANTHROPIC_API_KEY in your shell, \
                              or set api_key under [agent.providers.claude] in config.toml"
                        .to_string(),
                });
                return;
            }
            spawn_claude_stream(agent_id, api_key, model, messages, cancel, sender);
        }
    }
}

fn refresh_available_plugins(ctx: &mut ServiceContext<'_>) {
    let plugins_src = ctx.state.repo_root.join("plugins");
    if plugins_src.is_dir() {
        let registry = kiwi_core::plugins::default_registry_path()
            .map(|p| kiwi_core::plugins::PluginRegistry::load(&p).0)
            .unwrap_or_default();
        ctx.state.plugins.available =
            kiwi_core::plugins::scan_available_plugins(&[&plugins_src], &registry);
        ctx.state.dirty = true;
    }
}

fn plugin_user_message(
    result: &kiwi_core::plugins::PluginInstallResult,
    fallback: &str,
) -> String {
    result
        .messages
        .last()
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

fn register_installed_plugin(
    ctx: &mut ServiceContext<'_>,
    result: kiwi_core::plugins::PluginInstallResult,
    fallback_message: &str,
) {
    let name = result.entry.name.clone();
    let summary = plugin_user_message(&result, fallback_message);

    if let Some(path) = kiwi_core::plugins::default_registry_path() {
        let (mut registry, _) = kiwi_core::plugins::PluginRegistry::load(&path);
        registry.register(result.entry);
        match registry.save(&path) {
            Ok(()) => {
                ctx.state.logs.push_info(summary.clone());
                ctx.state.notifications.show_toast(summary);
            }
            Err(e) => {
                let message =
                    format!("Plugin `{name}` installed on disk but registry save failed: {e}");
                ctx.state.logs.push_info(message.clone());
                ctx.state.notifications.show_toast(message);
            }
        }
    } else {
        let message = format!("Plugin `{name}` installed on disk (HOME not set — registry not updated)");
        ctx.state.logs.push_info(message.clone());
        ctx.state.notifications.show_toast(message);
    }

    refresh_available_plugins(ctx);
}

fn spawn_plugin_install(
    ctx: &mut ServiceContext<'_>,
    src_path: std::path::PathBuf,
    reinstall: bool,
) {
    ctx.state.dirty = true;
    let sender = ctx.events.sender();

    std::thread::spawn(move || {
        let mut send_progress = |update: kiwi_core::plugins::InstallProgressUpdate| {
            let _ = sender.send(AppEvent::PluginInstallProgress {
                message: update.message,
                step: update.step,
                total: update.total,
            });
        };

        let result = if reinstall {
            kiwi_core::plugins::reinstall_plugin_from_source_with_progress(
                &src_path,
                &mut send_progress,
            )
        } else {
            kiwi_core::plugins::install_plugin_from_source_with_progress(
                &src_path,
                &mut send_progress,
            )
            .or_else(|err| {
                if err.contains("already exists") {
                    kiwi_core::plugins::reinstall_plugin_from_source_with_progress(
                        &src_path,
                        &mut send_progress,
                    )
                } else {
                    Err(err)
                }
            })
        };

        let event = match result {
            Ok(install_result) => AppEvent::PluginInstallFinished {
                result: Some(install_result),
                error: None,
            },
            Err(error) => AppEvent::PluginInstallFinished {
                result: None,
                error: Some(error),
            },
        };
        let _ = sender.send(event);
    });
}

fn handle_plugin_remove(ctx: &mut ServiceContext<'_>, name: &str) {
    let disk_result = kiwi_core::plugins::remove_plugin(name);

    if let Some(path) = kiwi_core::plugins::default_registry_path() {
        let (mut registry, _) = kiwi_core::plugins::PluginRegistry::load(&path);
        registry.remove(name);
        if let Err(e) = registry.save(&path) {
            let message = format!("Plugin `{name}` removed from disk but registry save failed: {e}");
            ctx.state.logs.push_info(message.clone());
            ctx.state.notifications.show_toast(message);
        }
    }

    refresh_available_plugins(ctx);

    match disk_result {
        Ok(result) => {
            let summary = result
                .messages
                .last()
                .cloned()
                .unwrap_or_else(|| format!("Plugin `{name}` removed."));
            ctx.state.logs.push_info(summary.clone());
            ctx.state.notifications.show_toast(summary);
        }
        Err(e) => {
            let message = format!("Plugin `{name}` removed from registry: {e}");
            ctx.state.logs.push_info(message.clone());
            ctx.state.notifications.show_toast(message);
        }
    }
    ctx.state.dirty = true;
}

fn spawn_editor_launch(
    repo_root: PathBuf,
    settings: kiwi_core::config::EditorSettings,
    path: PathBuf,
    line: Option<u32>,
    sender: kiwi_core::events::EventSender,
) {
    std::thread::spawn(move || {
        let event = match prepare_editor_launch(&repo_root, &settings, &path, line) {
            Ok(prepared) => match prepared.mode {
                EditorLaunchMode::Gui => match launch_gui_editor(&prepared) {
                    Ok(result) => AppEvent::EditorLaunched {
                        path: result.path,
                        command: result.command,
                    },
                    Err(err) => AppEvent::EditorLaunchFailed {
                        path: prepared.path,
                        error: err.user_message(),
                        show_modal: err.is_command_not_found(),
                    },
                },
                EditorLaunchMode::Terminal => match run_terminal_editor(&repo_root, &prepared) {
                    Ok(result) => AppEvent::EditorLaunched {
                        path: result.path,
                        command: result.command,
                    },
                    Err(err) => AppEvent::EditorLaunchFailed {
                        path: prepared.path,
                        error: err.user_message(),
                        show_modal: err.is_command_not_found(),
                    },
                },
            },
            Err(err) => AppEvent::EditorLaunchFailed {
                path,
                error: err.user_message(),
                show_modal: err.is_command_not_found(),
            },
        };
        let _ = sender.send(event);
    });
}

/// Drain pending events, apply reducers, and execute resulting side effects.
///
/// `dock_snapshot` is passed to `ServiceContext` so that `SideEffect::SaveWorkspace` can
/// persist the dock layout alongside the core workspace state. Pass `None` in tests or
/// when no live dock is available.
///
/// Returns `(should_quit, event_count)`.
pub fn process_pending_events(
    state: &mut AppState,
    events: &mut EventChannel,
    pty: &mut PtyRuntime,
    search: &mut SearchRuntime,
    dock_snapshot: Option<GuiWorkspaceSnapshot>,
) -> (bool, usize) {
    let pending: Vec<AppEvent> = events
        .drain_coalesced()
        .into_iter()
        .take(MAX_EVENTS_PER_FRAME)
        .collect();
    let count = pending.len();
    let mut should_quit = false;

    for event in pending {
        let effects = kiwi_core::reducer::reduce(&mut ReduceView::from_app_state(state), event);
        let mut ctx = ServiceContext {
            state,
            events,
            pty,
            search,
            dock_snapshot: dock_snapshot.clone(),
        };
        if execute_gui_effects(&mut ctx, effects) {
            should_quit = true;
            break;
        }
    }

    search.sync_debounce(state);
    (should_quit, count)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::events::AppEvent;
    use kiwi_core::git::{GitFileEntry, GitFileStatus};
    use kiwi_core::state::{AppState, ViewportMetrics};
    use kiwi_core::status_bar::compute_status_bar;
    use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use crate::pty::PtyRuntime;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        )
    }

    #[test]
    fn git_status_updated_updates_status_bar_snapshot() {
        let mut state = test_state();
        let mut events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut search = SearchRuntime::default();

        events
            .sender()
            .send(AppEvent::GitStatusUpdated {
                branch: Some("main".to_string()),
                remote_repo: Some("org/repo".to_string()),
                ahead: 0,
                behind: 0,
                file_entries: vec![GitFileEntry {
                    path: "src/main.rs".to_string(),
                    status: GitFileStatus::Modified,
                }],
                error: None,
            })
            .expect("send");

        let (quit, count) = process_pending_events(&mut state, &mut events, &mut pty, &mut search, None);
        assert!(!quit);
        assert_eq!(count, 1);

        let snapshot = compute_status_bar(&state);
        assert_eq!(snapshot.branch, "main");
        assert_eq!(snapshot.git_label, "1 Modified");
        assert_eq!(snapshot.remote_repo.as_deref(), Some("org/repo"));
    }

    #[test]
    fn spawn_git_refresh_effect_is_noop_for_non_git_repo() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = false;
        let events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut search = SearchRuntime::default();
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
            pty: &mut pty,
            search: &mut search,
            dock_snapshot: None,
        };

        let quit = execute_gui_effects(&mut ctx, vec![SideEffect::Git(GitEffect::SpawnRefresh)]);
        assert!(!quit);
    }

    #[test]
    fn spawn_github_auth_check_effect_does_not_quit() {
        let mut state = test_state();
        let events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut search = SearchRuntime::default();
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
            pty: &mut pty,
            search: &mut search,
            dock_snapshot: None,
        };

        let quit = execute_gui_effects(&mut ctx, vec![SideEffect::GitHub(GitHubEffect::SpawnAuthCheck)]);
        assert!(!quit);
    }

    #[test]
    fn sync_debounce_does_not_extend_timer_every_frame() {
        use std::thread;
        use std::time::Duration;

        use kiwi_core::search::SearchState;

        let mut state = test_state();
        state.search = SearchState {
            query: "main".to_string(),
            generation: 1,
            debounce_scheduled: true,
            ..SearchState::default()
        };
        let mut search = SearchRuntime::default();
        search.sync_debounce(&state);
        assert!(search.debounce_pending());

        thread::sleep(Duration::from_millis(50));
        search.sync_debounce(&state);
        thread::sleep(Duration::from_millis(160));
        assert!(
            search.poll_debounce(),
            "timer should fire without being pushed forward by repeated sync"
        );

        state.search.debounce_scheduled = false;
        search.sync_debounce(&state);
        assert!(!search.debounce_pending());
    }

    #[test]
    fn run_search_side_effect_does_not_quit() {
        use kiwi_core::search::SearchMode;

        let mut state = test_state();
        let events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut search = SearchRuntime::default();
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
            pty: &mut pty,
            search: &mut search,
            dock_snapshot: None,
        };

        let quit = execute_gui_effects(
            &mut ctx,
            vec![SideEffect::Search(SearchEffect::Run {
                mode: SearchMode::Files,
                query: "main".to_string(),
                generation: 1,
            })],
        );
        assert!(!quit);
    }

    #[test]
    fn github_refresh_via_command_reaches_auth_check_side_effect() {
        use kiwi_core::events::AppCommand;
        use kiwi_core::navigation::{FocusTarget, LeftNavTab, MainTab};

        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;
        state.navigation.main_tab = MainTab::Issues;
        state.navigation.focus = FocusTarget::Left;
        let events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut search = SearchRuntime::default();

        let effects = kiwi_core::reducer::reduce(
            &mut ReduceView::from_app_state(&mut state),
            AppEvent::Command(AppCommand::GitHubRefresh),
        );
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::GitHub(GitHubEffect::SpawnAuthCheck)
        )));
        // Reducer contract: GitHubRefresh must NOT emit SpawnRefresh (see #274).
        assert!(!effects.iter().any(|effect| matches!(
            effect,
            SideEffect::GitHub(GitHubEffect::SpawnRefresh)
        )));

        let quit = execute_gui_effects(
            &mut ServiceContext {
                state: &mut state,
                events: &events,
                pty: &mut pty,
                search: &mut search,
                dock_snapshot: None,
            },
            effects,
        );
        assert!(!quit);
        assert!(state.github.loading);
    }

    #[test]
    fn save_workspace_effect_does_not_quit() {
        // Verifies both save paths are symmetric (#276): dock_snapshot=None exercises
        // the core-state path; production passes Some(...) which also saves dock layout.
        let mut state = test_state();
        let events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut search = SearchRuntime::default();
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
            pty: &mut pty,
            search: &mut search,
            dock_snapshot: None,
        };
        let quit = execute_gui_effects(&mut ctx, vec![SideEffect::SaveWorkspace]);
        assert!(!quit);
    }

    #[test]
    fn copy_to_clipboard_effect_does_not_quit() {
        let mut state = test_state();
        let events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut search = SearchRuntime::default();
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
            pty: &mut pty,
            search: &mut search,
            dock_snapshot: None,
        };
        let quit =
            execute_gui_effects(&mut ctx, vec![SideEffect::CopyToClipboard("hello".to_string())]);
        assert!(!quit);
    }

    #[test]
    fn paste_from_clipboard_effect_does_not_quit() {
        let mut state = test_state();
        let events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut search = SearchRuntime::default();
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
            pty: &mut pty,
            search: &mut search,
            dock_snapshot: None,
        };
        // May fail to open clipboard in headless CI; either way it must not panic or quit.
        let quit = execute_gui_effects(&mut ctx, vec![SideEffect::PasteFromClipboard]);
        assert!(!quit);
    }

    #[test]
    fn persist_user_theme_effect_does_not_quit() {
        let mut state = test_state();
        let events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut search = SearchRuntime::default();
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
            pty: &mut pty,
            search: &mut search,
            dock_snapshot: None,
        };
        // May show a toast if HOME is unset; must not panic or quit.
        let quit = execute_gui_effects(
            &mut ctx,
            vec![SideEffect::PersistUserTheme { name: "kiwi-dark".to_string() }],
        );
        assert!(!quit);
    }
}
