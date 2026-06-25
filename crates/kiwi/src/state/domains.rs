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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentState {
    pub command: String,
    pub agent_name: String,
    pub spawned: bool,
    pub running: bool,
    pub child_pid: Option<u32>,
    pub cols: u16,
    pub rows: u16,
    pub spawn_error: Option<String>,
}

impl AgentState {
    pub fn apply_spawn(
        &mut self,
        command: &str,
        agent_name: &str,
        child_pid: Option<u32>,
        cols: u16,
        rows: u16,
    ) {
        self.command = command.to_string();
        self.agent_name = agent_name.to_string();
        self.spawned = true;
        self.running = true;
        self.child_pid = child_pid;
        self.cols = cols;
        self.rows = rows;
        self.spawn_error = None;
    }

    pub fn apply_spawn_error(&mut self, message: String) {
        self.spawned = true;
        self.running = false;
        self.child_pid = None;
        self.spawn_error = Some(message);
    }
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

    pub fn apply_resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
    }

    pub fn scroll_by(&mut self, delta: i32, page_size: u16) {
        let page = usize::from(page_size.max(1));
        let visible_height = page;
        let line_count = self.scrollback.line_count();
        let max_start = line_count.saturating_sub(visible_height);
        let current = if self.follow_tail {
            max_start
        } else {
            self.viewport_offset.min(max_start)
        };

        let scroll_lines = delta.signum() * i32::from(page_size.max(1));
        let new_offset = (current as i32 + scroll_lines).clamp(0, max_start as i32) as usize;

        if new_offset >= max_start {
            self.follow_tail = true;
            self.viewport_offset = 0;
        } else {
            self.follow_tail = false;
            self.viewport_offset = new_offset;
        }
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
