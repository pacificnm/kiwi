use std::time::SystemTime;

use super::agent::agent_spawn_effects_if_needed;
use super::git::git_refresh_effects;

use crate::github::{
    apply_label_picker_load, ensure_issue_selection, ensure_pr_selection,
    format_issue_agent_prompt, format_pr_agent_prompt, issue_body_excerpt_from_detail, issue_move_selection, issue_select_row,
    missing_browser_target_message, page_scroll_issue_detail, pr_move_selection, pr_select_row,
    pull_request_is_mergeable, resolve_browser_target, scroll_issue_detail, selected_pull_request,
    GhContextMenuAction, GhContextMenuState, GhContextTarget, GitHubLeftPane,
    IssueDetailLoadResult, IssueListLoadResult, LabelPickerState,
    PrDetailLoadResult, PrListLoadResult, ISSUE_LIST_CACHE_SECS, PR_LIST_CACHE_SECS,
};
use crate::navigation::{FocusTarget, LeftNavTab, MainTab, NavCommand};
use crate::state::{PalettePrompt, ReduceView};

use crate::events::{AgentEffect, GitHubEffect, SideEffect};

pub fn github_refresh_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.set_dirty();
    state.github.loading = true;
    state.github.auth_checked = false;
    state.github.issues_loaded_at = None;
    state.github.prs_loaded_at = None;
    clear_issue_detail_cache(state.github);
    clear_pr_detail_cache(state.github);
    vec![SideEffect::GitHub(GitHubEffect::SpawnAuthCheck)]
}

pub fn github_first_access_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !github_surface_active(state) {
        return Vec::new();
    }

    if state.github.auth_checked || state.github.loading {
        return Vec::new();
    }

    state.github.loading = true;
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnAuthCheck)]
}

pub fn github_issue_list_effects(state: &mut ReduceView<'_>, force: bool) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        return Vec::new();
    }

    if !github_issue_list_surface_active(state) {
        return Vec::new();
    }

    if state.github.issues_loading {
        return Vec::new();
    }

    if !force && issue_list_cache_fresh(state.github) {
        return Vec::new();
    }

    state.github.issues_loading = true;
    state.github.issues_error = None;
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnIssueList)]
}

pub fn github_pr_list_effects(state: &mut ReduceView<'_>, force: bool) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        return Vec::new();
    }

    if !github_pr_list_surface_active(state) {
        return Vec::new();
    }

    if state.github.prs_loading {
        return Vec::new();
    }

    if !force && pr_list_cache_fresh(state.github) {
        return Vec::new();
    }

    state.github.prs_loading = true;
    state.github.prs_error = None;
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnPrList)]
}

pub fn github_pr_list_access_effects(state: &mut ReduceView<'_>, force: bool) -> Vec<SideEffect> {
    github_pr_list_effects(state, force)
}

pub fn github_issue_detail_access_effects(
    state: &mut ReduceView<'_>,
    force: bool,
) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Issues {
        return Vec::new();
    }

    let Some(number) = selected_issue_number(state.github) else {
        return Vec::new();
    };

    github_issue_detail_effects(state, number, force)
}

pub(super) fn clear_issue_detail_cache(github: &mut crate::state::GitHubState) {
    github.issue_detail_number = None;
    github.issue_detail = None;
    github.issue_detail_loading = false;
    github.issue_detail_error = None;
    github.issue_detail_scroll_offset = 0;
}

pub(super) fn clear_pr_detail_cache(github: &mut crate::state::GitHubState) {
    github.pr_detail_number = None;
    github.pr_detail = None;
    github.pr_detail_loading = false;
    github.pr_detail_error = None;
    github.pr_detail_scroll_offset = 0;
}

pub fn github_pr_detail_access_effects(state: &mut ReduceView<'_>, force: bool) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Prs {
        return Vec::new();
    }

    let Some(number) = selected_pr_number(state.github) else {
        return Vec::new();
    };

    github_pr_detail_effects(state, number, force)
}

pub(super) fn selected_pr_number(github: &crate::state::GitHubState) -> Option<u32> {
    github.selected_pr
}

