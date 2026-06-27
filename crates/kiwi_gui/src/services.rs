//! Background service wiring and side-effect execution for the GUI.

use std::path::PathBuf;

use kiwi_core::diff::spawn_file_diff_load;
use kiwi_core::editor::{
    launch_gui_editor, prepare_editor_launch, run_terminal_editor, EditorLaunchMode,
};
use kiwi_core::events::{AppEvent, EventChannel, SideEffect};
use kiwi_core::file_tree::spawn_directory_load;
use kiwi_core::git::spawn_git_refresh;
use kiwi_core::github::{
    spawn_github_auth_check, spawn_github_issue_comment, spawn_github_issue_create_branch,
    spawn_github_issue_detail_load, spawn_github_issue_label_apply, spawn_github_issue_list_load,
    spawn_github_open_browser, spawn_github_pr_create, spawn_github_pr_detail_load,
    spawn_github_pr_list_load, spawn_github_pr_merge, spawn_github_repo_labels_load,
};
use kiwi_core::preview::spawn_preview_load;
use kiwi_core::state::{AppState, ReduceView};
use kiwi_core::workspace::try_save_from_reduce_view;

use crate::pty::PtyRuntime;

/// Maximum events processed per frame to avoid stalling egui.
pub const MAX_EVENTS_PER_FRAME: usize = 64;

pub struct ServiceContext<'a> {
    pub state: &'a mut AppState,
    pub events: &'a EventChannel,
    pub pty: &'a mut PtyRuntime,
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
        SideEffect::SpawnGitRefresh => {
            if ctx.state.workspace_meta.is_git_repo {
                spawn_git_refresh(
                    ctx.state.repo_root.clone(),
                    ctx.state.config.git.show_untracked,
                    ctx.events.sender(),
                );
            }
        }
        SideEffect::LoadDirectoryChildren(path) => {
            spawn_directory_load(path, ctx.events.sender());
        }
        SideEffect::LoadFileDiff { path, source } => {
            spawn_file_diff_load(
                ctx.state.repo_root.clone(),
                path,
                source,
                ctx.state.config.diff.context_lines,
                ctx.events.sender(),
            );
        }
        SideEffect::LoadPreviewFile(path) => {
            spawn_preview_load(
                path,
                ctx.state.config.preview.max_size_bytes,
                ctx.events.sender(),
            );
        }
        SideEffect::LaunchEditor { path, line } => {
            spawn_editor_launch(
                ctx.state.repo_root.clone(),
                ctx.state.config.editor.clone(),
                path,
                line,
                ctx.events.sender(),
            );
        }
        SideEffect::SaveWorkspace => {
            try_save_from_reduce_view(&ReduceView::from_app_state(ctx.state));
        }
        SideEffect::SpawnAgent(id) => {
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
        SideEffect::RestartAgent(id) => {
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
        SideEffect::WriteShell(data) => {
            let _ = ctx.pty.write_shell(&data);
        }
        SideEffect::WriteAgent(data) => {
            let id = ctx.state.agent_manager.active_id();
            let _ = ctx.pty.write_agent(id, &data);
        }
        SideEffect::ResizeShell { cols, rows } => {
            if ctx.pty.resize_shell(cols, rows) {
                ctx.state.shell.apply_resize(cols, rows);
            }
        }
        SideEffect::SpawnGitHubRefresh => {
            // Reducer refresh path emits `SpawnGitHubAuthCheck`; keep for parity with TUI.
        }
        SideEffect::SpawnGitHubAuthCheck => {
            spawn_github_auth_check(
                ctx.state.config.github.command.clone(),
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubIssueList => {
            spawn_github_issue_list_load(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubPrList => {
            spawn_github_pr_list_load(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubIssueDetail { number } => {
            spawn_github_issue_detail_load(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                number,
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubPrDetail { number } => {
            spawn_github_pr_detail_load(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                number,
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubIssueComment { number, body } => {
            spawn_github_issue_comment(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                number,
                body,
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubIssueCreateBranch { number } => {
            spawn_github_issue_create_branch(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                number,
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubRepoLabels => {
            spawn_github_repo_labels_load(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubIssueLabelApply { number, labels } => {
            spawn_github_issue_label_apply(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                number,
                labels,
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubOpenBrowser { target } => {
            spawn_github_open_browser(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                target,
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubPrCreate { request } => {
            spawn_github_pr_create(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                request,
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnGitHubPrMerge { number } => {
            spawn_github_pr_merge(
                ctx.state.repo_root.clone(),
                ctx.state.config.github.command.clone(),
                number,
                ctx.events.sender(),
            );
        }
        SideEffect::SpawnBranchList
        | SideEffect::SpawnBranchCheckout { .. }
        | SideEffect::CancelSearch
        | SideEffect::RunSearch { .. }
        | SideEffect::CopyToClipboard(_)
        | SideEffect::PasteFromClipboard
        | SideEffect::PersistUserTheme { .. } => {}
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
/// Returns `(should_quit, event_count)`.
pub fn process_pending_events(
    state: &mut AppState,
    events: &mut EventChannel,
    pty: &mut PtyRuntime,
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
        };
        if execute_gui_effects(&mut ctx, effects) {
            should_quit = true;
            break;
        }
    }

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

        let (quit, count) = process_pending_events(&mut state, &mut events, &mut pty);
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
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
            pty: &mut pty,
        };

        let quit = execute_gui_effects(&mut ctx, vec![SideEffect::SpawnGitRefresh]);
        assert!(!quit);
    }

    #[test]
    fn spawn_github_auth_check_effect_does_not_quit() {
        let mut state = test_state();
        let events = EventChannel::new();
        let mut pty = PtyRuntime::new();
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
            pty: &mut pty,
        };

        let quit = execute_gui_effects(&mut ctx, vec![SideEffect::SpawnGitHubAuthCheck]);
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

        let effects = kiwi_core::reducer::reduce(
            &mut ReduceView::from_app_state(&mut state),
            AppEvent::Command(AppCommand::GitHubRefresh),
        );
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::SpawnGitHubAuthCheck
        )));

        let quit = execute_gui_effects(
            &mut ServiceContext {
                state: &mut state,
                events: &events,
                pty: &mut pty,
            },
            effects,
        );
        assert!(!quit);
        assert!(state.github.loading);
    }
}
