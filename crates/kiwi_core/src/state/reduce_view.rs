//! Mutable view over reducer domain fields.
//!
//! Both [`super::AppState`] and the TUI `AppState` expose the same field names through
//! this struct so the reducer can run without depending on a specific frontend wrapper.

use std::path::PathBuf;

use crate::agent::AgentManager;
use crate::config::ResolvedConfig;
use crate::file_tree::FileTreeState;
use crate::navigation::NavigationState;
use crate::preview::PreviewState;
use crate::search::SearchState;
use crate::state::{
    AgentState, BranchState, CommandPaletteState, DiffState, GitHubState, GitState, LogsState,
    NotificationState, PluginsState, SettingsState, ShellState, StatusBarState, WorkspaceMeta,
};
use crate::theme::{TerminalCapabilities, ThemePalette};

use super::ViewportMetrics;

pub struct ReduceView<'a> {
    pub config: &'a mut ResolvedConfig,
    pub navigation: &'a mut NavigationState,
    pub theme: &'a mut ThemePalette,
    pub terminal_capabilities: &'a mut TerminalCapabilities,
    pub viewport: &'a mut ViewportMetrics,
    pub repo_root: &'a mut PathBuf,
    pub dirty: &'a mut bool,
    pub file_tree: &'a mut FileTreeState,
    pub preview: &'a mut PreviewState,
    pub search: &'a mut SearchState,
    pub git: &'a mut GitState,
    pub branches: &'a mut BranchState,
    pub diff: &'a mut DiffState,
    pub github: &'a mut GitHubState,
    pub agent_manager: &'a mut AgentManager,
    pub shell: &'a mut ShellState,
    pub palette: &'a mut CommandPaletteState,
    pub plugins: &'a mut PluginsState,
    pub logs: &'a mut LogsState,
    pub settings: &'a mut SettingsState,
    pub notifications: &'a mut NotificationState,
    pub status_bar: &'a mut StatusBarState,
    pub workspace_meta: &'a mut WorkspaceMeta,
}

impl<'a> ReduceView<'a> {
    #[must_use]
    pub fn from_app_state(state: &'a mut super::AppState) -> Self {
        Self {
            config: &mut state.config,
            navigation: &mut state.navigation,
            theme: &mut state.theme,
            terminal_capabilities: &mut state.terminal_capabilities,
            viewport: &mut state.viewport,
            repo_root: &mut state.repo_root,
            dirty: &mut state.dirty,
            file_tree: &mut state.file_tree,
            preview: &mut state.preview,
            search: &mut state.search,
            git: &mut state.git,
            branches: &mut state.branches,
            diff: &mut state.diff,
            github: &mut state.github,
            agent_manager: &mut state.agent_manager,
            shell: &mut state.shell,
            palette: &mut state.palette,
            plugins: &mut state.plugins,
            logs: &mut state.logs,
            settings: &mut state.settings,
            notifications: &mut state.notifications,
            status_bar: &mut state.status_bar,
            workspace_meta: &mut state.workspace_meta,
        }
    }

    pub fn mark_clean(&mut self) {
        *self.dirty = false;
    }

    #[must_use]
    pub fn active_agent(&self) -> &AgentState {
        self.agent_manager.active_pty()
    }

    pub fn active_agent_mut(&mut self) -> &mut AgentState {
        self.agent_manager.active_pty_mut()
    }

    pub fn set_dirty(&mut self) {
        *self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        *self.dirty
    }
}
