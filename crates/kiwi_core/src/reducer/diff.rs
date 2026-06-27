use std::path::PathBuf;
use std::time::SystemTime;

use super::git::sync_git_selection_for_path;

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

pub fn diff_select_file_effects(state: &mut ReduceView<'_>, path: String) -> Vec<SideEffect> {
    if state.diff.selected_path.as_deref() == Some(path.as_str()) {
        return Vec::new();
    }

    let source = state.diff.source;
    state.diff.begin_load(path.clone());
    sync_git_selection_for_path(state, &path);
    state.set_dirty();
    vec![SideEffect::LoadFileDiff { path, source }]
}

pub(super) fn reduce_diff_select_file(state: &mut ReduceView<'_>, path: String) -> Vec<SideEffect> {
    diff_select_file_effects(state, path)
}

pub(super) fn reduce_diff_move_file(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    diff_move_file_effects(state, delta)
}

pub fn diff_move_file_effects(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let paths = changed_file_paths(&state.git.file_entries, state.config.git.show_untracked);
    let Some(path) = adjacent_changed_file(&paths, state.diff.selected_path.as_deref(), delta)
    else {
        return Vec::new();
    };

    diff_select_file_effects(state, path)
}

pub(super) fn reduce_diff_loaded(
    state: &mut ReduceView<'_>,
    result: crate::diff::FileDiffLoadResult,
) -> Vec<SideEffect> {
    state.diff.apply_loaded(result);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_diff_toggle_source(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    diff_set_source_effects(state, state.diff.source.toggle())
}

pub fn diff_set_source_effects(
    state: &mut ReduceView<'_>,
    source: crate::diff::DiffSource,
) -> Vec<SideEffect> {
    if state.diff.source == source {
        return Vec::new();
    }

    let Some(path) = state.diff.selected_path.clone() else {
        state.diff.source = source;
        state.set_dirty();
        return Vec::new();
    };

    state.diff.source = source;
    state.diff.begin_source_reload();
    state.set_dirty();
    vec![SideEffect::LoadFileDiff { path, source }]
}

pub(super) fn reduce_diff_set_source(
    state: &mut ReduceView<'_>,
    source: crate::diff::DiffSource,
) -> Vec<SideEffect> {
    diff_set_source_effects(state, source)
}

pub(super) fn diff_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.preview_rows
}

pub(super) fn diff_text_width(state: &ReduceView<'_>) -> usize {
    let area_width = state.viewport.preview_cols;
    let gutter = diff_gutter_width(&state.diff.lines);
    area_width.saturating_sub(gutter).max(1)
}

pub(super) fn diff_gutter_width(lines: &[crate::diff::DiffLine]) -> usize {
    let old_width = max_lineno_width(lines.iter().filter_map(|line| line.old_lineno));
    let new_width = max_lineno_width(lines.iter().filter_map(|line| line.new_lineno));
    if old_width == 0 && new_width == 0 {
        return 0;
    }
    old_width + 1 + new_width + 2
}

pub(super) fn max_lineno_width(values: impl Iterator<Item = u32>) -> usize {
    values
        .map(|value| value.checked_ilog10().unwrap_or(0) as usize + 1)
        .max()
        .unwrap_or(1)
}

pub(super) fn reduce_diff_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.diff.scroll(delta, diff_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_diff_page_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.diff.page_scroll(delta, diff_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_diff_horizontal_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 || state.config.diff.word_wrap {
        return Vec::new();
    }

    state.diff.scroll_horizontal(delta, diff_text_width(state));
    state.set_dirty();
    Vec::new()
}
