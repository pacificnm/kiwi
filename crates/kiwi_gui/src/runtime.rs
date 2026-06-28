//! Post-bootstrap runtime: shared state, event channel, and background services.

use kiwi_core::events::{AppEvent, EventChannel};
use kiwi_core::reducer::{
    agent_spawn_effects_if_needed, file_tree_startup_effects, workspace_restore_effects,
};
use kiwi_core::state::{AppState, ReduceView, ViewportMetrics};
use kiwi_core::theme::TerminalCapabilities;
use kiwi_core::watcher::RepoWatcher;
use kiwi_core::workspace::{try_load_workspace_file, GuiWorkspaceSnapshot};

use crate::bootstrap::GuiBootstrapContext;
use crate::pty::PtyRuntime;
use crate::services::{execute_gui_effects, process_pending_events, SearchRuntime, ServiceContext};

pub struct GuiRuntime {
    pub state: AppState,
    pub events: EventChannel,
    pub pty: PtyRuntime,
    search: SearchRuntime,
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
        let mut pty = PtyRuntime::new();

        let mut gui_snapshot = None;
        if state.config.workspace.persist {
            if let Some(file) = try_load_workspace_file(&repo_root) {
                file.tui
                    .apply_to_reduce_view(&mut ReduceView::from_app_state(&mut state));
                gui_snapshot = file.gui;
            }
        }

        let shell_settings = state.config.shell.clone();
        pty.spawn_shell_at_startup(
            &repo_root,
            &shell_settings,
            &mut state,
            events.sender(),
        );

        let repo_watcher = match RepoWatcher::spawn(
            repo_root.clone(),
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
            pty,
            search: SearchRuntime::default(),
            _repo_watcher: repo_watcher,
        };

        let file_tree_effects =
            file_tree_startup_effects(&mut ReduceView::from_app_state(&mut runtime.state));
        let workspace_effects =
            workspace_restore_effects(&mut ReduceView::from_app_state(&mut runtime.state));
        let agent_spawn_effects =
            agent_spawn_effects_if_needed(&mut ReduceView::from_app_state(&mut runtime.state));

        let mut ctx = runtime.service_context();
        execute_gui_effects(&mut ctx, file_tree_effects);
        execute_gui_effects(&mut ctx, workspace_effects);
        execute_gui_effects(&mut ctx, agent_spawn_effects);

        if runtime.state.workspace_meta.is_git_repo {
            let _ = runtime.dispatch(AppEvent::GitRefreshRequested);
        }

        // Discover available plugins from repo_root/plugins/ and cross-ref with registry.
        let plugins_src = repo_root.join("plugins");
        if plugins_src.is_dir() {
            let registry = kiwi_core::plugins::default_registry_path()
                .map(|p| kiwi_core::plugins::PluginRegistry::load(&p).0)
                .unwrap_or_default();
            runtime.state.plugins.available =
                kiwi_core::plugins::scan_available_plugins(&[&plugins_src], &registry);
        }

