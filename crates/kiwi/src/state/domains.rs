use crate::agent::AgentStatus;
use crate::diff::{DiffLine, DiffSource, FileDiffLoadResult};
use crate::git::GitFileEntry;
use crate::layout::FocusTarget;
use crate::shell::ScrollbackBuffer;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitState {
    pub branch: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub selected_path: Option<String>,
    pub file_entries: Vec<GitFileEntry>,
    pub scroll_offset: usize,
    pub loading: bool,
    pub error: Option<String>,
}

impl GitState {
    #[must_use]
    pub fn changed_count(&self) -> usize {
        self.file_entries.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffState {
    pub selected_path: Option<String>,
    pub source: DiffSource,
    pub lines: Vec<DiffLine>,
    pub scroll_offset: usize,
    pub loading: bool,
    pub is_binary: bool,
    pub error: Option<String>,
}

impl Default for DiffState {
    fn default() -> Self {
        Self {
            selected_path: None,
            source: DiffSource::Unstaged,
            lines: Vec::new(),
            scroll_offset: 0,
            loading: false,
            is_binary: false,
            error: None,
        }
    }
}

impl DiffState {
    pub fn begin_load(&mut self, path: String) {
        self.selected_path = Some(path);
        self.loading = true;
        self.error = None;
        self.is_binary = false;
        self.lines.clear();
        self.scroll_offset = 0;
    }

    pub fn apply_loaded(&mut self, result: FileDiffLoadResult) {
        if self.selected_path.as_deref() != Some(result.path.as_str()) {
            return;
        }

        self.loading = false;
        self.is_binary = result.is_binary;
        self.error = result.error;
        self.lines = result.lines;
        if self.scroll_offset >= self.lines.len() && !self.lines.is_empty() {
            self.scroll_offset = self.lines.len() - 1;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitHubState {
    pub selected_issue: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentState {
    pub command: String,
    pub agent_name: String,
    pub spawned: bool,
    pub running: bool,
    pub child_pid: Option<u32>,
    pub cols: u16,
    pub rows: u16,
    pub spawn_error: Option<String>,
    pub scrollback: ScrollbackBuffer,
    pub viewport_offset: usize,
    pub follow_tail: bool,
    pub status: AgentStatus,
    pub exit_code: Option<i32>,
    pub restart_hint: Option<String>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            command: String::new(),
            agent_name: String::new(),
            spawned: false,
            running: false,
            child_pid: None,
            cols: 0,
            rows: 0,
            spawn_error: None,
            scrollback: ScrollbackBuffer::new(),
            viewport_offset: 0,
            follow_tail: true,
            status: AgentStatus::Idle,
            exit_code: None,
            restart_hint: None,
        }
    }
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
        self.exit_code = None;
        self.restart_hint = None;
        self.scrollback.clear();
        self.scrollback.set_cols(cols);
        self.viewport_offset = 0;
        self.follow_tail = true;
        self.status = AgentStatus::Executing;
    }

    pub fn apply_spawn_error(&mut self, message: String) {
        self.spawned = true;
        self.running = false;
        self.child_pid = None;
        self.spawn_error = Some(message);
        self.status = AgentStatus::Error;
        self.restart_hint = Some("Agent failed to start. Ctrl+Shift+R to retry.".to_string());
    }

    pub fn apply_exit(&mut self, code: i32) {
        self.running = false;
        self.child_pid = None;
        self.exit_code = Some(code);
        self.status = AgentStatus::from_exit_code(code);
        self.restart_hint = Some(format!(
            "Agent exited (code {code}). Ctrl+Shift+R to restart."
        ));
    }

    pub fn prepare_restart(&mut self) {
        self.spawned = false;
        self.running = false;
        self.child_pid = None;
        self.spawn_error = None;
        self.exit_code = None;
        self.restart_hint = None;
        self.status = AgentStatus::Idle;
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
        self.scrollback.set_cols(cols);
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

use crate::workspace::MAX_PALETTE_HISTORY_ENTRIES;

const MAX_PALETTE_HISTORY: usize = MAX_PALETTE_HISTORY_ENTRIES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteState {
    pub open: bool,
    pub input: String,
    pub matches: Vec<usize>,
    pub selected: usize,
    pub history: Vec<String>,
    pub focus_before_open: FocusTarget,
    pub history_cursor: Option<usize>,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
            open: false,
            input: String::new(),
            matches: Vec::new(),
            selected: 0,
            history: Vec::new(),
            focus_before_open: FocusTarget::Main,
            history_cursor: None,
        }
    }
}

impl CommandPaletteState {
    pub fn open_with_focus(&mut self, focus: FocusTarget) {
        self.open = true;
        self.input.clear();
        self.selected = 0;
        self.history_cursor = None;
        if focus != FocusTarget::CommandPalette {
            self.focus_before_open = focus;
        }
    }

    pub fn close(&mut self, focus: &mut FocusTarget) {
        self.open = false;
        self.input.clear();
        self.matches.clear();
        self.selected = 0;
        self.history_cursor = None;
        *focus = self.focus_before_open;
    }

    pub fn record_history(&mut self, command_id: &str) {
        if let Some(position) = self.history.iter().position(|id| id == command_id) {
            self.history.remove(position);
        }
        self.history.push(command_id.to_string());
        if self.history.len() > MAX_PALETTE_HISTORY {
            let overflow = self.history.len() - MAX_PALETTE_HISTORY;
            self.history.drain(0..overflow);
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.matches.is_empty() {
            self.selected = 0;
            return;
        }

        let len = self.matches.len();
        let current = self.selected as isize;
        let next = (current + delta).rem_euclid(len as isize);
        self.selected = usize::try_from(next).unwrap_or(0);
    }

    pub fn history_up(&mut self) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }

        let cursor = match self.history_cursor {
            None => self.history.len() - 1,
            Some(0) => return None,
            Some(value) => value - 1,
        };
        self.history_cursor = Some(cursor);
        Some(self.history[cursor].clone())
    }

    pub fn history_down(&mut self) -> Option<Option<String>> {
        let cursor = self.history_cursor?;

        if cursor + 1 >= self.history.len() {
            self.history_cursor = None;
            return Some(None);
        }

        self.history_cursor = Some(cursor + 1);
        Some(Some(self.history[cursor + 1].clone()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogsState {
    pub entries: Vec<LogEntry>,
}

impl LogsState {
    const MAX_ENTRIES: usize = 500;

    pub fn push_info(&mut self, message: impl Into<String>) {
        self.push(LogLevel::Info, message);
    }

    pub fn push_error(&mut self, message: impl Into<String>) {
        self.push(LogLevel::Error, message);
    }

    fn push(&mut self, level: LogLevel, message: impl Into<String>) {
        self.entries.push(LogEntry {
            level,
            message: message.into(),
        });
        if self.entries.len() > Self::MAX_ENTRIES {
            let overflow = self.entries.len() - Self::MAX_ENTRIES;
            self.entries.drain(0..overflow);
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToastState {
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalState {
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationState {
    pub toast: ToastState,
    pub modal: Option<ModalState>,
}

impl NotificationState {
    pub fn show_toast(&mut self, message: impl Into<String>) {
        self.toast.message = Some(message.into());
    }

    pub fn show_modal(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.modal = Some(ModalState {
            title: title.into(),
            message: message.into(),
        });
    }

    pub fn dismiss_modal(&mut self) {
        self.modal = None;
    }

    #[allow(dead_code)]
    pub fn clear_toast(&mut self) {
        self.toast.message = None;
    }
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