pub(super) fn github_pr_detail_effects(
    state: &mut ReduceView<'_>,
    number: u32,
    force: bool,
) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        return Vec::new();
    }

    if state.github.pr_detail_loading && state.github.pr_detail_number == Some(number) {
        return Vec::new();
    }

    if !force
        && state.github.pr_detail_number == Some(number)
        && state.github.pr_detail.is_some()
        && state.github.pr_detail_error.is_none()
    {
        return Vec::new();
    }

    state.github.pr_detail_loading = true;
    state.github.pr_detail_error = None;
    if state.github.pr_detail_number != Some(number) {
        state.github.pr_detail = None;
        state.github.pr_detail_scroll_offset = 0;
    }
    state.github.pr_detail_number = Some(number);
    state.set_dirty();

    vec![SideEffect::GitHub(GitHubEffect::SpawnPrDetail { number })]
}

pub(super) fn selected_issue_number(github: &crate::state::GitHubState) -> Option<u32> {
    github.selected_issue
}

pub(super) fn github_issue_detail_effects(
    state: &mut ReduceView<'_>,
    number: u32,
    force: bool,
) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        return Vec::new();
    }

    if state.github.issue_detail_loading
        && state.github.issue_detail_number == Some(number)
    {
        return Vec::new();
    }

    if !force
        && state.github.issue_detail_number == Some(number)
        && state.github.issue_detail.is_some()
        && state.github.issue_detail_error.is_none()
    {
        return Vec::new();
    }

    state.github.issue_detail_loading = true;
    state.github.issue_detail_error = None;
    if state.github.issue_detail_number != Some(number) {
        state.github.issue_detail = None;
        state.github.issue_detail_scroll_offset = 0;
    }
    state.github.issue_detail_number = Some(number);
    state.set_dirty();

    vec![SideEffect::GitHub(GitHubEffect::SpawnIssueDetail { number })]
}

pub(super) fn github_surface_active(state: &ReduceView<'_>) -> bool {
    state.navigation.left_tab == LeftNavTab::Gh
        || matches!(state.navigation.main_tab, MainTab::Issues | MainTab::Prs)
}

pub(super) fn github_issue_list_surface_active(state: &ReduceView<'_>) -> bool {
    state.navigation.main_tab == MainTab::Issues || state.navigation.left_tab == LeftNavTab::Gh
}

pub(super) fn issue_list_cache_fresh(github: &crate::state::GitHubState) -> bool {
    let Some(loaded_at) = github.issues_loaded_at else {
        return false;
    };

    loaded_at
        .elapsed()
        .map(|elapsed| elapsed.as_secs() < ISSUE_LIST_CACHE_SECS)
        .unwrap_or(false)
}

pub(super) fn github_pr_list_surface_active(state: &ReduceView<'_>) -> bool {
    state.navigation.main_tab == MainTab::Prs || state.navigation.left_tab == LeftNavTab::Gh
}

pub(super) fn pr_list_cache_fresh(github: &crate::state::GitHubState) -> bool {
    let Some(loaded_at) = github.prs_loaded_at else {
        return false;
    };

    loaded_at
        .elapsed()
        .map(|elapsed| elapsed.as_secs() < PR_LIST_CACHE_SECS)
        .unwrap_or(false)
}

pub(super) fn reduce_github_refresh_requested(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    github_refresh_effects(state)
}

pub(super) fn reduce_github_auth_checked(
    state: &mut ReduceView<'_>,
    result: crate::github::GitHubAuthCheckResult,
) -> Vec<SideEffect> {
    state.github.loading = false;
    state.github.auth_checked = true;
    state.github.auth_ok = result.auth_ok;
    state.github.error_kind = result.error_kind;
    state.github.error = if result.auth_ok {
        None
    } else {
        Some(result.message)
    };
    state.set_dirty();

    if result.auth_ok {
        let mut effects = github_issue_list_effects(state, true);
        effects.extend(github_pr_list_effects(state, true));
        effects
    } else {
        Vec::new()
    }
}

pub(super) fn reduce_github_issues_loaded(
    state: &mut ReduceView<'_>,
    result: IssueListLoadResult,
) -> Vec<SideEffect> {
    state.github.issues_loading = false;
    state.github.issues = result.issues;
    state.github.issues_error = result.error;
    state.github.issues_loaded_at = Some(SystemTime::now());
    ensure_issue_selection(state.github);
    state.set_dirty();

    if state.navigation.main_tab == MainTab::Issues {
        if let Some(number) = selected_issue_number(state.github) {
            return github_issue_detail_effects(state, number, true);
        }
    }

    Vec::new()
}

