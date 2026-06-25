use std::path::PathBuf;

use crate::agent::agent_launch_spec;
use crate::config::ResolvedConfig;
use crate::layout::LayoutState;
use crate::navigation::NavigationState;
use crate::shell::shell_launch_spec;
use crate::theme::ThemePalette;

use super::domains::{
    AgentState, CommandPaletteState, DiffState, FileTreeState, GitHubState, GitState, PreviewState,
    SearchState, ShellState, StatusBarState, WorkspaceMeta,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub config: ResolvedConfig,
    pub navigation: NavigationState,
    pub layout: LayoutState,
    pub theme: ThemePalette,
    pub repo_root: PathBuf,
    pub dirty: bool,
    pub file_tree: FileTreeState,
    pub preview: PreviewState,
    pub search: SearchState,
    pub git: GitState,
    pub diff: DiffState,
    pub github: GitHubState,
    pub agent: AgentState,
    pub shell: ShellState,
    pub palette: CommandPaletteState,
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
        layout: LayoutState,
    ) -> Self {
        let repo_name = repo_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("kiwi")
            .to_string();

        let shell_spec = shell_launch_spec(&config.shell);
        let agent_spec = agent_launch_spec(&config.agent);

        Self {
            config,
            navigation: NavigationState::default(),
            layout,
            theme,
            repo_root: repo_root.clone(),
            dirty: true,
            file_tree: FileTreeState::default(),
            preview: PreviewState::default(),
            search: SearchState::default(),
            git: GitState::default(),
            diff: DiffState::default(),
            github: GitHubState::default(),
            agent: AgentState {
                command: agent_spec.command,
                agent_name: agent_spec.agent_name,
                ..AgentState::default()
            },
            shell: ShellState {
                command: shell_spec.command,
                shell_name: shell_spec.shell_name,
                ..ShellState::default()
            },
            palette: CommandPaletteState::default(),
            status_bar: StatusBarState { repo_name },
            workspace_meta: WorkspaceMeta {
                repo_root: repo_root.display().to_string(),
                is_git_repo,
            },
        }
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}
