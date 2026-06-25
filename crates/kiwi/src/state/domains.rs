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

use crate::shell::ScrollbackBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellState {
    pub command: String,
    pub shell_name: String,
    pub running: bool,
    pub child_pid: Option<u32>,
    pub cols: u16,
    pub rows: u16,
    pub spawn_error: Option<String>,
    pub scrollback: ScrollbackBuffer,
    pub viewport_offset: usize,
    pub follow_tail: bool,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            command: String::new(),
            shell_name: String::new(),
            running: false,
            child_pid: None,
            cols: 0,
            rows: 0,
            spawn_error: None,
            scrollback: ScrollbackBuffer::new(),
            viewport_offset: 0,
            follow_tail: true,
        }
    }
}

impl ShellState {
    pub fn apply_spawn(
        &mut self,
        command: &str,
        shell_name: &str,
        child_pid: Option<u32>,
        cols: u16,
        rows: u16,
    ) {
        self.command = command.to_string();
        self.shell_name = shell_name.to_string();
        self.running = true;
        self.child_pid = child_pid;
        self.cols = cols;
        self.rows = rows;
        self.spawn_error = None;
        self.scrollback.clear();
        self.viewport_offset = 0;
        self.follow_tail = true;
    }

    pub fn apply_spawn_error(&mut self, message: String) {
        self.running = false;
        self.child_pid = None;
        self.spawn_error = Some(message);
    }
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