pub(super) fn reduce_github_prs_loaded(
    state: &mut ReduceView<'_>,
    result: PrListLoadResult,
) -> Vec<SideEffect> {
    state.github.prs_loading = false;
    state.github.prs = result.prs;
    state.github.prs_error = result.error;
    state.github.prs_loaded_at = Some(SystemTime::now());
    ensure_pr_selection(state.github);
    state.set_dirty();

    if state.navigation.main_tab == MainTab::Prs {
        if let Some(number) = selected_pr_number(state.github) {
            return github_pr_detail_effects(state, number, true);
        }
    }

    Vec::new()
}

pub(super) fn reduce_github_issue_detail_loaded(
    state: &mut ReduceView<'_>,
    number: u32,
    result: IssueDetailLoadResult,
) -> Vec<SideEffect> {
    if state.github.issue_detail_number != Some(number) {
        return Vec::new();
    }

    state.github.issue_detail_loading = false;
    state.github.issue_detail_error = result.error;
    state.github.issue_detail = result.detail;
    if state.github.issue_detail.is_some() {
        clamp_issue_detail_scroll(state);
    }
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_pr_detail_loaded(
    state: &mut ReduceView<'_>,
    number: u32,
    result: PrDetailLoadResult,
) -> Vec<SideEffect> {
    if state.github.pr_detail_number != Some(number) {
        return Vec::new();
    }

    state.github.pr_detail_loading = false;
    state.github.pr_detail_error = result.error;
    state.github.pr_detail = result.detail;
    if state.github.pr_detail.is_some() {
        clamp_pr_detail_scroll(state);
    }
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_move_issue_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = issues_viewport_rows(state);
    issue_move_selection(state.github, delta, viewport_rows);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_select_issue(state: &mut ReduceView<'_>, row_index: usize) -> Vec<SideEffect> {
    let viewport_rows = issues_viewport_rows(state);
    issue_select_row(state.github, row_index, viewport_rows);
    state.set_dirty();
    Vec::new()
}

pub(super) fn issues_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.github_list_rows
}

pub(super) fn reduce_github_move_pr_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = prs_viewport_rows(state);
    pr_move_selection(state.github, delta, viewport_rows);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_select_pr(state: &mut ReduceView<'_>, row_index: usize) -> Vec<SideEffect> {
    let viewport_rows = prs_viewport_rows(state);
    pr_select_row(state.github, row_index, viewport_rows);
    state.set_dirty();
    Vec::new()
}

pub(super) fn prs_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.github_list_rows
}

pub(super) fn reduce_github_open_selected(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    match state.github.left_pane {
        GitHubLeftPane::Issues => {
            let Some(number) = selected_issue_number(state.github) else {
                return Vec::new();
            };

            state
                .navigation
                .apply(NavCommand::SelectMainTab(MainTab::Issues));
            state
                .navigation
                .apply(NavCommand::SetFocus(FocusTarget::Main));
            state.set_dirty();
            github_issue_detail_effects(state, number, true)
        }
        GitHubLeftPane::Prs => {
            let Some(number) = selected_pr_number(state.github) else {
                return Vec::new();
            };

            state
                .navigation
                .apply(NavCommand::SelectMainTab(MainTab::Prs));
            state
                .navigation
                .apply(NavCommand::SetFocus(FocusTarget::Main));
            state.set_dirty();
            github_pr_detail_effects(state, number, true)
        }
    }
}

pub(super) fn reduce_github_select_left_pane(
    state: &mut ReduceView<'_>,
    pane: GitHubLeftPane,
) -> Vec<SideEffect> {
    state.github.left_pane = pane;
    match pane {
        GitHubLeftPane::Issues => {
            state
                .navigation
                .apply(NavCommand::SelectMainTab(MainTab::Issues));
        }
        GitHubLeftPane::Prs => {
            state
                .navigation
                .apply(NavCommand::SelectMainTab(MainTab::Prs));
        }
    }
    state.set_dirty();
    match pane {
        GitHubLeftPane::Issues => github_issue_list_effects(state, false),
        GitHubLeftPane::Prs => github_pr_list_effects(state, false),
    }
}

