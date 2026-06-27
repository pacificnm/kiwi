use std::path::PathBuf;
use std::time::SystemTime;

use crate::agent::infer_status_from_scrollback;
use crate::commands::{execute_command, history_input_for_id, refresh_matches};
use crate::config::{project_has_theme_override, ThemeSettings};
use crate::file_tree::ExpandAction;
use crate::git::{
    adjacent_changed_file, branch_move_selection, branch_select_row, branch_selected_name,
    build_panel_rows, changed_file_paths, clamp_git_scroll, ensure_branch_selection,
    ensure_git_selection, git_move_selection, git_select_row, patch_git_file_entries,
    row_for_path, BranchEntry, GitFileEntry,
};
use crate::github::{
    advance_pr_create_prompt, apply_label_picker_load, ensure_issue_selection, ensure_pr_selection,
    format_issue_agent_prompt, format_pr_agent_prompt, issue_body_excerpt_from_detail, issue_move_selection, issue_select_row,
    missing_browser_target_message, page_scroll_issue_detail, pr_move_selection, pr_select_row,
    pull_request_is_mergeable, resolve_browser_target, scroll_issue_detail, selected_pull_request,
    GhContextMenuAction, GhContextMenuState, GhContextTarget, GitHubLeftPane,
    IssueDetailLoadResult, IssueListLoadResult, LabelPickerState, PrCreatePromptAdvance,
    PrDetailLoadResult, PrListLoadResult, ISSUE_LIST_CACHE_SECS, PR_LIST_CACHE_SECS,
};
use crate::navigation::{FocusTarget, LeftNavTab, MainTab, NavCommand};
use crate::settings::{ensure_settings_selection, settings_move_selection, settings_select_row};
use crate::state::{PalettePrompt, ReduceView};
use crate::theme::load_theme_with_capabilities;

use crate::events::{AppCommand, AppEvent, SideEffect};

pub(super) fn reduce_shell_output(state: &mut ReduceView<'_>, data: Vec<u8>) -> Vec<SideEffect> {
    state.shell.scrollback.set_cols(state.shell.cols);
    state.shell.scrollback.append_bytes(&data);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_shell_exited(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.shell.running = false;
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_shell_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.shell_rows;
    state.shell.scroll_by(delta, page_size);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_shell_scroll_lines(state: &mut ReduceView<'_>, lines: i32) -> Vec<SideEffect> {
    if lines == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.shell_rows;
    state.shell.scroll_by_lines(lines, page_size);
    state.set_dirty();
    Vec::new()
}
