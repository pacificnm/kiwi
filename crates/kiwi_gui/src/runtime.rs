//! Post-bootstrap runtime: shared state, event channel, and background services.

use kiwi_core::events::{AppEvent, EventChannel};
use kiwi_core::reducer::file_tree_startup_effects;
use kiwi_core::state::{AppState, ReduceView, ViewportMetrics};
use kiwi_core::theme::TerminalCapabilities;
use kiwi_core::watcher::RepoWatcher;
use kiwi_core::workspace::{try_load_workspace_file, GuiWorkspaceSnapshot};

use crate::bootstrap::GuiBootstrapContext;
use crate::services::{execute_gui_effects, process_pending_events, ServiceContext};

pub struct GuiRuntime {
    pub state: AppState,
    pub events: EventChannel,
    _repo_watcher: Option<RepoWatcher>,
}

impl GuiRuntime {
    pub fn build(context: GuiBootstrapContext) -> (Self, Option<GuiWorkspaceSnapshot>) {
        let GuiBootstrapContext {
            repo_root,
            is_git_repo,
            config,
            theme,
        } = context;

        let mut state = AppState::from_startup(
            repo_root.clone(),
            is_git_repo,
            config.clone(),
            theme,
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        );
        let events = EventChannel::new();

        let mut gui_snapshot = None;
        if state.config.workspace.persist {
            if let Some(file) = try_load_workspace_file(&repo_root) {
                file.tui
                    .apply_to_reduce_view(&mut ReduceView::from_app_state(&mut state));
                gui_snapshot = file.gui;
            }
        }

        let repo_watcher = match RepoWatcher::spawn(
            repo_root,
            state.config.watcher.debounce_ms,
            events.sender(),
        ) {
            Ok(watcher) => Some(watcher),
            Err(err) => {
                eprintln!("file watcher disabled: {err}");
                state.logs.push_info(format!(
                    "File watcher disabled: {err}. Git status refreshes on startup only."
                ));
                None
            }
        };

        let mut runtime = Self {
            state,
            events,
            _repo_watcher: repo_watcher,
        };

        let file_tree_effects =
            file_tree_startup_effects(&mut ReduceView::from_app_state(&mut runtime.state));
        let mut ctx = ServiceContext {
            state: &mut runtime.state,
            events: &runtime.events,
        };
        execute_gui_effects(&mut ctx, file_tree_effects);

        if runtime.state.workspace_meta.is_git_repo {
            runtime.dispatch(AppEvent::GitRefreshRequested);
        }

        (runtime, gui_snapshot)
    }

    pub fn dispatch(&mut self, event: AppEvent) {
        let effects =
            kiwi_core::reducer::reduce(&mut ReduceView::from_app_state(&mut self.state), event);
        let mut ctx = ServiceContext {
            state: &mut self.state,
            events: &self.events,
        };
        execute_gui_effects(&mut ctx, effects);
    }

    /// Returns `(should_quit, event_count)`.
    pub fn process_pending_events(&mut self) -> (bool, usize) {
        process_pending_events(&mut self.state, &mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::events::AppEvent;
    use kiwi_core::status_bar::compute_status_bar;
    use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use crate::bootstrap::GuiBootstrapContext;

    use super::*;

    #[test]
    fn build_initializes_status_bar_root_name_from_repo() {
        let runtime = GuiRuntime::build(GuiBootstrapContext {
            repo_root: PathBuf::from("/tmp/kiwi"),
            is_git_repo: false,
            config: ResolvedConfig::default(),
            theme: load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
        })
        .0;

        let snapshot = compute_status_bar(&runtime.state);
        assert_eq!(snapshot.root_name, "kiwi");
        assert_eq!(snapshot.branch, "no git");
    }

    #[test]
    fn dispatch_git_status_updates_branch() {
        let mut runtime = GuiRuntime::build(GuiBootstrapContext {
            repo_root: PathBuf::from("/tmp/kiwi"),
            is_git_repo: true,
            config: ResolvedConfig::default(),
            theme: load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
        })
        .0;

        runtime.dispatch(AppEvent::GitStatusUpdated {
            branch: Some("feature/test".to_string()),
            remote_repo: None,
            ahead: 0,
            behind: 0,
            file_entries: Vec::new(),
            error: None,
        });

        let snapshot = compute_status_bar(&runtime.state);
        assert_eq!(snapshot.branch, "feature/test");
    }
}
