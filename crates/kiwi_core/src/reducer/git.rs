
use super::diff::diff_select_file_effects;
use super::navigation::apply_navigation;

use crate::git::{
    branch_move_selection, branch_select_row, branch_selected_name,
    build_panel_rows, clamp_git_scroll, ensure_branch_selection,
    ensure_git_selection, git_move_selection, git_select_row, patch_git_file_entries,
    row_for_path, BranchDetail, BranchEntry, GitFileEntry,
};
use crate::github::GitHubLeftPane;
use crate::navigation::{FocusTarget, LeftNavTab, MainTab, NavCommand};
use crate::state::ReduceView;

use crate::events::{GitEffect, SideEffect};

pub fn git_refresh_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.workspace_meta.is_git_repo || state.git.loading {
        return Vec::new();
    }

    state.set_dirty();
    state.git.loading = true;
    vec![SideEffect::Git(GitEffect::SpawnRefresh)]
}

pub fn branch_list_surface_active(state: &ReduceView<'_>) -> bool {
    state.navigation.main_tab == MainTab::Branches
        || (state.navigation.left_tab == LeftNavTab::Gh
            && state.github.left_pane == GitHubLeftPane::Branches)
}

pub fn branch_list_access_effects(state: &mut ReduceView<'_>, force: bool) -> Vec<SideEffect> {
    if !branch_list_surface_active(state) && !force {
        return Vec::new();
    }

    if !state.workspace_meta.is_git_repo {
        return Vec::new();
    }

    if state.branches.loading {
        return Vec::new();
    }

    if !force && !state.branches.entries.is_empty() {
        return Vec::new();
    }

    state.branches.loading = true;
    state.branches.error = None;
    state.set_dirty();
    vec![SideEffect::Git(GitEffect::SpawnBranchList)]
}

pub fn branch_checkout_effects(state: &mut ReduceView<'_>, branch_name: String) -> Vec<SideEffect> {
    if !state.workspace_meta.is_git_repo || state.branches.checkout_loading {
        return Vec::new();
    }

    if state.git.branch.as_deref() == Some(branch_name.as_str()) {
        return Vec::new();
    }

    state.branches.checkout_loading = true;
    state.branches.checkout_error = None;
    state.set_dirty();
    vec![SideEffect::Git(GitEffect::SpawnBranchCheckout { name: branch_name })]
}

pub fn branch_detail_access_effects(state: &mut ReduceView<'_>, force: bool) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Branches {
        return Vec::new();
    }

    if !state.workspace_meta.is_git_repo {
        return Vec::new();
    }

    let Some(branch_name) = branch_selected_name(state.branches).map(str::to_string) else {
        return Vec::new();
    };

    if state.branches.detail_loading {
        return Vec::new();
    }

    if !force
        && state.branches.detail_for_branch.as_deref() == Some(branch_name.as_str())
        && state.branches.detail.is_some()
    {
        return Vec::new();
    }

    state.branches.detail_loading = true;
    state.branches.detail_error = None;
    state.set_dirty();
    vec![SideEffect::Git(GitEffect::SpawnBranchDetail {
        name: branch_name,
    })]
}

fn clear_branch_detail(branches: &mut crate::state::BranchState) {
    branches.detail = None;
    branches.detail_for_branch = None;
    branches.detail_error = None;
    branches.detail_scroll_offset = 0;
}

pub(super) fn reduce_git_refresh_requested(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    git_refresh_effects(state)
}

