//! Background service wiring and side-effect execution for the GUI.

use kiwi_core::events::{AppEvent, EventChannel, SideEffect};
use kiwi_core::file_tree::spawn_directory_load;
use kiwi_core::git::spawn_git_refresh;
use kiwi_core::state::{AppState, ReduceView};
use kiwi_core::workspace::try_save_from_reduce_view;

/// Maximum events processed per frame to avoid stalling egui.
pub const MAX_EVENTS_PER_FRAME: usize = 64;

pub struct ServiceContext<'a> {
    pub state: &'a mut AppState,
    pub events: &'a EventChannel,
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
        SideEffect::Quit => return true,
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
        SideEffect::SaveWorkspace => {
            try_save_from_reduce_view(&ReduceView::from_app_state(ctx.state));
        }
        SideEffect::SpawnBranchList
        | SideEffect::SpawnBranchCheckout { .. }
        | SideEffect::SpawnGitHubRefresh
        | SideEffect::SpawnAgent(_)
        | SideEffect::RestartAgent(_)
        | SideEffect::WriteShell(_)
        | SideEffect::WriteAgent(_)
        | SideEffect::ResizeShell { .. }
        | SideEffect::LoadPreviewFile(_)
        | SideEffect::LoadFileDiff { .. }
        | SideEffect::CancelSearch
        | SideEffect::RunSearch { .. }
        | SideEffect::LaunchEditor { .. }
        | SideEffect::CopyToClipboard(_)
        | SideEffect::PasteFromClipboard
        | SideEffect::SpawnGitHubAuthCheck
        | SideEffect::SpawnGitHubIssueList
        | SideEffect::SpawnGitHubPrList
        | SideEffect::SpawnGitHubIssueDetail { .. }
        | SideEffect::SpawnGitHubPrDetail { .. }
        | SideEffect::SpawnGitHubIssueComment { .. }
        | SideEffect::SpawnGitHubIssueCreateBranch { .. }
        | SideEffect::SpawnGitHubRepoLabels
        | SideEffect::SpawnGitHubIssueLabelApply { .. }
        | SideEffect::SpawnGitHubOpenBrowser { .. }
        | SideEffect::SpawnGitHubPrCreate { .. }
        | SideEffect::SpawnGitHubPrMerge { .. }
        | SideEffect::PersistUserTheme { .. } => {}
    }
    false
}

/// Drain pending events, apply reducers, and execute resulting side effects.
///
/// Returns `(should_quit, event_count)`.
pub fn process_pending_events(state: &mut AppState, events: &mut EventChannel) -> (bool, usize) {
    let pending: Vec<AppEvent> = events
        .drain_coalesced()
        .into_iter()
        .take(MAX_EVENTS_PER_FRAME)
        .collect();
    let count = pending.len();
    let mut should_quit = false;

    for event in pending {
        let effects = kiwi_core::reducer::reduce(&mut ReduceView::from_app_state(state), event);
        let mut ctx = ServiceContext { state, events };
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

        let (quit, count) = process_pending_events(&mut state, &mut events);
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
        let mut ctx = ServiceContext {
            state: &mut state,
            events: &events,
        };

        let quit = execute_gui_effects(&mut ctx, vec![SideEffect::SpawnGitRefresh]);
        assert!(!quit);
    }
}