pub(super) fn reduce_github_issue_detail_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let line_count = issue_detail_line_count(state.github);
    let viewport_rows = issue_detail_viewport_rows(state);
    scroll_issue_detail(
        &mut state.github.issue_detail_scroll_offset,
        delta,
        line_count,
        viewport_rows,
    );
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_issue_detail_page_scroll(
    state: &mut ReduceView<'_>,
    delta: i32,
) -> Vec<SideEffect> {
    let line_count = issue_detail_line_count(state.github);
    let viewport_rows = issue_detail_viewport_rows(state);
    page_scroll_issue_detail(
        &mut state.github.issue_detail_scroll_offset,
        delta,
        line_count,
        viewport_rows,
    );
    state.set_dirty();
    Vec::new()
}

pub(super) fn issue_detail_line_count(github: &crate::state::GitHubState) -> usize {
    github
        .issue_detail
        .as_ref()
        .map(|detail| detail.display_lines.len())
        .unwrap_or(0)
}

pub(super) fn clamp_issue_detail_scroll(state: &mut ReduceView<'_>) {
    let line_count = issue_detail_line_count(state.github);
    let viewport_rows = issue_detail_viewport_rows(state);
    let max_offset = line_count.saturating_sub(viewport_rows);
    if state.github.issue_detail_scroll_offset > max_offset {
        state.github.issue_detail_scroll_offset = max_offset;
    }
}

pub(super) fn issue_detail_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.github_detail_rows
}

pub(super) fn reduce_github_pr_detail_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let line_count = pr_detail_line_count(state.github);
    let viewport_rows = pr_detail_viewport_rows(state);
    scroll_issue_detail(
        &mut state.github.pr_detail_scroll_offset,
        delta,
        line_count,
        viewport_rows,
    );
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_pr_detail_page_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let line_count = pr_detail_line_count(state.github);
    let viewport_rows = pr_detail_viewport_rows(state);
    page_scroll_issue_detail(
        &mut state.github.pr_detail_scroll_offset,
        delta,
        line_count,
        viewport_rows,
    );
    state.set_dirty();
    Vec::new()
}

pub(super) fn pr_detail_line_count(github: &crate::state::GitHubState) -> usize {
    github
        .pr_detail
        .as_ref()
        .map(|detail| detail.display_lines.len())
        .unwrap_or(0)
}

pub(super) fn clamp_pr_detail_scroll(state: &mut ReduceView<'_>) {
    let line_count = pr_detail_line_count(state.github);
    let viewport_rows = pr_detail_viewport_rows(state);
    let max_offset = line_count.saturating_sub(viewport_rows);
    if state.github.pr_detail_scroll_offset > max_offset {
        state.github.pr_detail_scroll_offset = max_offset;
    }
}

pub(super) fn pr_detail_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.github_detail_rows
}

