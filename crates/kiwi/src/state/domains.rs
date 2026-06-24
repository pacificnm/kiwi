#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileTreeState {
    pub selected_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreviewState {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchState {
    pub query: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitState {
    pub branch: Option<String>,
    pub selected_path: Option<String>,
    pub modified_files: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiffState {
    pub selected_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitHubState {
    pub selected_issue: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentState {
    pub running: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShellState {
    pub command: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandPaletteState {
    pub open: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatusBarState {
    pub repo_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceMeta {
    pub repo_root: String,
    pub is_git_repo: bool,
}