        (runtime, gui_snapshot)
    }

    fn service_context(&mut self) -> ServiceContext<'_> {
        ServiceContext {
            state: &mut self.state,
            events: &self.events,
            pty: &mut self.pty,
            search: &mut self.search,
            dock_snapshot: None,
        }
    }

    fn sync_search_debounce(&mut self) {
        self.search.sync_debounce(&self.state);
    }

    pub fn search_debounce_pending(&self) -> bool {
        self.search.debounce_pending()
    }

    /// Returns `true` if the debounce timer fired. The caller is responsible for
    /// dispatching `SearchExecute` when this returns `true`.
    pub fn poll_search_debounce(&mut self) -> bool {
        self.search.poll_debounce()
    }

    pub fn dispatch(&mut self, event: AppEvent) -> bool {
        let effects =
            kiwi_core::reducer::reduce(&mut ReduceView::from_app_state(&mut self.state), event);
        let mut ctx = self.service_context();
        let quit = execute_gui_effects(&mut ctx, effects);
        self.sync_search_debounce();
        quit
    }

    /// Dispatch an [`AppCommand`] via the reducer. Returns `true` when the app should quit.
    pub fn dispatch_command(&mut self, command: kiwi_core::events::AppCommand) -> bool {
        self.dispatch(AppEvent::Command(command))
    }

    fn poll_agent_exits(&mut self) {
        let exits = self.pty.poll_agent_exits();
        for (agent_id, code) in exits {
            self.dispatch(AppEvent::AgentExited { agent_id, code });
        }
    }

    /// Returns `(should_quit, event_count)`.
    ///
    /// `dock_snapshot` is forwarded to the service layer so that `SideEffect::SaveWorkspace`
    /// can persist the dock layout. Pass `None` when no live dock is available (e.g., tests).
    pub fn process_pending_events(
        &mut self,
        dock_snapshot: Option<GuiWorkspaceSnapshot>,
    ) -> (bool, usize) {
        self.poll_agent_exits();
        let (quit, count) = process_pending_events(
            &mut self.state,
            &mut self.events,
            &mut self.pty,
            &mut self.search,
            dock_snapshot,
        );
        self.sync_search_debounce();
        (quit, count)
    }

    /// Propagate measured viewport dimensions to running PTY sessions.
    pub fn sync_pty_resize_from_viewport(&mut self) {
        let shell_cols = self.state.viewport.shell_cols;
        let shell_rows = self.state.viewport.shell_rows;
        if self.state.shell.running
            && shell_cols > 0
            && shell_rows > 0
            && (self.state.shell.cols != shell_cols || self.state.shell.rows != shell_rows)
            && self.pty.resize_shell(shell_cols, shell_rows)
        {
            self.state.shell.apply_resize(shell_cols, shell_rows);
            self.state.shell.scrollback.set_cols(shell_cols);
        }

        let agent_id = self.state.agent_manager.active_id();
        let agent_cols = self.state.viewport.agent_cols;
        let agent_rows = self.state.viewport.agent_rows;
        let needs_agent_resize = self
            .state
            .agent_manager
            .pty(agent_id)
            .is_some_and(|pty| {
                pty.spawned
                    && pty.running
                    && agent_cols > 0
                    && agent_rows > 0
                    && (pty.cols != agent_cols || pty.rows != agent_rows)
            });

        if needs_agent_resize && self.pty.resize_agent(agent_id, agent_cols, agent_rows) {
            if let Some(pty) = self.state.agent_manager.pty_mut(agent_id) {
                pty.apply_resize(agent_cols, agent_rows);
                pty.scrollback.set_cols(agent_cols);
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.pty.shutdown();
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
    fn search_debounce_pipeline_finds_files_in_repo() {
        use std::thread;
        use std::time::Duration;

        use kiwi_core::events::AppCommand;

        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::Path::to_path_buf)
            .expect("workspace root");
        let mut config = ResolvedConfig::default();
        config.workspace.persist = false;
        let mut runtime = GuiRuntime::build(GuiBootstrapContext {
            repo_root: repo,
            is_git_repo: true,
            config,
            theme: load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
        })
        .0;

        runtime.dispatch_command(AppCommand::SearchSetQuery("Cargo.toml".to_string()));
        assert!(runtime.state.search.debounce_scheduled);
        assert!(runtime.search_debounce_pending());

        thread::sleep(Duration::from_millis(250));
        assert!(
            runtime.poll_search_debounce(),
            "debounce timer should fire"
        );
        runtime.dispatch_command(AppCommand::SearchExecute);
        assert!(runtime.state.search.running);

        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while std::time::Instant::now() < deadline {
            runtime.process_pending_events(None);
            if !runtime.state.search.running {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        assert!(
            !runtime.state.search.running,
            "search should finish: error={:?}",
            runtime.state.search.error
        );
        assert!(
            !runtime.state.search.results.is_empty(),
            "expected file hits for Cargo.toml, error={:?}",
            runtime.state.search.error
        );
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