pub(super) fn reduce_github_issue_comment_completed(
    state: &mut ReduceView<'_>,
    number: u32,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if state.github.selected_issue != Some(number) {
        return Vec::new();
    }

    if result.success {
        state.github.issue_action_message = Some(format!("Comment posted on #{number}"));
        state.github.issues_loaded_at = None;
        clear_issue_detail_cache(state.github);
        let mut effects = github_issue_list_effects(state, true);
        effects.extend(github_issue_detail_effects(state, number, true));
        state.set_dirty();
        return effects;
    }

    state.github.issue_action_message = result.error.clone();
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_issue_create_branch_completed(
    state: &mut ReduceView<'_>,
    number: u32,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if state.github.selected_issue != Some(number) {
        return Vec::new();
    }

    if result.success {
        state.github.issue_action_message = result
            .detail
            .or_else(|| Some(format!("Checked out branch for issue #{number}")));
        state.set_dirty();
        return git_refresh_effects(state);
    }

    state.github.issue_action_message = result.error.clone();
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_repo_labels_loaded(
    state: &mut ReduceView<'_>,
    result: crate::github::RepoLabelsLoadResult,
) -> Vec<SideEffect> {
    let Some(picker) = state.github.label_picker.as_mut() else {
        return Vec::new();
    };

    let existing = picker.existing_labels.clone();
    apply_label_picker_load(picker, result, &existing);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_issue_labels_applied(
    state: &mut ReduceView<'_>,
    number: u32,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    let Some(picker) = state.github.label_picker.as_mut() else {
        return Vec::new();
    };

    if picker.issue_number != number {
        return Vec::new();
    }

    picker.applying = false;

    if result.success {
        state.github.label_picker = None;
        state.github.issue_action_message = Some(format!("Labels updated on #{number}"));
        state.github.issues_loaded_at = None;
        clear_issue_detail_cache(state.github);
        let mut effects = github_issue_list_effects(state, true);
        effects.extend(github_issue_detail_effects(state, number, true));
        state.set_dirty();
        return effects;
    }

    picker.error = result.error.clone();
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_label_picker_move(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if let Some(picker) = state.github.label_picker.as_mut() {
        picker.move_cursor(delta);
        state.set_dirty();
    }
    Vec::new()
}

pub(super) fn reduce_github_label_picker_toggle(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if let Some(picker) = state.github.label_picker.as_mut() {
        if !picker.loading && !picker.applying {
            picker.toggle_cursor();
            state.set_dirty();
        }
    }
    Vec::new()
}

pub(super) fn reduce_github_label_picker_apply(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(picker) = state.github.label_picker.as_mut() else {
        return Vec::new();
    };

    if picker.loading || picker.applying {
        return Vec::new();
    }

    let labels = picker.labels_to_add();
    if labels.is_empty() {
        state
            .notifications
            .show_toast("Select at least one new label");
        state.set_dirty();
        return Vec::new();
    }

    let number = picker.issue_number;
    picker.applying = true;
    picker.error = None;
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnIssueLabelApply { number, labels })]
}

pub(super) fn reduce_github_label_picker_cancel(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.github.label_picker.is_some() {
        state.github.label_picker = None;
        state.set_dirty();
    }
    Vec::new()
}

pub(super) fn reduce_github_context_menu_open(
    state: &mut ReduceView<'_>,
    anchor_x: u16,
    anchor_y: u16,
    target: GhContextTarget,
) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let effects = match target {
        GhContextTarget::Issue { list_index } => {
            let viewport_rows = issues_viewport_rows(state);
            issue_select_row(state.github, list_index, viewport_rows);
            Vec::new()
        }
        GhContextTarget::PullRequest { list_index } => {
            let viewport_rows = prs_viewport_rows(state);
            pr_select_row(state.github, list_index, viewport_rows);
            Vec::new()
        }
    };

    state.github.context_menu = Some(GhContextMenuState::new(
        target,
        anchor_x,
        anchor_y,
        match target {
            GhContextTarget::PullRequest { .. } => {
                selected_pull_request(state.github).is_some_and(pull_request_is_mergeable)
            }
            GhContextTarget::Issue { .. } => false,
        },
    ));
    state.set_dirty();
    effects
}

pub(super) fn reduce_github_context_menu_move(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if let Some(menu) = state.github.context_menu.as_mut() {
        menu.move_cursor(delta);
        state.set_dirty();
    }
    Vec::new()
}

pub(super) fn reduce_github_context_menu_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    if let Some(menu) = state.github.context_menu.as_mut() {
        if index < menu.items.len() {
            menu.cursor = index;
            state.set_dirty();
        }
    }
    reduce_github_context_menu_execute(state)
}

pub(super) fn reduce_github_context_menu_execute(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(menu) = state.github.context_menu.take() else {
        return Vec::new();
    };

    let Some(action) = menu.selected_action() else {
        state.set_dirty();
        return Vec::new();
    };

    execute_github_list_action(state, menu.target, action)
}

pub(super) fn reduce_github_list_action(
    state: &mut ReduceView<'_>,
    target: GhContextTarget,
    action: GhContextMenuAction,
) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    execute_github_list_action(state, target, action)
}

