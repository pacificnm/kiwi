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

use crate::events::{AppCommand, AppEvent, SearchEffect, SideEffect};

pub(super) fn reduce_search_set_query(state: &mut ReduceView<'_>, query: String) -> Vec<SideEffect> {
    state.search.schedule_query(query);
    state.set_dirty();
    vec![SideEffect::Search(SearchEffect::Cancel)]
}

pub(super) fn reduce_search_append_char(state: &mut ReduceView<'_>, ch: char) -> Vec<SideEffect> {
    let mut query = state.search.query.clone();
    query.push(ch);
    reduce_search_set_query(state, query)
}

pub(super) fn reduce_search_backspace(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.search.query.is_empty() {
        return Vec::new();
    }

    let mut query = state.search.query.clone();
    query.pop();
    reduce_search_set_query(state, query)
}

pub(super) fn reduce_search_clear(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.search.clear_query();
    state.set_dirty();
    vec![SideEffect::Search(SearchEffect::Cancel)]
}

pub(super) fn reduce_search_set_mode(
    state: &mut ReduceView<'_>,
    mode: crate::search::SearchMode,
) -> Vec<SideEffect> {
    state.search.set_mode(mode);
    state.set_dirty();
    vec![SideEffect::Search(SearchEffect::Cancel)]
}

pub(super) fn reduce_search_execute(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let generation = state.search.begin_execute();
    state.set_dirty();

    if state.search.query.is_empty() {
        return vec![SideEffect::Search(SearchEffect::Cancel)];
    }

    vec![SideEffect::Search(SearchEffect::Run {
        mode: state.search.mode,
        query: state.search.query.clone(),
        generation,
    })]
}

pub(super) fn reduce_search_cancel(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.search.cancel();
    state.set_dirty();
    vec![SideEffect::Search(SearchEffect::Cancel)]
}

pub(super) fn search_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.search_rows
}

pub(super) fn reduce_search_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state
        .search
        .move_selection(delta, search_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_search_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    state
        .search
        .select_index(index, search_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_search_completed(
    state: &mut ReduceView<'_>,
    generation: u64,
    results: Vec<crate::search::SearchResult>,
    truncated: bool,
    error: Option<String>,
) -> Vec<SideEffect> {
    if generation != state.search.generation {
        return Vec::new();
    }

    if let Some(message) = error {
        state.search.apply_error(generation, message);
    } else {
        state.search.apply_results(generation, results, truncated);
    }
    state.set_dirty();
    Vec::new()
}
