//! Background service wiring and side-effect execution for the GUI.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use arboard::Clipboard;

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
    spawn_github_auth_check, spawn_github_issue_comment, spawn_github_issue_create_branch,
    spawn_github_issue_detail_load, spawn_github_issue_label_apply, spawn_github_issue_list_load,
    spawn_github_open_browser, spawn_github_pr_create, spawn_github_pr_detail_load,
    spawn_github_pr_list_load, spawn_github_pr_merge, spawn_github_repo_labels_load,
};
use kiwi_core::preview::spawn_preview_load;
use kiwi_core::config::persist_user_theme;
use kiwi_core::search::{spawn_search, DebounceTimer, SearchCancelHandle, SearchJob};
use kiwi_core::state::{AppState, ReduceView};
use kiwi_core::workspace::{try_merge_save_gui, try_save_from_reduce_view, GuiWorkspaceSnapshot};

use crate::pty::PtyRuntime;

/// Maximum events processed per frame to avoid stalling egui.
pub const MAX_EVENTS_PER_FRAME: usize = 64;

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
            _ => {}
        },
        SideEffect::GitHub(effect) => match effect {
            GitHubEffect::SpawnRefresh => {
                // Reducer refresh path emits SpawnAuthCheck; keep for parity with TUI.
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
            _ => {}
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
            _ => {}
        },
        SideEffect::Agent(effect) => match effect {
            AgentEffect::Spawn(id) => {
                let repo_root = ctx.state.repo_root.clone();
                let agent_settings = ctx.state.config.agent.clone();
                ctx.pty.spawn_agent(
                    id,
                    &repo_root,
                    &agent_settings,
                    ctx.state,
                    ctx.events.sender(),
                );
            }
            AgentEffect::Restart(id) => {
                let repo_root = ctx.state.repo_root.clone();
                let config = ctx.state.config.clone();
                ctx.pty.restart_agent(
                    id,
                    &repo_root,
                    &config,
                    ctx.state,
                    ctx.events.sender(),
                );
            }
            AgentEffect::Write(data) => {
                let id = ctx.state.agent_manager.active_id();
                let _ = ctx.pty.write_agent(id, &data);
            }
            _ => {}
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
            _ => {}
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
            _ => {}
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
        SideEffect::PasteFromClipboard => {
            match Clipboard::new().and_then(|mut cb| cb.get_text()) {
                Ok(text) => {
                    let effects = kiwi_core::reducer::reduce(
                        &mut ReduceView::from_app_state(ctx.state),
                        kiwi_core::events::AppEvent::Command(
                            kiwi_core::events::AppCommand::PasteText(text),
                        ),
                    );
                    // execute resulting effects inline (PasteText only produces shell/agent writes)
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
            vec![SideEffect::PersistUserTheme { name: "dark".to_string() }],
        );
        assert!(!quit);
    }
}