pub(super) fn reduce_git_status_updated(
    state: &mut ReduceView<'_>,
    branch: Option<String>,
    remote_repo: Option<String>,
    ahead: u32,
    behind: u32,
    file_entries: Vec<GitFileEntry>,
    error: Option<String>,
) -> Vec<SideEffect> {
    let git_selected = state.git.selected_path.clone();
    let tree_selected = state.file_tree.selected.clone();

    state.git.loading = false;
    state.git.ahead = ahead;
    state.git.behind = behind;

    if let Some(message) = error {
        state.git.error = Some(message.clone());
        state
            .logs
            .push_error(format!("git refresh failed: {message}"));
    } else {
        state.git.error = None;
        state.git.branch = branch;
        state.git.remote_repo = remote_repo;
        let file_patch = patch_git_file_entries(&mut state.git.file_entries, &file_entries);
        sync_git_status_patch_to_file_tree(state, &file_patch);
        let git_rows = build_panel_rows(&state.git.file_entries, state.config.git.show_untracked);
        ensure_git_selection(state.git, &git_rows);
        clamp_git_scroll(
            state.git,
            &git_rows,
            git_rows.len().saturating_sub(git_viewport_rows(state).max(1)),
        );
    }

    if let Some(path) = git_selected {
        if state
            .git
            .file_entries
            .iter()
            .any(|entry| entry.path == path)
        {
            state.git.selected_path = Some(path);
        } else {
            state.git.selected_path = None;
        }
    }

    if let Some(path) = tree_selected {
        if state.file_tree.nodes.contains_key(&path) {
            state.file_tree.selected = Some(path);
        }
    }

    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_branch_refresh(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let mut effects = branch_list_access_effects(state, true);
    effects.extend(branch_detail_access_effects(state, true));
    effects
}

pub(super) fn reduce_branch_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = state.viewport.branches_rows;
    branch_move_selection(state.branches, delta, viewport_rows);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_branch_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    let viewport_rows = state.viewport.branches_rows;
    let previous = branch_selected_name(state.branches).map(str::to_string);
    branch_select_row(state.branches, index, viewport_rows);
    let current = branch_selected_name(state.branches).map(str::to_string);
    if previous != current {
        clear_branch_detail(state.branches);
    }
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_branch_checkout_selected(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(branch_name) = branch_selected_name(state.branches).map(str::to_string) else {
        return Vec::new();
    };

    branch_checkout_effects(state, branch_name)
}

pub(super) fn reduce_branch_list_loaded(
    state: &mut ReduceView<'_>,
    entries: Vec<BranchEntry>,
    error: Option<String>,
) -> Vec<SideEffect> {
    let selected_name = state
        .branches
        .selected_index
        .and_then(|index| state.branches.entries.get(index))
        .map(|entry| entry.name.clone());

    state.branches.loading = false;
    if let Some(message) = error {
        state.branches.error = Some(message.clone());
        state
            .logs
            .push_error(format!("branch list failed: {message}"));
    } else {
        state.branches.error = None;
        state.branches.entries = entries;
    }

    if let Some(name) = selected_name {
        if let Some(index) = state
            .branches
            .entries
            .iter()
            .position(|entry| entry.name == name)
        {
            state.branches.selected_index = Some(index);
            state.set_dirty();
            return Vec::new();
        }
    }

    ensure_branch_selection(state.branches);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_branch_checkout_completed(
    state: &mut ReduceView<'_>,
    branch_name: String,
    error: Option<String>,
) -> Vec<SideEffect> {
    state.branches.checkout_loading = false;

    if let Some(message) = error {
        state.branches.checkout_error = Some(message.clone());
        state
            .logs
            .push_error(format!("checkout {branch_name} failed: {message}"));
        state.set_dirty();
        return Vec::new();
    }

    state.branches.checkout_error = None;
    let mut effects = git_refresh_effects(state);
    effects.extend(branch_list_access_effects(state, true));
    if let Some(index) = state
        .branches
        .entries
        .iter()
        .position(|entry| entry.name == branch_name)
    {
        state.branches.selected_index = Some(index);
    } else {
        ensure_branch_selection(state.branches);
    }
    state.set_dirty();
    effects
}

pub(super) fn reduce_branch_detail_loaded(
    state: &mut ReduceView<'_>,
    name: String,
    detail: Option<BranchDetail>,
    error: Option<String>,
) -> Vec<SideEffect> {
    state.branches.detail_loading = false;
    if let Some(message) = error {
        state.branches.detail = None;
        state.branches.detail_for_branch = Some(name.clone());
        state.branches.detail_error = Some(message.clone());
        state
            .logs
            .push_error(format!("branch detail failed for {name}: {message}"));
    } else {
        state.branches.detail_error = None;
        state.branches.detail_for_branch = Some(name);
        state.branches.detail = detail;
    }
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_branch_detail_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let line_count = state
        .branches
        .detail
        .as_ref()
        .map(|detail| detail.display_lines(state.git.ahead, state.git.behind).len())
        .unwrap_or(0);
    let viewport_rows = state.viewport.github_detail_rows.max(1);
    let max_offset = line_count.saturating_sub(viewport_rows);
    let next = (state.branches.detail_scroll_offset as i32 + delta).clamp(0, max_offset as i32);
    state.branches.detail_scroll_offset = usize::try_from(next).unwrap_or(0);
    state.set_dirty();
    Vec::new()
}

pub(super) fn sync_git_status_patch_to_file_tree(
    state: &mut ReduceView<'_>,
    patch: &crate::git::GitFileStatusPatch,
) {
    if patch.is_empty() {
        return;
    }

    state
        .file_tree
        .apply_git_status_patch(state.repo_root, patch, state.config.git.show_untracked);
}

pub(super) fn sync_git_statuses_to_file_tree(state: &mut ReduceView<'_>) {
    let entries = state.git.file_entries.clone();
    state
        .file_tree
        .apply_git_statuses(state.repo_root, &entries, state.config.git.show_untracked);
}

pub(super) fn git_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.git_rows
}

pub(super) fn reduce_git_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let viewport_rows = git_viewport_rows(state);
    let rows = build_panel_rows(&state.git.file_entries, state.config.git.show_untracked);
    git_move_selection(state.git, &rows, delta, viewport_rows);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_git_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    let viewport_rows = git_viewport_rows(state);
    let rows = build_panel_rows(&state.git.file_entries, state.config.git.show_untracked);
    git_select_row(state.git, &rows, index, viewport_rows);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_git_open_selected(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(path) = state.git.selected_path.clone() else {
        return Vec::new();
    };

    apply_navigation(state, NavCommand::SelectMainTab(MainTab::Diff));
    state
        .navigation
        .apply(NavCommand::SetFocus(FocusTarget::Main));
    diff_select_file_effects(state, path)
}

pub(super) fn sync_git_selection_for_path(state: &mut ReduceView<'_>, path: &str) {
    let viewport_rows = git_viewport_rows(state);
    let rows = build_panel_rows(&state.git.file_entries, state.config.git.show_untracked);
    if let Some(row) = row_for_path(&rows, path) {
        git_select_row(state.git, &rows, row, viewport_rows);
    } else {
        state.git.selected_path = Some(path.to_string());
    }
}
