use std::path::PathBuf;

use kiwi_core::state::ViewportMetrics;
use kiwi_core::theme::{TerminalCapabilities, ThemePalette as CoreThemePalette};

use crate::agent::{agent_launch_spec, AgentManager};
use crate::config::ResolvedConfig;
use crate::file_tree::FileTreeState;
use crate::layout::{viewport_metrics_from_layout, LayoutState};
use crate::navigation::NavigationState;
use crate::preview::PreviewState;
use crate::search::SearchState;
use crate::selection::TextSelection;
use crate::shell::shell_launch_spec;
use crate::theme::capabilities::detect_terminal_capabilities;
use crate::theme::{load_core_theme_with_capabilities, ThemePalette};

pub use kiwi_core::state::{
    AgentState, BranchState, CommandPaletteState, DiffState, GitHubState, GitState, LogsState,
    NotificationState, PluginsState, SettingsState, ShellState, StatusBarState, WorkspaceMeta,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub config: ResolvedConfig,
    pub navigation: NavigationState,
    pub layout: LayoutState,
    /// Ratatui palette; kept in sync with [`Self::core_theme`].
    pub theme: ThemePalette,
    pub core_theme: CoreThemePalette,
    pub terminal_capabilities: TerminalCapabilities,
    pub viewport: ViewportMetrics,
    pub repo_root: PathBuf,
    pub dirty: bool,
    pub file_tree: FileTreeState,
    pub preview: PreviewState,
    pub search: SearchState,
    pub git: GitState,
    pub branches: BranchState,
    pub diff: DiffState,
    pub github: GitHubState,
    pub agent_manager: AgentManager,
    pub shell: ShellState,
    pub palette: CommandPaletteState,
    pub plugins: PluginsState,
    pub logs: LogsState,
    pub settings: SettingsState,
    pub notifications: NotificationState,
    pub status_bar: StatusBarState,
    pub workspace_meta: WorkspaceMeta,
    pub text_selection: TextSelection,
    pub pty_cursor_blink_on: bool,
}

impl AppState {
    #[must_use]
    pub fn from_startup(
        repo_root: PathBuf,
        is_git_repo: bool,
        config: ResolvedConfig,
        theme: ThemePalette,
        layout: LayoutState,
    ) -> Self {
        let root_name = repo_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("kiwi")
            .to_string();

        let shell_spec = shell_launch_spec(&config.shell);
        let agent_spec = agent_launch_spec(&config.agent);
        let mut file_tree = FileTreeState::at_root(repo_root.clone());
        file_tree.ensure_selection();

        let terminal_capabilities = detect_terminal_capabilities();
        let core_theme =
            load_core_theme_with_capabilities(&config.theme, terminal_capabilities).expect("theme");
        let viewport = viewport_metrics_from_layout(&layout.rects);

        Self {
            config,
            navigation: NavigationState::default(),
            layout,
            theme,
            core_theme,
            terminal_capabilities,
            viewport,
            repo_root: repo_root.clone(),
            dirty: true,
            file_tree,
            preview: PreviewState::default(),
            search: SearchState::default(),
            git: GitState::default(),
            branches: BranchState::default(),
            diff: DiffState::default(),
            github: GitHubState::default(),
            agent_manager: AgentManager::with_initial_agent(AgentState {
                command: agent_spec.command,
                agent_name: agent_spec.agent_name,
                ..AgentState::default()
            }),
            shell: ShellState {
                command: shell_spec.command,
                shell_name: shell_spec.shell_name,
                ..ShellState::default()
            },
            palette: CommandPaletteState::default(),
            plugins: PluginsState::default(),
            logs: LogsState::default(),
            settings: SettingsState::default(),
            notifications: NotificationState::default(),
            status_bar: StatusBarState { root_name },
            workspace_meta: WorkspaceMeta {
                repo_root: repo_root.display().to_string(),
                is_git_repo,
                ..WorkspaceMeta::default()
            },
            text_selection: TextSelection::default(),
            pty_cursor_blink_on: true,
        }
    }

    pub fn sync_viewport_from_layout(&mut self) {
        self.viewport = viewport_metrics_from_layout(&self.layout.rects);
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    #[must_use]
    pub fn active_agent(&self) -> &AgentState {
        self.agent_manager.active_pty()
    }

    #[cfg(test)]
    pub fn active_agent_mut(&mut self) -> &mut AgentState {
        self.agent_manager.active_pty_mut()
    }

    pub fn reduce_view(&mut self) -> kiwi_core::state::ReduceView<'_> {
        kiwi_core::state::ReduceView {
            config: &mut self.config,
            navigation: &mut self.navigation,
            theme: &mut self.core_theme,
            terminal_capabilities: &mut self.terminal_capabilities,
            viewport: &mut self.viewport,
            repo_root: &mut self.repo_root,
            dirty: &mut self.dirty,
            file_tree: &mut self.file_tree,
            preview: &mut self.preview,
            search: &mut self.search,
            git: &mut self.git,
            branches: &mut self.branches,
            diff: &mut self.diff,
            github: &mut self.github,
            agent_manager: &mut self.agent_manager,
            shell: &mut self.shell,
            palette: &mut self.palette,
            plugins: &mut self.plugins,
            logs: &mut self.logs,
            settings: &mut self.settings,
            notifications: &mut self.notifications,
            status_bar: &mut self.status_bar,
            workspace_meta: &mut self.workspace_meta,
        }
    }
}
