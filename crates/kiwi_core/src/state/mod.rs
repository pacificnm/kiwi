mod domains;
mod reduce_view;
mod viewport;

pub use domains::{
    AgentState, BranchState, CommandPaletteState, DiffState, GitHubPrCreatePrompt,
    GitHubPrCreateStep, GitHubState, GitState, LogEntry, LogLevel, LogsState, ModalState,
    NotificationState, PalettePrompt, PluginEntry, PluginPaletteCommand, PluginStatus,
    PluginsState, SettingsState,
    ShellState, StatusBarState, ToastState, WorkspaceMeta, MAX_PALETTE_HISTORY_ENTRIES,
};
pub use reduce_view::ReduceView;
pub use viewport::ViewportMetrics;

use std::path::PathBuf;

use crate::agent::{agent_launch_spec, AgentManager};
use crate::config::ResolvedConfig;
use crate::file_tree::FileTreeState;
use crate::navigation::NavigationState;
use crate::preview::PreviewState;
use crate::search::SearchState;
use crate::shell::shell_launch_spec;
use crate::theme::{TerminalCapabilities, ThemePalette};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub config: ResolvedConfig,
    pub navigation: NavigationState,
    pub theme: ThemePalette,
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
}

impl AppState {
    #[must_use]
    pub fn from_startup(
        repo_root: PathBuf,
        is_git_repo: bool,
        config: ResolvedConfig,
        theme: ThemePalette,
        terminal_capabilities: TerminalCapabilities,
        viewport: ViewportMetrics,
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

        Self {
            config,
            navigation: NavigationState::default(),
            theme,
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
        }
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    #[must_use]
    pub fn active_agent(&self) -> &AgentState {
        self.agent_manager.active_pty()
    }

    pub fn active_agent_mut(&mut self) -> &mut AgentState {
        self.agent_manager.active_pty_mut()
    }
}
