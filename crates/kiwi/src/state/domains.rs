use std::collections::HashMap;
use std::time::SystemTime;

use crate::agent::AgentStatus;
use crate::diff::{DiffLine, DiffSource, FileDiffLoadResult};
use crate::git::{BranchEntry, GitFileEntry};
use crate::github::{Issue, PullRequest};
use crate::layout::FocusTarget;
use crate::shell::ScrollbackBuffer;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SettingsState {
    pub selected_index: usize,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitState {
    pub branch: Option<String>,
    pub remote_repo: Option<String>,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BranchState {
    pub entries: Vec<BranchEntry>,
    pub selected_index: Option<usize>,
    pub scroll_offset: usize,
    pub loading: bool,
    pub checkout_loading: bool,
    pub error: Option<String>,
    pub checkout_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffState {
    pub selected_path: Option<String>,
    pub source: DiffSource,
    pub lines: Vec<DiffLine>,
    pub scroll_offset: usize,
    pub horizontal_scroll_offset: usize,
    pub scroll_by_path: HashMap<String, (usize, usize)>,
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
            horizontal_scroll_offset: 0,
            scroll_by_path: HashMap::new(),
            loading: false,
            is_binary: false,
            error: None,
        }
    }
}

impl DiffState {
    fn save_scroll_for_current(&mut self) {
        if let Some(path) = self.selected_path.clone() {
            self.scroll_by_path
                .insert(path, (self.scroll_offset, self.horizontal_scroll_offset));
        }
    }

    pub fn begin_load(&mut self, path: String) {
        self.save_scroll_for_current();
        self.selected_path = Some(path.clone());
        self.loading = true;
        self.error = None;
        self.is_binary = false;
        self.lines.clear();
        if let Some((scroll_offset, horizontal_scroll_offset)) =
            self.scroll_by_path.get(&path).copied()
        {
            self.scroll_offset = scroll_offset;
            self.horizontal_scroll_offset = horizontal_scroll_offset;
        } else {
            self.scroll_offset = 0;
            self.horizontal_scroll_offset = 0;
        }
    }

    pub fn begin_source_reload(&mut self) {
        self.loading = true;
        self.error = None;
        self.is_binary = false;
    }

    pub fn apply_loaded(&mut self, result: FileDiffLoadResult) {
        if self.selected_path.as_deref() != Some(result.path.as_str()) {
            return;
        }

        self.loading = false;
        self.is_binary = result.is_binary;
        self.error = result.error;
        self.lines = result.lines;
        let max_offset = self.lines.len().saturating_sub(1);
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    pub fn scroll(&mut self, delta: i32, viewport_rows: usize) {
        if viewport_rows == 0 {
            return;
        }

        let max_offset = self.lines.len().saturating_sub(viewport_rows);
        let current = self.scroll_offset as i32;
        let next = (current + delta).clamp(0, max_offset as i32);
        self.scroll_offset = usize::try_from(next).unwrap_or(0);
    }

    pub fn page_scroll(&mut self, delta: i32, viewport_rows: usize) {
        if viewport_rows == 0 {
            return;
        }

        let page = i32::try_from(viewport_rows.saturating_sub(1).max(1)).unwrap_or(1);
        self.scroll(delta * page, viewport_rows);
    }

    pub fn scroll_horizontal(&mut self, delta: i32, visible_text_width: usize) {
        let max_width = self
            .lines
            .iter()
            .map(|line| line.content.chars().count())
            .max()
            .unwrap_or(0);
        if visible_text_width >= max_width {
            self.horizontal_scroll_offset = 0;
            return;
        }

        let max_offset = max_width.saturating_sub(visible_text_width);
        let current = self.horizontal_scroll_offset as i32;
        let next = (current + delta).clamp(0, max_offset as i32);
        self.horizontal_scroll_offset = usize::try_from(next).unwrap_or(0);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitHubState {
    pub selected_issue: Option<u64>,
    pub selected_pr: Option<u64>,
    pub auth_checked: bool,
    pub auth_ok: bool,
    pub loading: bool,
    pub error_kind: Option<crate::github::GitHubAuthErrorKind>,
    pub error: Option<String>,
    pub issues: Vec<Issue>,
    pub issues_loading: bool,
    pub issues_error: Option<String>,
    pub issues_scroll_offset: usize,
    pub issues_loaded_at: Option<SystemTime>,
    pub prs: Vec<PullRequest>,
    pub prs_loading: bool,
    pub prs_error: Option<String>,
    pub prs_scroll_offset: usize,
    pub prs_loaded_at: Option<SystemTime>,
    pub issue_detail_number: Option<u64>,
    pub issue_detail: Option<crate::github::IssueDetail>,
    pub issue_detail_loading: bool,
    pub issue_detail_error: Option<String>,
    pub issue_detail_scroll_offset: usize,
    pub pr_detail_number: Option<u64>,
    pub pr_detail: Option<crate::github::PrDetail>,
    pub pr_detail_loading: bool,
    pub pr_detail_error: Option<String>,
    pub pr_detail_scroll_offset: usize,
    pub left_pane: crate::github::GitHubLeftPane,
    pub label_picker: Option<crate::github::LabelPickerState>,
    pub context_menu: Option<crate::github::GhContextMenuState>,
    pub issue_action_message: Option<String>,
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
        let scroll_lines = delta.signum() * i32::from(page_size.max(1));
        self.scroll_by_lines(scroll_lines, page_size);
    }

    pub fn scroll_by_lines(&mut self, line_delta: i32, visible_height: u16) {
        if line_delta == 0 {
            return;
        }

        let visible = usize::from(visible_height.max(1));
        let line_count = self.scrollback.line_count();
        let max_start = line_count.saturating_sub(visible);
        let current = if self.follow_tail {
            max_start
        } else {
            self.viewport_offset.min(max_start)
        };

        let new_offset = (current as i32 + line_delta).clamp(0, max_start as i32) as usize;

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
        let scroll_lines = delta.signum() * i32::from(page_size.max(1));
        self.scroll_by_lines(scroll_lines, page_size);
    }

    pub fn scroll_by_lines(&mut self, line_delta: i32, visible_height: u16) {
        if line_delta == 0 {
            return;
        }

        let visible = usize::from(visible_height.max(1));
        let line_count = self.scrollback.line_count();
        let max_start = line_count.saturating_sub(visible);
        let current = if self.follow_tail {
            max_start
        } else {
            self.viewport_offset.min(max_start)
        };

        let new_offset = (current as i32 + line_delta).clamp(0, max_start as i32) as usize;

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
pub enum PalettePrompt {
    GitHubIssueComment { number: u32 },
    GitHubPrCreate(GitHubPrCreatePrompt),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitHubPrCreateStep {
    Title,
    Body,
    Base,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubPrCreatePrompt {
    pub step: GitHubPrCreateStep,
    pub title: String,
    pub body: String,
}

impl Default for GitHubPrCreatePrompt {
    fn default() -> Self {
        Self {
            step: GitHubPrCreateStep::Title,
            title: String::new(),
            body: String::new(),
        }
    }
}

impl PalettePrompt {
    #[must_use]
    pub fn title(&self) -> String {
        match self {
            Self::GitHubIssueComment { number } => format!("Comment on issue #{number}"),
            Self::GitHubPrCreate(prompt) => match prompt.step {
                GitHubPrCreateStep::Title => "Create pull request — title".to_string(),
                GitHubPrCreateStep::Body => "Create pull request — body".to_string(),
                GitHubPrCreateStep::Base => "Create pull request — base branch".to_string(),
            },
        }
    }

    #[must_use]
    pub fn hint(&self) -> &'static str {
        match self {
            Self::GitHubIssueComment { .. } => "Enter to post · Esc to cancel",
            Self::GitHubPrCreate(prompt) => match prompt.step {
                GitHubPrCreateStep::Title => "Enter to continue · Esc to cancel",
                GitHubPrCreateStep::Body => "Enter to continue · Esc to cancel",
                GitHubPrCreateStep::Base => "Enter to create · leave empty for default base",
            },
        }
    }
}

use crate::workspace::MAX_PALETTE_HISTORY_ENTRIES;

const MAX_PALETTE_HISTORY: usize = MAX_PALETTE_HISTORY_ENTRIES;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PluginsState {
    pub commands: Vec<PluginPaletteCommand>,
}

#[derive(Debug, Clone)]
pub struct PluginPaletteCommand {
    pub id: String,
    pub title: String,
    pub plugin_name: String,
    pub callback: extern "C" fn() -> kiwi_plugin_api::PluginResult,
    pub enabled: bool,
}

impl PartialEq for PluginPaletteCommand {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.title == other.title
            && self.plugin_name == other.plugin_name
            && self.enabled == other.enabled
    }
}

impl Eq for PluginPaletteCommand {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteState {
    pub open: bool,
    pub input: String,
    pub prompt: Option<PalettePrompt>,
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
            prompt: None,
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
        self.prompt = None;
        self.selected = 0;
        self.history_cursor = None;
        if focus != FocusTarget::CommandPalette {
            self.focus_before_open = focus;
        }
    }

    pub fn begin_prompt(&mut self, prompt: PalettePrompt, focus: FocusTarget) {
        self.open = true;
        self.prompt = Some(prompt);
        self.input.clear();
        self.matches.clear();
        self.selected = 0;
        self.history_cursor = None;
        if focus != FocusTarget::CommandPalette {
            self.focus_before_open = focus;
        }
    }

    pub fn close(&mut self, focus: &mut FocusTarget) {
        self.open = false;
        self.input.clear();
        self.prompt = None;
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
    pub root_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceMeta {
    pub repo_root: String,
    pub is_git_repo: bool,
    /// Expanded directories from a saved snapshot that are not yet present in the tree.
    pub pending_expanded_paths: Vec<std::path::PathBuf>,
    /// Selected file path from a saved snapshot waiting for tree nodes to load.
    pub pending_selected_path: Option<std::path::PathBuf>,
}