pub(super) fn execute_github_list_action(
    state: &mut ReduceView<'_>,
    target: GhContextTarget,
    action: GhContextMenuAction,
) -> Vec<SideEffect> {
    match action {
        GhContextMenuAction::View => match target {
            GhContextTarget::Issue { list_index } => {
                let viewport_rows = issues_viewport_rows(state);
                issue_select_row(state.github, list_index, viewport_rows);
                let Some(number) = selected_issue_number(state.github) else {
                    return Vec::new();
                };
                state
                    .navigation
                    .apply(NavCommand::SelectMainTabUnpaired(MainTab::Issues));
                state
                    .navigation
                    .apply(NavCommand::SetFocus(FocusTarget::Main));
                state.set_dirty();
                github_issue_detail_effects(state, number, true)
            }
            GhContextTarget::PullRequest { list_index } => {
                let viewport_rows = prs_viewport_rows(state);
                pr_select_row(state.github, list_index, viewport_rows);
                let Some(number) = selected_pr_number(state.github) else {
                    return Vec::new();
                };
                state
                    .navigation
                    .apply(NavCommand::SelectMainTabUnpaired(MainTab::Prs));
                state
                    .navigation
                    .apply(NavCommand::SetFocus(FocusTarget::Main));
                state.set_dirty();
                github_pr_detail_effects(state, number, true)
            }
        },
        GhContextMenuAction::CreateBranch => {
            if let GhContextTarget::Issue { list_index } = target {
                let viewport_rows = issues_viewport_rows(state);
                issue_select_row(state.github, list_index, viewport_rows);
            }
            github_issue_create_branch_effects(state, selected_issue_number(state.github))
        }
        GhContextMenuAction::Comment => github_issue_comment_prompt_effects(state),
        GhContextMenuAction::AddLabels => github_issue_label_picker_effects(state),
        GhContextMenuAction::Merge => github_pr_merge_effects(state),
        GhContextMenuAction::OpenInBrowser => github_open_in_browser_effects(state),
        GhContextMenuAction::SendToAgent => match target {
            GhContextTarget::Issue { list_index } => {
                let viewport_rows = issues_viewport_rows(state);
                issue_select_row(state.github, list_index, viewport_rows);
                github_send_issue_to_agent_effects(state)
            }
            GhContextTarget::PullRequest { list_index } => {
                let viewport_rows = prs_viewport_rows(state);
                pr_select_row(state.github, list_index, viewport_rows);
                github_send_pr_to_agent_effects(state)
            }
        },
    }
}

pub(super) fn reduce_github_context_menu_cancel(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.github.context_menu.is_some() {
        state.github.context_menu = None;
        state.set_dirty();
    }
    Vec::new()
}

pub(super) fn github_issue_create_branch_effects(
    state: &mut ReduceView<'_>,
    number: Option<u32>,
) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(number) = number else {
        state
            .notifications
            .show_toast("Select an issue in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    state.github.issue_action_message = Some(format!("Creating branch for issue #{number}..."));
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnIssueCreateBranch { number })]
}

pub(super) fn github_issue_comment_prompt_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(number) = selected_issue_number(state.github) else {
        state
            .notifications
            .show_toast("Select an issue in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    let focus = state.navigation.focus;
    state
        .palette
        .begin_prompt(PalettePrompt::GitHubIssueComment { number }, focus);
    state.navigation.focus = FocusTarget::CommandPalette;
    state.set_dirty();
    Vec::new()
}

pub(super) fn github_issue_label_picker_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(number) = selected_issue_number(state.github) else {
        state
            .notifications
            .show_toast("Select an issue in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    let existing_labels = issue_labels_for_number(state.github, number);
    state.github.label_picker = Some(LabelPickerState::new(number, existing_labels));
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnRepoLabels)]
}

pub(super) fn github_open_in_browser_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(target) = resolve_browser_target(state.navigation, state.github) else {
        state
            .notifications
            .show_toast(missing_browser_target_message(
                state.navigation,
                state.github,
            ));
        state.set_dirty();
        return Vec::new();
    };

    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnOpenBrowser { target })]
}

pub(super) fn github_pr_merge_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(number) = selected_pr_number(state.github) else {
        state
            .notifications
            .show_toast("Select a PR in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    let mergeable = selected_pull_request(state.github).is_some_and(pull_request_is_mergeable);
    if !mergeable {
        state
            .notifications
            .show_toast("Only open, non-draft pull requests can be merged");
        state.set_dirty();
        return Vec::new();
    }

    state.github.issue_action_message = Some(format!("Merging pull request #{number}..."));
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnPrMerge { number })]
}

pub(super) fn issue_labels_for_number(github: &crate::state::GitHubState, number: u32) -> Vec<String> {
    if github
        .issue_detail
        .as_ref()
        .is_some_and(|detail| detail.number == number)
    {
        return github
            .issue_detail
            .as_ref()
            .map(|detail| detail.labels.clone())
            .unwrap_or_default();
    }

    github
        .issues
        .iter()
        .find(|issue| issue.number == number)
        .map(|issue| issue.labels.clone())
        .unwrap_or_default()
}

