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

use crate::events::{AppCommand, AppEvent, FsEffect, SideEffect};

use super::diff::diff_viewport_rows;
use super::git::{branch_list_access_effects, reduce_git_refresh_requested, sync_git_statuses_to_file_tree};
use super::github::{github_first_access_effects, github_issue_list_effects, github_pr_list_access_effects};

pub fn file_tree_startup_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let root = state.file_tree.root.clone();
    reduce_file_tree_expand(state, root)
}

pub fn workspace_expand_pending_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let pending = std::mem::take(&mut state.workspace_meta.pending_expanded_paths);
    let mut remaining = Vec::new();
    let mut effects = Vec::new();

    for path in pending {
        if !state.file_tree.nodes.contains_key(&path) {
            remaining.push(path);
            continue;
        }

        match state.file_tree.expand(&path) {
            Ok(ExpandAction::NeedsLoad) => {
                effects.push(SideEffect::Fs(FsEffect::LoadDirectoryChildren(path)));
            }
            Ok(ExpandAction::AlreadyExpanded) => {}
            Err(_) => {}
        }
    }

    state.workspace_meta.pending_expanded_paths = remaining;
    effects
}

pub fn workspace_restore_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let mut effects = workspace_expand_pending_effects(state);
    workspace_apply_pending_selection(state);
    effects.extend(github_first_access_effects(state));
    effects.extend(github_issue_list_effects(state, false));
    effects.extend(github_pr_list_access_effects(state, false));
    effects.extend(branch_list_access_effects(state, false));
    effects
}

pub(super) fn workspace_apply_pending_selection(state: &mut ReduceView<'_>) {
    let Some(path) = state.workspace_meta.pending_selected_path.clone() else {
        return;
    };

    if state.file_tree.nodes.contains_key(&path) {
        state.file_tree.select(path);
        state.workspace_meta.pending_selected_path = None;
        state.set_dirty();
    }
}

pub(super) fn reduce_file_tree_expand(state: &mut ReduceView<'_>, path: PathBuf) -> Vec<SideEffect> {
    match state.file_tree.expand(&path) {
        Ok(ExpandAction::NeedsLoad) => {
            state.set_dirty();
            vec![SideEffect::Fs(FsEffect::LoadDirectoryChildren(path))]
        }
        Ok(ExpandAction::AlreadyExpanded) => {
            state.set_dirty();
            Vec::new()
        }
        Err(_) => {
            state.set_dirty();
            Vec::new()
        }
    }
}

pub(super) fn reduce_file_tree_collapse(state: &mut ReduceView<'_>, path: PathBuf) -> Vec<SideEffect> {
    state.file_tree.collapse(&path);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_file_tree_select(state: &mut ReduceView<'_>, path: PathBuf) -> Vec<SideEffect> {
    state.file_tree.select(path);
    state.set_dirty();
    Vec::new()
}

pub(super) fn file_tree_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.file_tree_rows
}

pub(super) fn reduce_file_tree_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    state
        .file_tree
        .move_selection(delta, file_tree_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_file_tree_refresh(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let expanded: Vec<PathBuf> = state
        .file_tree
        .nodes
        .values()
        .filter(|node| node.expanded)
        .map(|node| node.path.clone())
        .collect();

    for path in &expanded {
        state.file_tree.invalidate_children(path);
    }

    let mut effects = Vec::new();
    for path in expanded {
        if let Ok(ExpandAction::NeedsLoad) = state.file_tree.expand(&path) {
            effects.push(SideEffect::Fs(FsEffect::LoadDirectoryChildren(path)));
        }
    }

    state.set_dirty();
    effects
}

pub(super) fn reduce_file_tree_children_loaded(
    state: &mut ReduceView<'_>,
    parent: PathBuf,
    children: Vec<crate::file_tree::DirectoryEntry>,
    error: Option<String>,
) -> Vec<SideEffect> {
    state
        .file_tree
        .apply_children_loaded(&parent, children, error);
    sync_git_statuses_to_file_tree(state);
    state.set_dirty();
    let effects = workspace_expand_pending_effects(state);
    workspace_apply_pending_selection(state);
    effects
}

pub(super) fn preview_viewport_rows(state: &ReduceView<'_>) -> usize {
    diff_viewport_rows(state)
}

pub(super) fn reduce_preview_file(
    state: &mut ReduceView<'_>,
    path: PathBuf,
    line: Option<u32>,
) -> Vec<SideEffect> {
    state.preview.begin_load(path.clone(), line);
    state
        .navigation
        .apply(NavCommand::SelectMainTab(MainTab::Preview));
    state
        .navigation
        .apply(NavCommand::SetFocus(FocusTarget::Main));
    state.set_dirty();
    vec![SideEffect::Fs(FsEffect::LoadPreviewFile(path))]
}

pub(super) fn reduce_preview_loaded(
    state: &mut ReduceView<'_>,
    path: PathBuf,
    result: crate::preview::PreviewLoadResult,
) -> Vec<SideEffect> {
    if state.preview.path.as_ref() != Some(&path) {
        return Vec::new();
    }

    state
        .preview
        .apply_loaded(path, result, preview_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_preview_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.preview.scroll(delta, preview_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_preview_page_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state
        .preview
        .page_scroll(delta, preview_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_fs_changed(state: &mut ReduceView<'_>, paths: Vec<PathBuf>) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    let reload_dirs = state
        .file_tree
        .apply_fs_invalidation(state.repo_root, &paths);
    if !reload_dirs.is_empty() {
        state.set_dirty();
        for dir in reload_dirs {
            effects.push(SideEffect::Fs(FsEffect::LoadDirectoryChildren(dir)));
        }
    }

    if let Some(preview_path) = state.preview.path.clone() {
        if !state.preview.loading
            && crate::watcher::preview_reload_paths(&paths, &preview_path)
            && !preview_file_unchanged(&preview_path, state.preview.loaded_mtime)
        {
            state.preview.begin_reload();
            state.set_dirty();
            effects.push(SideEffect::Fs(FsEffect::LoadPreviewFile(preview_path)));
        }
    }

    if state.workspace_meta.is_git_repo && state.config.git.watch {
        effects.extend(reduce_git_refresh_requested(state));
    }

    effects
}

pub(super) fn preview_file_unchanged(
    path: &std::path::Path,
    loaded_mtime: Option<std::time::SystemTime>,
) -> bool {
    let Some(loaded_mtime) = loaded_mtime else {
        return false;
    };
    std::fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .is_some_and(|modified| modified == loaded_mtime)
}