pub(super) fn github_send_issue_to_agent_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(number) = selected_issue_number(state.github) else {
        state
            .notifications
            .show_toast("Select an issue in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    let title = issue_title_for_number(state.github, number)
        .unwrap_or_else(|| "Untitled issue".to_string());
    let body_excerpt = state
        .github
        .issue_detail
        .as_ref()
        .filter(|detail| detail.number == number)
        .and_then(issue_body_excerpt_from_detail);
    let prompt = format_issue_agent_prompt(number, &title, body_excerpt.as_deref());
    github_send_prompt_to_agent_effects(state, prompt)
}

pub(super) fn github_send_pr_to_agent_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(number) = selected_pr_number(state.github) else {
        state
            .notifications
            .show_toast("Select a PR in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    let title = pr_title_for_number(state.github, number)
        .unwrap_or_else(|| "Untitled pull request".to_string());
    let prompt = format_pr_agent_prompt(number, &title);
    github_send_prompt_to_agent_effects(state, prompt)
}

pub(super) fn github_send_prompt_to_agent_effects(
    state: &mut ReduceView<'_>,
    prompt: String,
) -> Vec<SideEffect> {
    state
        .navigation
        .apply(NavCommand::SelectMainTab(MainTab::Agent));
    state
        .navigation
        .apply(NavCommand::SetFocus(FocusTarget::Main));
    state.set_dirty();

    let mut effects = agent_spawn_effects_if_needed(state);
    effects.push(SideEffect::Agent(AgentEffect::Write(prompt.into_bytes())));
    effects
}

pub(super) fn issue_title_for_number(github: &crate::state::GitHubState, number: u32) -> Option<String> {
    github
        .issues
        .iter()
        .find(|issue| issue.number == number)
        .map(|issue| issue.title.clone())
        .or_else(|| {
            github
                .issue_detail
                .as_ref()
                .filter(|detail| detail.number == number)
                .map(|detail| detail.title.clone())
        })
}

pub(super) fn pr_title_for_number(github: &crate::state::GitHubState, number: u32) -> Option<String> {
    github
        .prs
        .iter()
        .find(|pr| pr.number == number)
        .map(|pr| pr.title.clone())
        .or_else(|| {
            github
                .pr_detail
                .as_ref()
                .filter(|detail| detail.number == number)
                .map(|detail| detail.title.clone())
        })
}

pub(super) fn reduce_github_open_in_browser(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(target) = resolve_browser_target(state.navigation, state.github) else {
        state
            .notifications
            .show_toast(missing_browser_target_message(
                state.navigation,
                state.github,
            ));
        state.set_dirty();
        return Vec::new();
    };

    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnOpenBrowser { target })]
}

pub(super) fn reduce_github_open_browser_completed(
    state: &mut ReduceView<'_>,
    target: crate::github::GitHubBrowserTarget,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if result.success {
        state.github.issue_action_message = result.detail.or_else(|| {
            Some(format!(
                "Opened {} #{} in browser",
                target.label(),
                target.number()
            ))
        });
    } else if let Some(error) = result.error {
        state.github.issue_action_message = Some(error);
    }
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_pr_create_completed(
    state: &mut ReduceView<'_>,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if result.success {
        state.github.issue_action_message = result
            .detail
            .or_else(|| Some("Pull request created".to_string()));
        state.github.prs_loaded_at = None;
        clear_pr_detail_cache(state.github);
        state.set_dirty();
        return github_pr_list_effects(state, true);
    }

    if let Some(error) = result.error {
        state.github.issue_action_message = Some(error);
    }
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_github_pr_merge_completed(
    state: &mut ReduceView<'_>,
    number: u32,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if state.github.selected_pr != Some(number) {
        return Vec::new();
    }

    if result.success {
        state.github.issue_action_message = result
            .detail
            .or_else(|| Some(format!("Merged pull request #{number}")));
        state.github.prs_loaded_at = None;
        clear_pr_detail_cache(state.github);
        state.set_dirty();
        return github_pr_list_effects(state, true);
    }

    state.github.issue_action_message = result.error;
    state.set_dirty();
    Vec::new()
}
