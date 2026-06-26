use std::path::PathBuf;
use std::time::SystemTime;

use crate::agent::infer_status_from_scrollback;
use crate::clipboard::{resolve_copy_text, PasteTarget};
use crate::commands::{execute_command, history_input_for_id, refresh_matches};
use crate::file_tree::ExpandAction;
use crate::git::{
    adjacent_changed_file, branch_move_selection, branch_select_row, branch_selected_name,
    build_panel_rows, changed_file_paths, ensure_branch_selection, ensure_git_selection,
    git_move_selection, git_select_row, patch_git_file_entries, row_for_path, BranchEntry,
    GitFileEntry,
};
use crate::github::{
    advance_pr_create_prompt, apply_label_picker_load, ensure_issue_selection, ensure_pr_selection,
    issue_move_selection, issue_select_row, missing_browser_target_message,
    page_scroll_issue_detail, pr_move_selection, pr_select_row, resolve_browser_target,
    scroll_issue_detail, GitHubLeftPane, IssueDetailLoadResult, IssueListLoadResult,
    PrCreatePromptAdvance, PrDetailLoadResult, PrListLoadResult, ISSUE_LIST_CACHE_SECS,
    PR_LIST_CACHE_SECS,
};
use crate::layout::{agent_pty_size, compute_layout, shell_pty_size, FocusTarget};
use crate::navigation::{LeftNavTab, MainTab, NavCommand};
use crate::state::PalettePrompt;

use super::app_state::AppState;
use super::event::{AppCommand, AppEvent, SideEffect};

pub fn reduce(state: &mut AppState, event: AppEvent) -> Vec<SideEffect> {
    match event {
        AppEvent::Command(command) => reduce_command(state, command),
        AppEvent::TerminalResize { width, height } => reduce_terminal_resize(state, width, height),
        AppEvent::GitRefreshRequested => reduce_git_refresh_requested(state),
        AppEvent::GitStatusUpdated {
            branch,
            ahead,
            behind,
            file_entries,
            error,
        } => reduce_git_status_updated(state, branch, ahead, behind, file_entries, error),
        AppEvent::Quit => {
            state.dirty = true;
            vec![SideEffect::Quit]
        }
        AppEvent::ShellOutput(data) => reduce_shell_output(state, data),
        AppEvent::ShellExited(_code) => reduce_shell_exited(state),
        AppEvent::AgentOutput(data) => reduce_agent_output(state, data),
        AppEvent::AgentExited(code) => reduce_agent_exited(state, code),
        AppEvent::FileTreeChildrenLoaded {
            parent,
            children,
            error,
        } => reduce_file_tree_children_loaded(state, parent, children, error),
        AppEvent::PreviewLoaded { path, result } => reduce_preview_loaded(state, path, result),
        AppEvent::SearchCompleted {
            generation,
            results,
            truncated,
            error,
        } => reduce_search_completed(state, generation, results, truncated, error),
        AppEvent::EditorLaunched { path, command } => reduce_editor_launched(state, path, command),
        AppEvent::EditorLaunchFailed {
            path,
            error,
            show_modal,
        } => reduce_editor_launch_failed(state, path, error, show_modal),
        AppEvent::FsChanged { paths } => reduce_fs_changed(state, paths),
        AppEvent::DiffLoaded { result } => reduce_diff_loaded(state, result),
        AppEvent::GitHubAuthChecked { result } => reduce_github_auth_checked(state, result),
        AppEvent::GitHubIssuesLoaded { result } => reduce_github_issues_loaded(state, result),
        AppEvent::GitHubPrsLoaded { result } => reduce_github_prs_loaded(state, result),
        AppEvent::GitHubIssueDetailLoaded { number, result } => {
            reduce_github_issue_detail_loaded(state, number, result)
        }
        AppEvent::GitHubPrDetailLoaded { number, result } => {
            reduce_github_pr_detail_loaded(state, number, result)
        }
        AppEvent::GitHubIssueCommentCompleted { number, result } => {
            reduce_github_issue_comment_completed(state, number, result)
        }
        AppEvent::GitHubIssueCreateBranchCompleted { number, result } => {
            reduce_github_issue_create_branch_completed(state, number, result)
        }
        AppEvent::GitHubRepoLabelsLoaded { result } => {
            reduce_github_repo_labels_loaded(state, result)
        }
        AppEvent::GitHubIssueLabelsApplied { number, result } => {
            reduce_github_issue_labels_applied(state, number, result)
        }
        AppEvent::GitHubOpenBrowserCompleted { target, result } => {
            reduce_github_open_browser_completed(state, target, result)
        }
        AppEvent::GitHubPrCreateCompleted { result } => {
            reduce_github_pr_create_completed(state, result)
        }
        AppEvent::BranchListLoaded { entries, error } => {
            reduce_branch_list_loaded(state, entries, error)
        }
        AppEvent::BranchCheckoutCompleted { branch_name, error } => {
            reduce_branch_checkout_completed(state, branch_name, error)
        }
    }
}

fn reduce_command(state: &mut AppState, command: AppCommand) -> Vec<SideEffect> {
    match command {
        AppCommand::Navigation(nav) => {
            apply_navigation(state, nav);
            let mut effects = agent_spawn_effects_if_needed(state);
            effects.extend(github_first_access_effects(state));
            effects.extend(github_issue_list_access_effects(state, false));
            effects.extend(github_pr_list_access_effects(state, false));
            effects.extend(github_issue_detail_access_effects(state, false));
            effects.extend(github_pr_detail_access_effects(state, false));
            effects.extend(branch_list_access_effects(state, false));
            effects
        }
        AppCommand::Quit => {
            state.dirty = true;
            vec![SideEffect::Quit]
        }
        AppCommand::RequestGitRefresh => reduce_git_refresh_requested(state),
        AppCommand::GitHubRefresh => reduce_github_refresh_requested(state),
        AppCommand::GitHubMoveIssueSelection(delta) => {
            reduce_github_move_issue_selection(state, delta)
        }
        AppCommand::GitHubMovePrSelection(delta) => reduce_github_move_pr_selection(state, delta),
        AppCommand::GitHubSelectIssue(index) => reduce_github_select_issue(state, index),
        AppCommand::GitHubSelectPr(index) => reduce_github_select_pr(state, index),
        AppCommand::GitHubOpenSelected => reduce_github_open_selected(state),
        AppCommand::GitHubSelectLeftPane(pane) => reduce_github_select_left_pane(state, pane),
        AppCommand::GitHubIssueDetailScroll(delta) => {
            reduce_github_issue_detail_scroll(state, delta)
        }
        AppCommand::GitHubIssueDetailPageScroll(delta) => {
            reduce_github_issue_detail_page_scroll(state, delta)
        }
        AppCommand::GitHubPrDetailScroll(delta) => reduce_github_pr_detail_scroll(state, delta),
        AppCommand::GitHubPrDetailPageScroll(delta) => {
            reduce_github_pr_detail_page_scroll(state, delta)
        }
        AppCommand::GitHubLabelPickerMove(delta) => reduce_github_label_picker_move(state, delta),
        AppCommand::GitHubLabelPickerToggle => reduce_github_label_picker_toggle(state),
        AppCommand::GitHubLabelPickerApply => reduce_github_label_picker_apply(state),
        AppCommand::GitHubLabelPickerCancel => reduce_github_label_picker_cancel(state),
        AppCommand::GitHubOpenInBrowser => reduce_github_open_in_browser(state),
        AppCommand::ShellWrite(data) => vec![SideEffect::WriteShell(data)],
        AppCommand::ShellScroll(delta) => reduce_shell_scroll(state, delta),
        AppCommand::ShellScrollLines(lines) => reduce_shell_scroll_lines(state, lines),
        AppCommand::AgentWrite(data) => vec![SideEffect::WriteAgent(data)],
        AppCommand::AgentScroll(delta) => reduce_agent_scroll(state, delta),
        AppCommand::AgentScrollLines(lines) => reduce_agent_scroll_lines(state, lines),
        AppCommand::AgentRestart => reduce_agent_restart(state),
        AppCommand::PaletteOpen => reduce_palette_open(state),
        AppCommand::PaletteClose => reduce_palette_close(state),
        AppCommand::PaletteAppendChar(ch) => reduce_palette_append_char(state, ch),
        AppCommand::PaletteBackspace => reduce_palette_backspace(state),
        AppCommand::PaletteMoveSelection(delta) => reduce_palette_move_selection(state, delta),
        AppCommand::PaletteHistoryUp => reduce_palette_history_up(state),
        AppCommand::PaletteHistoryDown => reduce_palette_history_down(state),
        AppCommand::PaletteExecuteSelected => reduce_palette_execute_selected(state),
        AppCommand::PaletteExecuteMatch(index) => reduce_palette_execute_match(state, index),
        AppCommand::FileTreeExpand(path) => reduce_file_tree_expand(state, path),
        AppCommand::FileTreeCollapse(path) => reduce_file_tree_collapse(state, path),
        AppCommand::FileTreeSelect(path) => reduce_file_tree_select(state, path),
        AppCommand::FileTreeRefresh => reduce_file_tree_refresh(state),
        AppCommand::FileTreeMoveSelection(delta) => reduce_file_tree_move_selection(state, delta),
        AppCommand::GitMoveSelection(delta) => reduce_git_move_selection(state, delta),
        AppCommand::GitSelect(index) => reduce_git_select(state, index),
        AppCommand::GitOpenSelected => reduce_git_open_selected(state),
        AppCommand::GitRefresh => reduce_git_refresh_requested(state),
        AppCommand::BranchMoveSelection(delta) => reduce_branch_move_selection(state, delta),
        AppCommand::BranchSelect(index) => reduce_branch_select(state, index),
        AppCommand::BranchCheckoutSelected => reduce_branch_checkout_selected(state),
        AppCommand::BranchRefresh => reduce_branch_refresh(state),
        AppCommand::PreviewFile { path, line } => reduce_preview_file(state, path, line),
        AppCommand::PreviewScroll(delta) => reduce_preview_scroll(state, delta),
        AppCommand::PreviewPageScroll(delta) => reduce_preview_page_scroll(state, delta),
        AppCommand::DiffScroll(delta) => reduce_diff_scroll(state, delta),
        AppCommand::DiffPageScroll(delta) => reduce_diff_page_scroll(state, delta),
        AppCommand::DiffHorizontalScroll(delta) => reduce_diff_horizontal_scroll(state, delta),
        AppCommand::DiffToggleSource => reduce_diff_toggle_source(state),
        AppCommand::DiffSetSource(source) => reduce_diff_set_source(state, source),
        AppCommand::DiffNextFile => reduce_diff_move_file(state, 1),
        AppCommand::DiffPrevFile => reduce_diff_move_file(state, -1),
        #[cfg_attr(not(test), allow(dead_code))]
        AppCommand::DiffSelectFile(path) => reduce_diff_select_file(state, path),
        AppCommand::SearchSetQuery(query) => reduce_search_set_query(state, query),
        AppCommand::SearchAppendChar(ch) => reduce_search_append_char(state, ch),
        AppCommand::SearchBackspace => reduce_search_backspace(state),
        AppCommand::SearchClear => reduce_search_clear(state),
        AppCommand::SearchSetMode(mode) => reduce_search_set_mode(state, mode),
        AppCommand::SearchExecute => reduce_search_execute(state),
        AppCommand::SearchCancel => reduce_search_cancel(state),
        AppCommand::SearchMoveSelection(delta) => reduce_search_move_selection(state, delta),
        AppCommand::SearchSelect(index) => reduce_search_select(state, index),
        AppCommand::OpenEditor { path, line } => reduce_open_editor(state, path, line),
        AppCommand::ModalDismiss => reduce_modal_dismiss(state),
        AppCommand::ClipboardCopy => reduce_clipboard_copy(state),
        AppCommand::ClipboardCut => reduce_clipboard_cut(state),
        AppCommand::ClipboardPaste => reduce_clipboard_paste(state),
        AppCommand::PasteText(text) => reduce_paste_text(state, text),
        AppCommand::SelectionBegin { pane, line, col } => {
            reduce_selection_begin(state, pane, line, col)
        }
        AppCommand::SelectionExtend { line, col } => reduce_selection_extend(state, line, col),
        AppCommand::SelectionEnd => reduce_selection_end(state),
        AppCommand::SelectionClear => reduce_selection_clear(state),
    }
}

pub fn apply_navigation(state: &mut AppState, command: NavCommand) {
    let before = state.navigation.clone();
    state.navigation.apply(command);
    if state.navigation != before {
        state.dirty = true;
    }
    if state.navigation.left_tab == LeftNavTab::Gh {
        sync_github_left_pane_from_main_tab(state);
    }
    if state.navigation.left_tab == LeftNavTab::Files {
        state.file_tree.ensure_selection();
    }
    if state.navigation.left_tab == LeftNavTab::Git {
        ensure_git_selection(&mut state.git, state.config.git.show_untracked);
    }
    if state.navigation.main_tab == MainTab::Branches {
        ensure_branch_selection(&mut state.branches);
    }
}

fn sync_github_left_pane_from_main_tab(state: &mut AppState) {
    state.github.left_pane = match state.navigation.main_tab {
        MainTab::Issues => GitHubLeftPane::Issues,
        MainTab::Prs => GitHubLeftPane::Prs,
        _ => state.github.left_pane,
    };
}

pub fn agent_spawn_effects_if_needed(state: &mut AppState) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent || state.agent.spawned {
        return Vec::new();
    }

    state.dirty = true;
    vec![SideEffect::SpawnAgent]
}

fn reduce_terminal_resize(state: &mut AppState, width: u16, height: u16) -> Vec<SideEffect> {
    let Ok(layout) = compute_layout(width, height, state.config.app.left_width) else {
        return Vec::new();
    };

    if state.layout == layout {
        return Vec::new();
    }

    state.layout = layout;
    state.dirty = true;

    if !state.shell.running {
        return Vec::new();
    }

    let (cols, rows) = shell_pty_size(&state.layout.rects);
    if cols == state.shell.cols && rows == state.shell.rows {
        return Vec::new();
    }

    state.shell.apply_resize(cols, rows);
    vec![SideEffect::ResizeShell { cols, rows }]
}

pub fn git_refresh_effects(state: &mut AppState) -> Vec<SideEffect> {
    state.dirty = true;
    if !state.workspace_meta.is_git_repo {
        return Vec::new();
    }

    state.git.loading = true;
    vec![SideEffect::SpawnGitRefresh]
}

pub fn branch_list_access_effects(state: &mut AppState, force: bool) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Branches && !force {
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
    state.dirty = true;
    vec![SideEffect::SpawnBranchList]
}

pub fn branch_checkout_effects(state: &mut AppState, branch_name: String) -> Vec<SideEffect> {
    if !state.workspace_meta.is_git_repo || state.branches.checkout_loading {
        return Vec::new();
    }

    if state.git.branch.as_deref() == Some(branch_name.as_str()) {
        return Vec::new();
    }

    state.branches.checkout_loading = true;
    state.branches.checkout_error = None;
    state.dirty = true;
    vec![SideEffect::SpawnBranchCheckout { name: branch_name }]
}

pub fn github_refresh_effects(state: &mut AppState) -> Vec<SideEffect> {
    state.dirty = true;
    state.github.loading = true;
    state.github.auth_checked = false;
    state.github.issues_loaded_at = None;
    state.github.prs_loaded_at = None;
    clear_issue_detail_cache(&mut state.github);
    clear_pr_detail_cache(&mut state.github);
    vec![SideEffect::SpawnGitHubAuthCheck]
}

pub fn github_first_access_effects(state: &mut AppState) -> Vec<SideEffect> {
    if !github_surface_active(state) {
        return Vec::new();
    }

    if state.github.auth_checked || state.github.loading {
        return Vec::new();
    }

    state.github.loading = true;
    state.dirty = true;
    vec![SideEffect::SpawnGitHubAuthCheck]
}

pub fn github_issue_list_effects(state: &mut AppState, force: bool) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        return Vec::new();
    }

    if !github_issue_list_surface_active(state) {
        return Vec::new();
    }

    if state.github.issues_loading {
        return Vec::new();
    }

    if !force && issue_list_cache_fresh(&state.github) {
        return Vec::new();
    }

    state.github.issues_loading = true;
    state.github.issues_error = None;
    state.dirty = true;
    vec![SideEffect::SpawnGitHubIssueList]
}

pub fn github_issue_list_access_effects(state: &mut AppState, force: bool) -> Vec<SideEffect> {
    github_issue_list_effects(state, force)
}

pub fn github_pr_list_effects(state: &mut AppState, force: bool) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        return Vec::new();
    }

    if !github_pr_list_surface_active(state) {
        return Vec::new();
    }

    if state.github.prs_loading {
        return Vec::new();
    }

    if !force && pr_list_cache_fresh(&state.github) {
        return Vec::new();
    }

    state.github.prs_loading = true;
    state.github.prs_error = None;
    state.dirty = true;
    vec![SideEffect::SpawnGitHubPrList]
}

pub fn github_pr_list_access_effects(state: &mut AppState, force: bool) -> Vec<SideEffect> {
    github_pr_list_effects(state, force)
}

pub fn github_issue_detail_access_effects(state: &mut AppState, force: bool) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Issues {
        return Vec::new();
    }

    let Some(number) = selected_issue_number(&state.github) else {
        return Vec::new();
    };

    github_issue_detail_effects(state, number, force)
}

fn clear_issue_detail_cache(github: &mut crate::state::GitHubState) {
    github.issue_detail_number = None;
    github.issue_detail = None;
    github.issue_detail_loading = false;
    github.issue_detail_error = None;
    github.issue_detail_scroll_offset = 0;
}

fn clear_pr_detail_cache(github: &mut crate::state::GitHubState) {
    github.pr_detail_number = None;
    github.pr_detail = None;
    github.pr_detail_loading = false;
    github.pr_detail_error = None;
    github.pr_detail_scroll_offset = 0;
}

pub fn github_pr_detail_access_effects(state: &mut AppState, force: bool) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Prs {
        return Vec::new();
    }

    let Some(number) = selected_pr_number(&state.github) else {
        return Vec::new();
    };

    github_pr_detail_effects(state, number, force)
}

fn selected_pr_number(github: &crate::state::GitHubState) -> Option<u32> {
    github
        .selected_pr
        .and_then(|number| u32::try_from(number).ok())
}

fn github_pr_detail_effects(state: &mut AppState, number: u32, force: bool) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        return Vec::new();
    }

    if state.github.pr_detail_loading && state.github.pr_detail_number == Some(u64::from(number)) {
        return Vec::new();
    }

    if !force
        && state.github.pr_detail_number == Some(u64::from(number))
        && state.github.pr_detail.is_some()
        && state.github.pr_detail_error.is_none()
    {
        return Vec::new();
    }

    state.github.pr_detail_loading = true;
    state.github.pr_detail_error = None;
    if state.github.pr_detail_number != Some(u64::from(number)) {
        state.github.pr_detail = None;
        state.github.pr_detail_scroll_offset = 0;
    }
    state.github.pr_detail_number = Some(u64::from(number));
    state.dirty = true;

    vec![SideEffect::SpawnGitHubPrDetail { number }]
}

fn selected_issue_number(github: &crate::state::GitHubState) -> Option<u32> {
    github
        .selected_issue
        .and_then(|number| u32::try_from(number).ok())
}

fn github_issue_detail_effects(state: &mut AppState, number: u32, force: bool) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        return Vec::new();
    }

    if state.github.issue_detail_loading
        && state.github.issue_detail_number == Some(u64::from(number))
    {
        return Vec::new();
    }

    if !force
        && state.github.issue_detail_number == Some(u64::from(number))
        && state.github.issue_detail.is_some()
        && state.github.issue_detail_error.is_none()
    {
        return Vec::new();
    }

    state.github.issue_detail_loading = true;
    state.github.issue_detail_error = None;
    if state.github.issue_detail_number != Some(u64::from(number)) {
        state.github.issue_detail = None;
        state.github.issue_detail_scroll_offset = 0;
    }
    state.github.issue_detail_number = Some(u64::from(number));
    state.dirty = true;

    vec![SideEffect::SpawnGitHubIssueDetail { number }]
}

fn github_surface_active(state: &AppState) -> bool {
    state.navigation.left_tab == LeftNavTab::Gh
        || matches!(state.navigation.main_tab, MainTab::Issues | MainTab::Prs)
}

fn github_issue_list_surface_active(state: &AppState) -> bool {
    state.navigation.main_tab == MainTab::Issues || state.navigation.left_tab == LeftNavTab::Gh
}

fn issue_list_cache_fresh(github: &crate::state::GitHubState) -> bool {
    let Some(loaded_at) = github.issues_loaded_at else {
        return false;
    };

    loaded_at
        .elapsed()
        .map(|elapsed| elapsed.as_secs() < ISSUE_LIST_CACHE_SECS)
        .unwrap_or(false)
}

fn github_pr_list_surface_active(state: &AppState) -> bool {
    state.navigation.main_tab == MainTab::Prs || state.navigation.left_tab == LeftNavTab::Gh
}

fn pr_list_cache_fresh(github: &crate::state::GitHubState) -> bool {
    let Some(loaded_at) = github.prs_loaded_at else {
        return false;
    };

    loaded_at
        .elapsed()
        .map(|elapsed| elapsed.as_secs() < PR_LIST_CACHE_SECS)
        .unwrap_or(false)
}

fn reduce_github_refresh_requested(state: &mut AppState) -> Vec<SideEffect> {
    github_refresh_effects(state)
}

fn reduce_github_auth_checked(
    state: &mut AppState,
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
    state.dirty = true;

    if result.auth_ok {
        let mut effects = github_issue_list_effects(state, true);
        effects.extend(github_pr_list_effects(state, true));
        effects
    } else {
        Vec::new()
    }
}

fn reduce_github_issues_loaded(
    state: &mut AppState,
    result: IssueListLoadResult,
) -> Vec<SideEffect> {
    state.github.issues_loading = false;
    state.github.issues = result.issues;
    state.github.issues_error = result.error;
    state.github.issues_loaded_at = Some(SystemTime::now());
    ensure_issue_selection(&mut state.github);
    state.dirty = true;

    if state.navigation.main_tab == MainTab::Issues {
        if let Some(number) = selected_issue_number(&state.github) {
            return github_issue_detail_effects(state, number, true);
        }
    }

    if state.navigation.main_tab == MainTab::Prs {
        if let Some(number) = selected_pr_number(&state.github) {
            return github_pr_detail_effects(state, number, true);
        }
    }

    Vec::new()
}

fn reduce_github_prs_loaded(state: &mut AppState, result: PrListLoadResult) -> Vec<SideEffect> {
    state.github.prs_loading = false;
    state.github.prs = result.prs;
    state.github.prs_error = result.error;
    state.github.prs_loaded_at = Some(SystemTime::now());
    ensure_pr_selection(&mut state.github);
    state.dirty = true;

    if state.navigation.main_tab == MainTab::Prs {
        if let Some(number) = selected_pr_number(&state.github) {
            return github_pr_detail_effects(state, number, true);
        }
    }

    Vec::new()
}

fn reduce_github_issue_detail_loaded(
    state: &mut AppState,
    number: u32,
    result: IssueDetailLoadResult,
) -> Vec<SideEffect> {
    if state.github.issue_detail_number != Some(u64::from(number)) {
        return Vec::new();
    }

    state.github.issue_detail_loading = false;
    state.github.issue_detail_error = result.error;
    state.github.issue_detail = result.detail;
    if state.github.issue_detail.is_some() {
        state.text_selection.clear();
        clamp_issue_detail_scroll(state);
    }
    state.dirty = true;
    Vec::new()
}

fn reduce_github_pr_detail_loaded(
    state: &mut AppState,
    number: u32,
    result: PrDetailLoadResult,
) -> Vec<SideEffect> {
    if state.github.pr_detail_number != Some(u64::from(number)) {
        return Vec::new();
    }

    state.github.pr_detail_loading = false;
    state.github.pr_detail_error = result.error;
    state.github.pr_detail = result.detail;
    if state.github.pr_detail.is_some() {
        state.text_selection.clear();
        clamp_pr_detail_scroll(state);
    }
    state.dirty = true;
    Vec::new()
}

fn reduce_github_move_issue_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = issues_viewport_rows(state);
    issue_move_selection(&mut state.github, delta, viewport_rows);
    state.dirty = true;
    Vec::new()
}

fn reduce_github_select_issue(state: &mut AppState, row_index: usize) -> Vec<SideEffect> {
    let viewport_rows = issues_viewport_rows(state);
    issue_select_row(&mut state.github, row_index, viewport_rows);
    state.dirty = true;
    Vec::new()
}

fn issues_viewport_rows(state: &AppState) -> usize {
    crate::ui::issues_viewport_rows(state.layout.rects.left_content)
}

fn reduce_github_move_pr_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = prs_viewport_rows(state);
    pr_move_selection(&mut state.github, delta, viewport_rows);
    state.dirty = true;
    Vec::new()
}

fn reduce_github_select_pr(state: &mut AppState, row_index: usize) -> Vec<SideEffect> {
    let viewport_rows = prs_viewport_rows(state);
    pr_select_row(&mut state.github, row_index, viewport_rows);
    state.dirty = true;
    Vec::new()
}

fn prs_viewport_rows(state: &AppState) -> usize {
    crate::ui::prs_viewport_rows(state.layout.rects.left_content)
}

fn reduce_github_open_selected(state: &mut AppState) -> Vec<SideEffect> {
    match state.github.left_pane {
        GitHubLeftPane::Issues => {
            let Some(number) = selected_issue_number(&state.github) else {
                return Vec::new();
            };

            state.github.left_pane = GitHubLeftPane::Issues;
            state
                .navigation
                .apply(NavCommand::SelectMainTab(MainTab::Issues));
            state
                .navigation
                .apply(NavCommand::SetFocus(FocusTarget::Main));
            state.dirty = true;
            github_issue_detail_effects(state, number, true)
        }
        GitHubLeftPane::Prs => {
            let Some(number) = selected_pr_number(&state.github) else {
                return Vec::new();
            };

            state.github.left_pane = GitHubLeftPane::Prs;
            state
                .navigation
                .apply(NavCommand::SelectMainTab(MainTab::Prs));
            state
                .navigation
                .apply(NavCommand::SetFocus(FocusTarget::Main));
            state.dirty = true;
            github_pr_detail_effects(state, number, true)
        }
    }
}

fn reduce_github_select_left_pane(state: &mut AppState, pane: GitHubLeftPane) -> Vec<SideEffect> {
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
    state.dirty = true;
    match pane {
        GitHubLeftPane::Issues => github_issue_list_effects(state, false),
        GitHubLeftPane::Prs => github_pr_list_effects(state, false),
    }
}

fn reduce_github_issue_detail_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    let line_count = issue_detail_line_count(&state.github);
    let viewport_rows = issue_detail_viewport_rows(state);
    scroll_issue_detail(
        &mut state.github.issue_detail_scroll_offset,
        delta,
        line_count,
        viewport_rows,
    );
    state.dirty = true;
    Vec::new()
}

fn reduce_github_issue_detail_page_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    let line_count = issue_detail_line_count(&state.github);
    let viewport_rows = issue_detail_viewport_rows(state);
    page_scroll_issue_detail(
        &mut state.github.issue_detail_scroll_offset,
        delta,
        line_count,
        viewport_rows,
    );
    state.dirty = true;
    Vec::new()
}

fn issue_detail_line_count(github: &crate::state::GitHubState) -> usize {
    github
        .issue_detail
        .as_ref()
        .map(|detail| detail.display_lines.len())
        .unwrap_or(0)
}

fn clamp_issue_detail_scroll(state: &mut AppState) {
    let line_count = issue_detail_line_count(&state.github);
    let viewport_rows = issue_detail_viewport_rows(state);
    let max_offset = line_count.saturating_sub(viewport_rows);
    if state.github.issue_detail_scroll_offset > max_offset {
        state.github.issue_detail_scroll_offset = max_offset;
    }
}

fn issue_detail_viewport_rows(state: &AppState) -> usize {
    crate::ui::issue_detail_viewport_rows(state.layout.rects.main_content)
}

fn reduce_github_pr_detail_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    let line_count = pr_detail_line_count(&state.github);
    let viewport_rows = pr_detail_viewport_rows(state);
    scroll_issue_detail(
        &mut state.github.pr_detail_scroll_offset,
        delta,
        line_count,
        viewport_rows,
    );
    state.dirty = true;
    Vec::new()
}

fn reduce_github_pr_detail_page_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    let line_count = pr_detail_line_count(&state.github);
    let viewport_rows = pr_detail_viewport_rows(state);
    page_scroll_issue_detail(
        &mut state.github.pr_detail_scroll_offset,
        delta,
        line_count,
        viewport_rows,
    );
    state.dirty = true;
    Vec::new()
}

fn pr_detail_line_count(github: &crate::state::GitHubState) -> usize {
    github
        .pr_detail
        .as_ref()
        .map(|detail| detail.display_lines.len())
        .unwrap_or(0)
}

fn clamp_pr_detail_scroll(state: &mut AppState) {
    let line_count = pr_detail_line_count(&state.github);
    let viewport_rows = pr_detail_viewport_rows(state);
    let max_offset = line_count.saturating_sub(viewport_rows);
    if state.github.pr_detail_scroll_offset > max_offset {
        state.github.pr_detail_scroll_offset = max_offset;
    }
}

fn pr_detail_viewport_rows(state: &AppState) -> usize {
    crate::ui::pr_detail_viewport_rows(state.layout.rects.main_content)
}

fn reduce_github_issue_comment_completed(
    state: &mut AppState,
    number: u32,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if state.github.selected_issue != Some(u64::from(number)) {
        return Vec::new();
    }

    if result.success {
        state.github.issue_action_message = Some(format!("Comment posted on #{number}"));
        state.github.issues_loaded_at = None;
        clear_issue_detail_cache(&mut state.github);
        let mut effects = github_issue_list_effects(state, true);
        effects.extend(github_issue_detail_effects(state, number, true));
        state.dirty = true;
        return effects;
    }

    state.github.issue_action_message = result.error.clone();
    state.dirty = true;
    Vec::new()
}

fn reduce_github_issue_create_branch_completed(
    state: &mut AppState,
    number: u32,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if state.github.selected_issue != Some(u64::from(number)) {
        return Vec::new();
    }

    if result.success {
        state.github.issue_action_message = result
            .detail
            .or_else(|| Some(format!("Checked out branch for issue #{number}")));
        state.dirty = true;
        return git_refresh_effects(state);
    }

    state.github.issue_action_message = result.error.clone();
    state.dirty = true;
    Vec::new()
}

fn reduce_github_repo_labels_loaded(
    state: &mut AppState,
    result: crate::github::RepoLabelsLoadResult,
) -> Vec<SideEffect> {
    let Some(picker) = state.github.label_picker.as_mut() else {
        return Vec::new();
    };

    let existing = picker.existing_labels.clone();
    apply_label_picker_load(picker, result, &existing);
    state.dirty = true;
    Vec::new()
}

fn reduce_github_issue_labels_applied(
    state: &mut AppState,
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
        clear_issue_detail_cache(&mut state.github);
        let mut effects = github_issue_list_effects(state, true);
        effects.extend(github_issue_detail_effects(state, number, true));
        state.dirty = true;
        return effects;
    }

    picker.error = result.error.clone();
    state.dirty = true;
    Vec::new()
}

fn reduce_github_label_picker_move(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if let Some(picker) = state.github.label_picker.as_mut() {
        picker.move_cursor(delta);
        state.dirty = true;
    }
    Vec::new()
}

fn reduce_github_label_picker_toggle(state: &mut AppState) -> Vec<SideEffect> {
    if let Some(picker) = state.github.label_picker.as_mut() {
        if !picker.loading && !picker.applying {
            picker.toggle_cursor();
            state.dirty = true;
        }
    }
    Vec::new()
}

fn reduce_github_label_picker_apply(state: &mut AppState) -> Vec<SideEffect> {
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
        state.dirty = true;
        return Vec::new();
    }

    let number = picker.issue_number;
    picker.applying = true;
    picker.error = None;
    state.dirty = true;
    vec![SideEffect::SpawnGitHubIssueLabelApply { number, labels }]
}

fn reduce_github_label_picker_cancel(state: &mut AppState) -> Vec<SideEffect> {
    if state.github.label_picker.is_some() {
        state.github.label_picker = None;
        state.dirty = true;
    }
    Vec::new()
}

fn reduce_github_open_in_browser(state: &mut AppState) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.dirty = true;
        return Vec::new();
    }

    let Some(target) = resolve_browser_target(state) else {
        state
            .notifications
            .show_toast(missing_browser_target_message(state));
        state.dirty = true;
        return Vec::new();
    };

    state.dirty = true;
    vec![SideEffect::SpawnGitHubOpenBrowser { target }]
}

fn reduce_github_open_browser_completed(
    state: &mut AppState,
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
    state.dirty = true;
    Vec::new()
}

fn reduce_github_pr_create_completed(
    state: &mut AppState,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if result.success {
        state.github.issue_action_message = result
            .detail
            .or_else(|| Some("Pull request created".to_string()));
        state.github.prs_loaded_at = None;
        clear_pr_detail_cache(&mut state.github);
        state.dirty = true;
        return github_pr_list_effects(state, true);
    }

    if let Some(error) = result.error {
        state.github.issue_action_message = Some(error);
    }
    state.dirty = true;
    Vec::new()
}

fn reduce_git_refresh_requested(state: &mut AppState) -> Vec<SideEffect> {
    git_refresh_effects(state)
}

fn reduce_git_status_updated(
    state: &mut AppState,
    branch: Option<String>,
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
        let file_patch = patch_git_file_entries(&mut state.git.file_entries, &file_entries);
        sync_git_status_patch_to_file_tree(state, &file_patch);
        ensure_git_selection(&mut state.git, state.config.git.show_untracked);
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

    state.dirty = true;
    Vec::new()
}

fn reduce_branch_refresh(state: &mut AppState) -> Vec<SideEffect> {
    branch_list_access_effects(state, true)
}

fn reduce_branch_move_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = crate::ui::branches_viewport_rows(state.layout.rects.main_content);
    branch_move_selection(&mut state.branches, delta, viewport_rows);
    state.dirty = true;
    Vec::new()
}

fn reduce_branch_select(state: &mut AppState, index: usize) -> Vec<SideEffect> {
    let viewport_rows = crate::ui::branches_viewport_rows(state.layout.rects.main_content);
    branch_select_row(&mut state.branches, index, viewport_rows);
    state.dirty = true;
    Vec::new()
}

fn reduce_branch_checkout_selected(state: &mut AppState) -> Vec<SideEffect> {
    let Some(branch_name) = branch_selected_name(&state.branches).map(str::to_string) else {
        return Vec::new();
    };

    branch_checkout_effects(state, branch_name)
}

fn reduce_branch_list_loaded(
    state: &mut AppState,
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
            state.dirty = true;
            return Vec::new();
        }
    }

    ensure_branch_selection(&mut state.branches);
    state.dirty = true;
    Vec::new()
}

fn reduce_branch_checkout_completed(
    state: &mut AppState,
    branch_name: String,
    error: Option<String>,
) -> Vec<SideEffect> {
    state.branches.checkout_loading = false;

    if let Some(message) = error {
        state.branches.checkout_error = Some(message.clone());
        state
            .logs
            .push_error(format!("checkout {branch_name} failed: {message}"));
        state.dirty = true;
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
        ensure_branch_selection(&mut state.branches);
    }
    state.dirty = true;
    effects
}

fn sync_git_status_patch_to_file_tree(
    state: &mut AppState,
    patch: &crate::git::GitFileStatusPatch,
) {
    if patch.is_empty() {
        return;
    }

    state.file_tree.apply_git_status_patch(
        &state.repo_root,
        patch,
        state.config.git.show_untracked,
    );
}

fn sync_git_statuses_to_file_tree(state: &mut AppState) {
    let entries = state.git.file_entries.clone();
    state
        .file_tree
        .apply_git_statuses(&state.repo_root, &entries, state.config.git.show_untracked);
}

fn reduce_shell_output(state: &mut AppState, data: Vec<u8>) -> Vec<SideEffect> {
    state.shell.scrollback.set_cols(state.shell.cols);
    state.shell.scrollback.append_bytes(&data);
    state.dirty = true;
    Vec::new()
}

fn reduce_shell_exited(state: &mut AppState) -> Vec<SideEffect> {
    state.shell.running = false;
    state.dirty = true;
    Vec::new()
}

fn reduce_agent_output(state: &mut AppState, data: Vec<u8>) -> Vec<SideEffect> {
    state.agent.scrollback.set_cols(state.agent.cols);
    state.agent.scrollback.append_bytes(&data);
    if let Some(status) = infer_status_from_scrollback(&state.agent.scrollback) {
        state.agent.status = status;
    }
    state.dirty = true;
    Vec::new()
}

fn reduce_agent_exited(state: &mut AppState, code: i32) -> Vec<SideEffect> {
    state.agent.apply_exit(code);
    state.dirty = true;
    Vec::new()
}

fn reduce_agent_restart(state: &mut AppState) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    state.dirty = true;
    vec![SideEffect::RestartAgent]
}

fn reduce_shell_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let (_, page_size) = shell_pty_size(&state.layout.rects);
    state.shell.scroll_by(delta, page_size);
    state.dirty = true;
    Vec::new()
}

fn reduce_shell_scroll_lines(state: &mut AppState, lines: i32) -> Vec<SideEffect> {
    if lines == 0 {
        return Vec::new();
    }

    let (_, page_size) = shell_pty_size(&state.layout.rects);
    state.shell.scroll_by_lines(lines, page_size);
    state.dirty = true;
    Vec::new()
}

fn reduce_agent_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let (_, page_size) = agent_pty_size(&state.layout.rects);
    state.agent.scroll_by(delta, page_size);
    state.dirty = true;
    Vec::new()
}

fn reduce_agent_scroll_lines(state: &mut AppState, lines: i32) -> Vec<SideEffect> {
    if lines == 0 {
        return Vec::new();
    }

    let (_, page_size) = agent_pty_size(&state.layout.rects);
    state.agent.scroll_by_lines(lines, page_size);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_open(state: &mut AppState) -> Vec<SideEffect> {
    state.palette.open_with_focus(state.navigation.focus);
    state.navigation.focus = FocusTarget::CommandPalette;
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_close(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.close(&mut state.navigation.focus);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_append_char(state: &mut AppState, ch: char) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_cursor = None;
    state.palette.input.push(ch);
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_backspace(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_cursor = None;
    state.palette.input.pop();
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_move_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if !state.palette.open || state.palette.prompt.is_some() {
        return Vec::new();
    }

    state.palette.move_selection(delta as isize);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_history_up(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    if let Some(command_id) = state.palette.history_up() {
        state.palette.input = history_input_for_id(&command_id);
    }
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_history_down(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    match state.palette.history_down() {
        None => {}
        Some(None) => state.palette.input.clear(),
        Some(Some(command_id)) => state.palette.input = history_input_for_id(&command_id),
    }
    refresh_matches(state);
    state.dirty = true;
    Vec::new()
}

fn reduce_palette_execute_selected(state: &mut AppState) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    if let Some(prompt) = state.palette.prompt.clone() {
        return reduce_palette_prompt_submit(state, prompt);
    }

    let Some(registry_index) = state.palette.matches.get(state.palette.selected).copied() else {
        return Vec::new();
    };

    execute_command(state, registry_index)
}

fn reduce_palette_prompt_submit(state: &mut AppState, prompt: PalettePrompt) -> Vec<SideEffect> {
    match prompt {
        PalettePrompt::GitHubIssueComment { number } => {
            let body = state.palette.input.trim().to_string();
            if body.is_empty() {
                state.notifications.show_toast("Comment cannot be empty");
                state.dirty = true;
                return Vec::new();
            }

            state.palette.close(&mut state.navigation.focus);
            state.dirty = true;
            vec![SideEffect::SpawnGitHubIssueComment { number, body }]
        }
        PalettePrompt::GitHubPrCreate(prompt) => {
            let input = state.palette.input.clone();
            match advance_pr_create_prompt(prompt, &input) {
                Ok(PrCreatePromptAdvance::Continue(next)) => {
                    state.palette.input.clear();
                    state.palette.prompt = Some(PalettePrompt::GitHubPrCreate(next));
                    state.dirty = true;
                    Vec::new()
                }
                Ok(PrCreatePromptAdvance::Submit(request)) => {
                    state.palette.close(&mut state.navigation.focus);
                    state.dirty = true;
                    vec![SideEffect::SpawnGitHubPrCreate { request }]
                }
                Err(message) => {
                    state.notifications.show_toast(message);
                    state.dirty = true;
                    Vec::new()
                }
            }
        }
    }
}

fn reduce_palette_execute_match(state: &mut AppState, match_index: usize) -> Vec<SideEffect> {
    if !state.palette.open || state.palette.prompt.is_some() {
        return Vec::new();
    }

    let Some(registry_index) = state.palette.matches.get(match_index).copied() else {
        return Vec::new();
    };

    execute_command(state, registry_index)
}

fn reduce_file_tree_expand(state: &mut AppState, path: PathBuf) -> Vec<SideEffect> {
    match state.file_tree.expand(&path) {
        Ok(ExpandAction::NeedsLoad) => {
            state.dirty = true;
            vec![SideEffect::LoadDirectoryChildren(path)]
        }
        Ok(ExpandAction::AlreadyExpanded) => {
            state.dirty = true;
            Vec::new()
        }
        Err(_) => {
            state.dirty = true;
            Vec::new()
        }
    }
}

fn reduce_file_tree_collapse(state: &mut AppState, path: PathBuf) -> Vec<SideEffect> {
    state.file_tree.collapse(&path);
    state.dirty = true;
    Vec::new()
}

fn reduce_file_tree_select(state: &mut AppState, path: PathBuf) -> Vec<SideEffect> {
    state.file_tree.select(path);
    state.dirty = true;
    Vec::new()
}

fn file_tree_viewport_rows(state: &AppState) -> usize {
    state.layout.rects.left_content.height.saturating_sub(2) as usize
}

fn reduce_file_tree_move_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    state
        .file_tree
        .move_selection(delta, file_tree_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn git_viewport_rows(state: &AppState) -> usize {
    crate::ui::git_viewport_rows(state.layout.rects.left_content)
}

fn reduce_git_move_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let viewport_rows = git_viewport_rows(state);
    let show_untracked = state.config.git.show_untracked;
    git_move_selection(&mut state.git, delta, viewport_rows, show_untracked);
    state.dirty = true;
    Vec::new()
}

fn reduce_git_select(state: &mut AppState, index: usize) -> Vec<SideEffect> {
    let viewport_rows = git_viewport_rows(state);
    let show_untracked = state.config.git.show_untracked;
    git_select_row(&mut state.git, index, viewport_rows, show_untracked);
    state.dirty = true;
    Vec::new()
}

fn reduce_git_open_selected(state: &mut AppState) -> Vec<SideEffect> {
    let Some(path) = state.git.selected_path.clone() else {
        return Vec::new();
    };

    apply_navigation(state, NavCommand::SelectMainTab(MainTab::Diff));
    state
        .navigation
        .apply(NavCommand::SetFocus(FocusTarget::Main));
    diff_select_file_effects(state, path)
}

fn sync_git_selection_for_path(state: &mut AppState, path: &str) {
    let viewport_rows = git_viewport_rows(state);
    let show_untracked = state.config.git.show_untracked;
    let rows = build_panel_rows(&state.git.file_entries, show_untracked);
    if let Some(row) = row_for_path(&rows, path) {
        git_select_row(&mut state.git, row, viewport_rows, show_untracked);
    } else {
        state.git.selected_path = Some(path.to_string());
    }
}

pub fn diff_select_file_effects(state: &mut AppState, path: String) -> Vec<SideEffect> {
    if state.diff.selected_path.as_deref() == Some(path.as_str()) {
        return Vec::new();
    }

    let source = state.diff.source;
    state.diff.begin_load(path.clone());
    sync_git_selection_for_path(state, &path);
    state.dirty = true;
    vec![SideEffect::LoadFileDiff { path, source }]
}

fn reduce_diff_select_file(state: &mut AppState, path: String) -> Vec<SideEffect> {
    diff_select_file_effects(state, path)
}

fn reduce_diff_move_file(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    diff_move_file_effects(state, delta)
}

pub fn diff_move_file_effects(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    let paths = changed_file_paths(&state.git.file_entries, state.config.git.show_untracked);
    let Some(path) = adjacent_changed_file(&paths, state.diff.selected_path.as_deref(), delta)
    else {
        return Vec::new();
    };

    diff_select_file_effects(state, path)
}

fn reduce_diff_loaded(
    state: &mut AppState,
    result: crate::diff::FileDiffLoadResult,
) -> Vec<SideEffect> {
    state.diff.apply_loaded(result);
    state.dirty = true;
    Vec::new()
}

fn reduce_diff_toggle_source(state: &mut AppState) -> Vec<SideEffect> {
    diff_set_source_effects(state, state.diff.source.toggle())
}

pub fn diff_set_source_effects(
    state: &mut AppState,
    source: crate::diff::DiffSource,
) -> Vec<SideEffect> {
    if state.diff.source == source {
        return Vec::new();
    }

    let Some(path) = state.diff.selected_path.clone() else {
        state.diff.source = source;
        state.dirty = true;
        return Vec::new();
    };

    state.diff.source = source;
    state.diff.begin_source_reload();
    state.dirty = true;
    vec![SideEffect::LoadFileDiff { path, source }]
}

fn reduce_diff_set_source(
    state: &mut AppState,
    source: crate::diff::DiffSource,
) -> Vec<SideEffect> {
    diff_set_source_effects(state, source)
}

fn reduce_file_tree_refresh(state: &mut AppState) -> Vec<SideEffect> {
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
            effects.push(SideEffect::LoadDirectoryChildren(path));
        }
    }

    state.dirty = true;
    effects
}

fn reduce_file_tree_children_loaded(
    state: &mut AppState,
    parent: PathBuf,
    children: Vec<crate::file_tree::DirectoryEntry>,
    error: Option<String>,
) -> Vec<SideEffect> {
    state
        .file_tree
        .apply_children_loaded(&parent, children, error);
    sync_git_statuses_to_file_tree(state);
    state.dirty = true;
    Vec::new()
}

fn preview_viewport_rows(state: &AppState) -> usize {
    diff_viewport_rows(state)
}

fn diff_viewport_rows(state: &AppState) -> usize {
    state.layout.rects.main_content.height.saturating_sub(4) as usize
}

fn diff_text_width(state: &AppState) -> usize {
    let area_width = state.layout.rects.main_content.width.saturating_sub(4) as usize;
    let gutter = diff_gutter_width(&state.diff.lines);
    area_width.saturating_sub(gutter).max(1)
}

fn diff_gutter_width(lines: &[crate::diff::DiffLine]) -> usize {
    let old_width = max_lineno_width(lines.iter().filter_map(|line| line.old_lineno));
    let new_width = max_lineno_width(lines.iter().filter_map(|line| line.new_lineno));
    if old_width == 0 && new_width == 0 {
        return 0;
    }
    old_width + 1 + new_width + 2
}

fn max_lineno_width(values: impl Iterator<Item = u32>) -> usize {
    values
        .map(|value| value.ilog10() as usize + 1)
        .max()
        .unwrap_or(1)
}

fn reduce_preview_file(state: &mut AppState, path: PathBuf, line: Option<u32>) -> Vec<SideEffect> {
    state.preview.begin_load(path.clone(), line);
    state
        .navigation
        .apply(NavCommand::SelectMainTab(MainTab::Preview));
    state
        .navigation
        .apply(NavCommand::SetFocus(FocusTarget::Main));
    state.dirty = true;
    vec![SideEffect::LoadPreviewFile(path)]
}

fn reduce_preview_loaded(
    state: &mut AppState,
    path: PathBuf,
    result: crate::preview::PreviewLoadResult,
) -> Vec<SideEffect> {
    if state.preview.path.as_ref() != Some(&path) {
        return Vec::new();
    }

    state
        .preview
        .apply_loaded(path, result, preview_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_preview_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.preview.scroll(delta, preview_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_preview_page_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state
        .preview
        .page_scroll(delta, preview_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_diff_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.diff.scroll(delta, diff_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_diff_page_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.diff.page_scroll(delta, diff_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_diff_horizontal_scroll(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 || state.config.diff.word_wrap {
        return Vec::new();
    }

    state.diff.scroll_horizontal(delta, diff_text_width(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_search_set_query(state: &mut AppState, query: String) -> Vec<SideEffect> {
    state.search.schedule_query(query);
    state.dirty = true;
    vec![SideEffect::CancelSearch]
}

fn reduce_search_append_char(state: &mut AppState, ch: char) -> Vec<SideEffect> {
    let mut query = state.search.query.clone();
    query.push(ch);
    reduce_search_set_query(state, query)
}

fn reduce_search_backspace(state: &mut AppState) -> Vec<SideEffect> {
    if state.search.query.is_empty() {
        return Vec::new();
    }

    let mut query = state.search.query.clone();
    query.pop();
    reduce_search_set_query(state, query)
}

fn reduce_search_clear(state: &mut AppState) -> Vec<SideEffect> {
    state.search.clear_query();
    state.dirty = true;
    vec![SideEffect::CancelSearch]
}

fn reduce_search_set_mode(
    state: &mut AppState,
    mode: crate::search::SearchMode,
) -> Vec<SideEffect> {
    state.search.set_mode(mode);
    state.dirty = true;
    vec![SideEffect::CancelSearch]
}

fn reduce_search_execute(state: &mut AppState) -> Vec<SideEffect> {
    let generation = state.search.begin_execute();
    state.dirty = true;

    if state.search.query.is_empty() {
        return vec![SideEffect::CancelSearch];
    }

    vec![SideEffect::RunSearch {
        mode: state.search.mode,
        query: state.search.query.clone(),
        generation,
    }]
}

fn reduce_search_cancel(state: &mut AppState) -> Vec<SideEffect> {
    state.search.cancel();
    state.dirty = true;
    vec![SideEffect::CancelSearch]
}

fn search_viewport_rows(state: &AppState) -> usize {
    state.layout.rects.left_content.height.saturating_sub(4) as usize
}

fn reduce_search_move_selection(state: &mut AppState, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state
        .search
        .move_selection(delta, search_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_search_select(state: &mut AppState, index: usize) -> Vec<SideEffect> {
    state
        .search
        .select_index(index, search_viewport_rows(state));
    state.dirty = true;
    Vec::new()
}

fn reduce_search_completed(
    state: &mut AppState,
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
    state.dirty = true;
    Vec::new()
}

fn reduce_open_editor(state: &mut AppState, path: PathBuf, line: Option<u32>) -> Vec<SideEffect> {
    state.dirty = true;
    vec![SideEffect::LaunchEditor { path, line }]
}

fn reduce_modal_dismiss(state: &mut AppState) -> Vec<SideEffect> {
    if state.notifications.modal.is_some() {
        state.notifications.dismiss_modal();
        state.dirty = true;
    }
    Vec::new()
}

fn reduce_editor_launched(state: &mut AppState, path: PathBuf, command: String) -> Vec<SideEffect> {
    state.logs.push_info(format!(
        "Launched editor `{command}` for {}",
        path.display()
    ));
    state.dirty = true;
    Vec::new()
}

fn reduce_editor_launch_failed(
    state: &mut AppState,
    path: PathBuf,
    error: String,
    show_modal: bool,
) -> Vec<SideEffect> {
    state.logs.push_error(format!(
        "Editor launch failed for {}: {error}",
        path.display()
    ));
    if show_modal {
        state.notifications.show_modal(
            "Editor command not found",
            format!("{error}\n\nPress Esc to dismiss."),
        );
    } else {
        state.notifications.show_toast(error);
    }
    state.dirty = true;
    Vec::new()
}

fn reduce_clipboard_copy(state: &mut AppState) -> Vec<SideEffect> {
    let Some(text) = resolve_copy_text(state) else {
        state.notifications.show_toast("Nothing to copy");
        state.dirty = true;
        return Vec::new();
    };

    state.dirty = true;
    vec![SideEffect::CopyToClipboard(text)]
}

fn reduce_clipboard_cut(state: &mut AppState) -> Vec<SideEffect> {
    let Some(text) = resolve_copy_text(state) else {
        state.notifications.show_toast("Nothing to cut");
        state.dirty = true;
        return Vec::new();
    };

    apply_cut_mutation(state);
    state.dirty = true;
    vec![SideEffect::CopyToClipboard(text)]
}

fn reduce_clipboard_paste(state: &mut AppState) -> Vec<SideEffect> {
    state.dirty = true;
    vec![SideEffect::PasteFromClipboard]
}

fn reduce_paste_text(state: &mut AppState, text: String) -> Vec<SideEffect> {
    if text.is_empty() {
        return Vec::new();
    }

    state.dirty = true;
    match crate::clipboard::resolve_paste_target(state) {
        PasteTarget::PaletteInput => {
            state.palette.history_cursor = None;
            state.palette.input.push_str(&text);
            refresh_matches(state);
            Vec::new()
        }
        PasteTarget::SearchQuery => {
            let mut query = state.search.query.clone();
            query.push_str(&text);
            reduce_search_set_query(state, query)
        }
        PasteTarget::ShellPty => vec![SideEffect::WriteShell(crate::clipboard::pty_paste_bytes(
            &text,
        ))],
        PasteTarget::AgentPty => vec![SideEffect::WriteAgent(crate::clipboard::pty_paste_bytes(
            &text,
        ))],
        PasteTarget::Unsupported => {
            state
                .notifications
                .show_toast("Paste is not supported in this pane");
            Vec::new()
        }
    }
}

fn reduce_selection_begin(
    state: &mut AppState,
    pane: crate::selection::SelectionPane,
    line: usize,
    col: usize,
) -> Vec<SideEffect> {
    state
        .text_selection
        .begin(pane, crate::selection::TextPosition { line, col });
    state.dirty = true;
    Vec::new()
}

fn reduce_selection_extend(state: &mut AppState, line: usize, col: usize) -> Vec<SideEffect> {
    if state.text_selection.dragging {
        state
            .text_selection
            .extend(crate::selection::TextPosition { line, col });
        state.dirty = true;
    }
    Vec::new()
}

fn reduce_selection_end(state: &mut AppState) -> Vec<SideEffect> {
    state.text_selection.end_drag();
    state.dirty = true;
    Vec::new()
}

fn reduce_selection_clear(state: &mut AppState) -> Vec<SideEffect> {
    if state.text_selection.pane.is_some() {
        state.text_selection.clear();
        state.dirty = true;
    }
    Vec::new()
}

fn apply_cut_mutation(state: &mut AppState) {
    if state.palette.open {
        state.palette.input.clear();
        refresh_matches(state);
        return;
    }

    if state.navigation.focus == FocusTarget::Left
        && state.navigation.left_tab == LeftNavTab::Search
        && state.search.results.is_empty()
    {
        state.search.query.clear();
    }
}

fn reduce_fs_changed(state: &mut AppState, paths: Vec<PathBuf>) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    let reload_dirs = state
        .file_tree
        .apply_fs_invalidation(&state.repo_root, &paths);
    if !reload_dirs.is_empty() {
        state.dirty = true;
        for dir in reload_dirs {
            effects.push(SideEffect::LoadDirectoryChildren(dir));
        }
    }

    if let Some(preview_path) = state.preview.path.clone() {
        if !state.preview.loading
            && crate::watcher::preview_reload_paths(&paths, &preview_path)
            && !preview_file_unchanged(&preview_path, state.preview.loaded_mtime)
        {
            state.preview.begin_reload();
            state.dirty = true;
            effects.push(SideEffect::LoadPreviewFile(preview_path));
        }
    }

    if state.workspace_meta.is_git_repo && state.config.git.watch {
        effects.extend(reduce_git_refresh_requested(state));
    }

    effects
}

fn preview_file_unchanged(
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::agent::AgentStatus;
    use crate::config::ResolvedConfig;
    use crate::git::{BranchEntry, GitFileEntry, GitFileStatus};
    use crate::layout::compute_layout;
    use crate::layout::FocusTarget;
    use crate::navigation::{LeftNavTab, MainTab, NavCommand, NavigationState};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;
    use crate::file_tree::{DirectoryEntry, FileTreeState};
    use crate::preview::{PreviewLoadResult, PreviewState};
    use crate::search::{SearchMode, SearchResult, SearchState};
    use crate::state::domains::{
        AgentState, BranchState, CommandPaletteState, DiffState, GitHubState, GitState, ShellState,
        StatusBarState, WorkspaceMeta,
    };

    fn test_state() -> AppState {
        AppState {
            config: ResolvedConfig::default(),
            navigation: NavigationState::default(),
            layout: compute_layout(120, 40, 30).expect("layout"),
            theme: load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            repo_root: PathBuf::from("."),
            dirty: false,
            file_tree: FileTreeState::at_root(PathBuf::from(".")),
            preview: PreviewState::default(),
            search: SearchState::default(),
            git: GitState::default(),
            branches: BranchState::default(),
            diff: DiffState::default(),
            github: GitHubState::default(),
            agent: AgentState::default(),
            shell: ShellState::default(),
            palette: CommandPaletteState::default(),
            logs: crate::state::domains::LogsState::default(),
            notifications: crate::state::domains::NotificationState::default(),
            status_bar: StatusBarState::default(),
            workspace_meta: WorkspaceMeta::default(),
            text_selection: crate::selection::TextSelection::default(),
            pty_cursor_blink_on: true,
        }
    }

    #[test]
    fn navigation_command_sets_dirty() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectLeftTab(
                LeftNavTab::Git,
            ))),
        );
        assert!(state.dirty);
        assert_eq!(state.navigation.left_tab, LeftNavTab::Git);
    }

    #[test]
    fn orthogonal_tabs_preserved_in_app_state() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectLeftTab(
                LeftNavTab::Git,
            ))),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Issues,
            ))),
        );
        assert_eq!(state.navigation.left_tab, LeftNavTab::Git);
        assert_eq!(state.navigation.main_tab, MainTab::Issues);
    }

    #[test]
    fn git_refresh_preserves_selection_when_file_still_modified() {
        let mut state = test_state();
        state.git.selected_path = Some("src/main.rs".to_string());
        state.git.file_entries = vec![
            GitFileEntry {
                path: "src/main.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "other.rs".to_string(),
                status: GitFileStatus::Modified,
            },
        ];

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                ahead: 0,
                behind: 0,
                file_entries: vec![
                    GitFileEntry {
                        path: "src/main.rs".to_string(),
                        status: GitFileStatus::Modified,
                    },
                    GitFileEntry {
                        path: "new.rs".to_string(),
                        status: GitFileStatus::Added,
                    },
                ],
                error: None,
            },
        );

        assert_eq!(state.git.selected_path, Some("src/main.rs".to_string()));
    }

    #[test]
    fn git_refresh_clears_selection_when_file_no_longer_modified() {
        let mut state = test_state();
        state.git.selected_path = Some("removed.rs".to_string());

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                ahead: 0,
                behind: 0,
                file_entries: vec![GitFileEntry {
                    path: "other.rs".to_string(),
                    status: GitFileStatus::Modified,
                }],
                error: None,
            },
        );

        assert_eq!(state.git.selected_path, None);
    }

    #[test]
    fn git_status_refresh_preserves_file_tree_selection() {
        let mut state = test_state();
        let selected = state.file_tree.root.join("src/main.rs");
        state.file_tree.nodes.insert(
            selected.clone(),
            crate::file_tree::FileNode {
                path: selected.clone(),
                name: "main.rs".to_string(),
                is_dir: false,
                expanded: false,
                children_loaded: true,
                load_error: None,
                git_status: None,
            },
        );
        state.file_tree.selected = Some(selected.clone());
        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                ahead: 0,
                behind: 0,
                file_entries: vec![GitFileEntry {
                    path: "src/main.rs".to_string(),
                    status: GitFileStatus::Modified,
                }],
                error: None,
            },
        );
        assert_eq!(state.file_tree.selected, Some(selected));
        assert_eq!(
            state.file_tree.nodes[&state.file_tree.root.join("src/main.rs")].git_status,
            Some(GitFileStatus::Modified)
        );
    }

    #[test]
    fn git_refresh_requested_emits_side_effect_for_git_repo() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = reduce(&mut state, AppEvent::GitRefreshRequested);
        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn git_refresh_requested_skips_non_git_repo() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = false;
        let effects = reduce(&mut state, AppEvent::GitRefreshRequested);
        assert!(effects.is_empty());
        assert!(!state.git.loading);
    }

    #[test]
    fn git_status_updated_sets_branch_and_tracking_counts() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: Some("feature/42".to_string()),
                ahead: 2,
                behind: 1,
                file_entries: Vec::new(),
                error: None,
            },
        );

        assert_eq!(state.git.branch.as_deref(), Some("feature/42"));
        assert_eq!(state.git.ahead, 2);
        assert_eq!(state.git.behind, 1);
        assert!(!state.git.loading);
        assert!(state.git.error.is_none());
    }

    #[test]
    fn git_status_updated_records_error_without_branch() {
        let mut state = test_state();
        state.git.branch = Some("main".to_string());

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                ahead: 0,
                behind: 0,
                file_entries: Vec::new(),
                error: Some("corrupt".to_string()),
            },
        );

        assert_eq!(state.git.branch, Some("main".to_string()));
        assert_eq!(state.git.error.as_deref(), Some("corrupt"));
        assert_eq!(state.logs.entries.len(), 1);
    }

    #[test]
    fn branch_tab_navigation_spawns_branch_list() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Branches,
            ))),
        );

        assert!(effects.contains(&SideEffect::SpawnBranchList));
        assert!(state.branches.loading);
    }

    #[test]
    fn branch_checkout_selected_spawns_checkout_for_other_branch() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.git.branch = Some("main".to_string());
        state.branches.entries = vec![
            BranchEntry {
                name: "main".to_string(),
                is_current: true,
            },
            BranchEntry {
                name: "dev".to_string(),
                is_current: false,
            },
        ];
        state.branches.selected_index = Some(1);

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::BranchCheckoutSelected),
        );

        assert!(effects.iter().any(
            |effect| matches!(effect, SideEffect::SpawnBranchCheckout { name } if name == "dev")
        ));
        assert!(state.branches.checkout_loading);
    }

    #[test]
    fn branch_checkout_selected_no_op_for_current_branch() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.git.branch = Some("main".to_string());
        state.branches.entries = vec![BranchEntry {
            name: "main".to_string(),
            is_current: true,
        }];
        state.branches.selected_index = Some(0);

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::BranchCheckoutSelected),
        );

        assert!(effects.is_empty());
        assert!(!state.branches.checkout_loading);
    }

    #[test]
    fn branch_list_loaded_preserves_selection_by_name() {
        let mut state = test_state();
        state.branches.selected_index = Some(1);
        state.branches.entries = vec![
            BranchEntry {
                name: "main".to_string(),
                is_current: true,
            },
            BranchEntry {
                name: "dev".to_string(),
                is_current: false,
            },
        ];
        state.branches.scroll_offset = 1;

        reduce(
            &mut state,
            AppEvent::BranchListLoaded {
                entries: vec![
                    BranchEntry {
                        name: "dev".to_string(),
                        is_current: false,
                    },
                    BranchEntry {
                        name: "feature".to_string(),
                        is_current: false,
                    },
                    BranchEntry {
                        name: "main".to_string(),
                        is_current: true,
                    },
                ],
                error: None,
            },
        );

        assert_eq!(state.branches.selected_index, Some(0));
        assert_eq!(state.branches.scroll_offset, 1);
    }

    #[test]
    fn branch_checkout_completed_refreshes_git_and_branch_list() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.branches.entries = vec![BranchEntry {
            name: "dev".to_string(),
            is_current: false,
        }];
        state.branches.selected_index = Some(0);

        let effects = reduce(
            &mut state,
            AppEvent::BranchCheckoutCompleted {
                branch_name: "dev".to_string(),
                error: None,
            },
        );

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(effects.contains(&SideEffect::SpawnBranchList));
        assert!(!state.branches.checkout_loading);
        assert!(state.branches.checkout_error.is_none());
    }

    #[test]
    fn git_status_updated_applies_incremental_file_patch() {
        let mut state = test_state();
        state.git.file_entries = vec![GitFileEntry {
            path: "src/main.rs".to_string(),
            status: GitFileStatus::Modified,
        }];

        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: Some("main".to_string()),
                ahead: 0,
                behind: 0,
                file_entries: vec![
                    GitFileEntry {
                        path: "src/main.rs".to_string(),
                        status: GitFileStatus::Modified,
                    },
                    GitFileEntry {
                        path: "src/new.rs".to_string(),
                        status: GitFileStatus::Added,
                    },
                ],
                error: None,
            },
        );

        assert_eq!(state.git.file_entries.len(), 2);
        assert!(state
            .git
            .file_entries
            .iter()
            .any(|entry| entry.path == "src/new.rs"));
    }

    #[test]
    fn fs_changed_many_paths_trigger_single_git_refresh() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = true;

        let paths: Vec<PathBuf> = (0..50)
            .map(|index| PathBuf::from(format!("/repo/file{index}.rs")))
            .collect();
        let effects = reduce(&mut state, AppEvent::FsChanged { paths });

        assert_eq!(
            effects
                .iter()
                .filter(|effect| **effect == SideEffect::SpawnGitRefresh)
                .count(),
            1
        );
        assert!(state.git.loading);
    }

    #[test]
    fn fs_changed_git_head_triggers_refresh() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = true;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/repo/.git/HEAD")],
            },
        );

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn fs_changed_requests_git_refresh_when_watch_enabled() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = true;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/main.rs")],
            },
        );

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn fs_changed_skips_git_refresh_when_watch_disabled() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = false;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/main.rs")],
            },
        );

        assert!(!effects.contains(&SideEffect::SpawnGitRefresh));
    }

    #[test]
    fn request_git_refresh_sets_loading() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::RequestGitRefresh));

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn git_refresh_command_emits_side_effect() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::GitRefresh));
        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
    }

    #[test]
    fn git_open_selected_switches_to_main_diff_tab() {
        let mut state = test_state();
        state.git.selected_path = Some("src/main.rs".to_string());

        let effects = reduce(&mut state, AppEvent::Command(AppCommand::GitOpenSelected));

        assert_eq!(state.navigation.main_tab, MainTab::Diff);
        assert_eq!(state.navigation.focus, FocusTarget::Main);
        assert_eq!(state.diff.selected_path.as_deref(), Some("src/main.rs"));
        assert!(state.diff.loading);
        assert!(
            effects
                .iter()
                .any(|effect| matches!(effect, SideEffect::LoadFileDiff { path, .. } if path == "src/main.rs"))
        );
    }

    #[test]
    fn diff_loaded_populates_lines_for_selected_path() {
        use crate::diff::{DiffLine, DiffLineKind, FileDiffLoadResult};

        let mut state = test_state();
        state.diff.begin_load("src/main.rs".to_string());

        reduce(
            &mut state,
            AppEvent::DiffLoaded {
                result: FileDiffLoadResult {
                    path: "src/main.rs".to_string(),
                    lines: vec![DiffLine {
                        kind: DiffLineKind::Addition,
                        content: "+hello".to_string(),
                        old_lineno: None,
                        new_lineno: Some(1),
                    }],
                    is_binary: false,
                    error: None,
                },
            },
        );

        assert!(!state.diff.loading);
        assert_eq!(state.diff.lines.len(), 1);
    }

    #[test]
    fn diff_scroll_moves_viewport() {
        let mut state = test_state();
        state.diff.selected_path = Some("src/main.rs".to_string());
        state.diff.lines = (0..100)
            .map(|index| crate::diff::DiffLine {
                kind: crate::diff::DiffLineKind::Context,
                content: format!(" line {index}"),
                old_lineno: Some(index as u32 + 1),
                new_lineno: Some(index as u32 + 1),
            })
            .collect();

        reduce(&mut state, AppEvent::Command(AppCommand::DiffScroll(10)));
        assert_eq!(state.diff.scroll_offset, 10);

        reduce(&mut state, AppEvent::Command(AppCommand::DiffScroll(-20)));
        assert_eq!(state.diff.scroll_offset, 0);
    }

    #[test]
    fn diff_toggle_source_switches_and_reloads() {
        let mut state = test_state();
        state.diff.selected_path = Some("src/main.rs".to_string());
        state.diff.source = crate::diff::DiffSource::Unstaged;

        let effects = reduce(&mut state, AppEvent::Command(AppCommand::DiffToggleSource));

        assert_eq!(state.diff.source, crate::diff::DiffSource::Staged);
        assert!(state.diff.loading);
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::LoadFileDiff { path, source }
                if path == "src/main.rs" && *source == crate::diff::DiffSource::Staged
        )));
    }

    #[test]
    fn diff_set_source_noop_when_unchanged() {
        let mut state = test_state();
        state.diff.selected_path = Some("src/main.rs".to_string());
        state.diff.source = crate::diff::DiffSource::Staged;

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::DiffSetSource(crate::diff::DiffSource::Staged)),
        );

        assert!(effects.is_empty());
        assert!(!state.diff.loading);
    }

    #[test]
    fn diff_toggle_source_without_file_updates_source_only() {
        let mut state = test_state();
        state.diff.source = crate::diff::DiffSource::Unstaged;

        let effects = reduce(&mut state, AppEvent::Command(AppCommand::DiffToggleSource));

        assert_eq!(state.diff.source, crate::diff::DiffSource::Staged);
        assert!(effects.is_empty());
    }

    #[test]
    fn diff_next_file_selects_next_changed_file_and_reloads() {
        let mut state = test_state();
        state.git.file_entries = vec![
            GitFileEntry {
                path: "a.rs".to_string(),
                status: crate::git::GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "b.rs".to_string(),
                status: crate::git::GitFileStatus::Modified,
            },
        ];
        state.diff.selected_path = Some("a.rs".to_string());

        let effects = reduce(&mut state, AppEvent::Command(AppCommand::DiffNextFile));

        assert_eq!(state.diff.selected_path.as_deref(), Some("b.rs"));
        assert_eq!(state.git.selected_path.as_deref(), Some("b.rs"));
        assert!(state.diff.loading);
        assert!(effects.iter().any(
            |effect| matches!(effect, SideEffect::LoadFileDiff { path, .. } if path == "b.rs")
        ));
    }

    #[test]
    fn diff_prev_file_clamps_at_first_file() {
        let mut state = test_state();
        state.git.file_entries = vec![
            GitFileEntry {
                path: "a.rs".to_string(),
                status: crate::git::GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "b.rs".to_string(),
                status: crate::git::GitFileStatus::Modified,
            },
        ];
        state.diff.selected_path = Some("a.rs".to_string());

        let effects = reduce(&mut state, AppEvent::Command(AppCommand::DiffPrevFile));

        assert_eq!(state.diff.selected_path.as_deref(), Some("a.rs"));
        assert!(effects.is_empty());
        assert!(!state.diff.loading);
    }

    #[test]
    fn diff_file_navigation_preserves_scroll_when_returning() {
        let mut state = test_state();
        state.git.file_entries = vec![
            GitFileEntry {
                path: "a.rs".to_string(),
                status: crate::git::GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "b.rs".to_string(),
                status: crate::git::GitFileStatus::Modified,
            },
        ];
        state.diff.selected_path = Some("a.rs".to_string());
        state.diff.scroll_offset = 42;
        state.diff.horizontal_scroll_offset = 7;

        reduce(&mut state, AppEvent::Command(AppCommand::DiffNextFile));
        reduce(&mut state, AppEvent::Command(AppCommand::DiffPrevFile));

        assert_eq!(state.diff.selected_path.as_deref(), Some("a.rs"));
        assert_eq!(state.diff.scroll_offset, 42);
        assert_eq!(state.diff.horizontal_scroll_offset, 7);
    }

    #[test]
    fn navigating_to_issues_triggers_github_auth_check() {
        let mut state = test_state();

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Issues,
            ))),
        );

        assert!(state.github.loading);
        assert!(effects.contains(&SideEffect::SpawnGitHubAuthCheck));
    }

    #[test]
    fn github_auth_checked_updates_state() {
        use crate::github::{GitHubAuthCheckResult, GitHubAuthErrorKind};

        let mut state = test_state();
        state.github.loading = true;

        reduce(
            &mut state,
            AppEvent::GitHubAuthChecked {
                result: GitHubAuthCheckResult {
                    auth_ok: false,
                    error_kind: Some(GitHubAuthErrorKind::NotAuthenticated),
                    message: "Not logged in".to_string(),
                },
            },
        );

        assert!(state.github.auth_checked);
        assert!(!state.github.auth_ok);
        assert_eq!(state.github.error.as_deref(), Some("Not logged in"));
        assert!(!state.github.loading);
    }

    #[test]
    fn github_refresh_rechecks_auth() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        state.github.auth_checked = true;
        state.github.auth_ok = true;

        let effects = reduce(&mut state, AppEvent::Command(AppCommand::GitHubRefresh));

        assert!(!state.github.auth_checked);
        assert!(state.github.loading);
        assert!(effects.contains(&SideEffect::SpawnGitHubAuthCheck));
    }

    #[test]
    fn github_auth_success_on_issues_tab_loads_issue_list() {
        use crate::github::GitHubAuthCheckResult;

        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        state.github.loading = true;

        let effects = reduce(
            &mut state,
            AppEvent::GitHubAuthChecked {
                result: GitHubAuthCheckResult {
                    auth_ok: true,
                    error_kind: None,
                    message: String::new(),
                },
            },
        );

        assert!(state.github.auth_ok);
        assert!(state.github.issues_loading);
        assert!(effects.contains(&SideEffect::SpawnGitHubIssueList));
    }

    #[test]
    fn github_auth_success_on_prs_tab_loads_pr_list() {
        use crate::github::GitHubAuthCheckResult;

        let mut state = test_state();
        state.navigation.main_tab = MainTab::Prs;
        state.github.loading = true;

        let effects = reduce(
            &mut state,
            AppEvent::GitHubAuthChecked {
                result: GitHubAuthCheckResult {
                    auth_ok: true,
                    error_kind: None,
                    message: String::new(),
                },
            },
        );

        assert!(state.github.auth_ok);
        assert!(state.github.prs_loading);
        assert!(effects.contains(&SideEffect::SpawnGitHubPrList));
    }

    #[test]
    fn github_prs_loaded_populates_list_and_selection() {
        use crate::github::{PrListLoadResult, PrState, PullRequest};

        let mut state = test_state();
        state.github.prs_loading = true;

        reduce(
            &mut state,
            AppEvent::GitHubPrsLoaded {
                result: PrListLoadResult {
                    prs: vec![PullRequest {
                        number: 59,
                        title: "PR list via gh json".to_string(),
                        state: PrState::Open,
                        author: "octocat".to_string(),
                        is_draft: false,
                    }],
                    error: None,
                },
            },
        );

        assert!(!state.github.prs_loading);
        assert_eq!(state.github.prs.len(), 1);
        assert_eq!(state.github.selected_pr, Some(59));
        assert!(state.github.prs_loaded_at.is_some());
    }

    #[test]
    fn github_pr_list_cache_skips_reload_within_sixty_seconds() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;
        state.github.auth_ok = true;
        state.github.auth_checked = true;
        state.github.prs_loaded_at = Some(SystemTime::now());
        state.github.prs = vec![crate::github::PullRequest {
            number: 59,
            title: "Cached".to_string(),
            state: crate::github::PrState::Open,
            author: "octocat".to_string(),
            is_draft: false,
        }];

        let effects = super::github_pr_list_access_effects(&mut state, false);

        assert!(effects.is_empty());
        assert!(!state.github.prs_loading);
    }

    #[test]
    fn github_issues_loaded_populates_list_and_selection() {
        use crate::github::{Issue, IssueListLoadResult, IssueState};

        let mut state = test_state();
        state.github.issues_loading = true;

        reduce(
            &mut state,
            AppEvent::GitHubIssuesLoaded {
                result: IssueListLoadResult {
                    issues: vec![Issue {
                        number: 55,
                        title: "Issue list".to_string(),
                        state: IssueState::Open,
                        labels: Vec::new(),
                        assignees: Vec::new(),
                    }],
                    error: None,
                },
            },
        );

        assert!(!state.github.issues_loading);
        assert_eq!(state.github.issues.len(), 1);
        assert_eq!(state.github.selected_issue, Some(55));
        assert!(state.github.issues_loaded_at.is_some());
    }

    #[test]
    fn github_issue_list_cache_skips_reload_within_sixty_seconds() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;
        state.github.auth_ok = true;
        state.github.auth_checked = true;
        state.github.issues_loaded_at = Some(SystemTime::now());
        state.github.issues = vec![crate::github::Issue {
            number: 1,
            title: "Cached".to_string(),
            state: crate::github::IssueState::Open,
            labels: Vec::new(),
            assignees: Vec::new(),
        }];

        let effects = super::github_issue_list_access_effects(&mut state, false);

        assert!(effects.is_empty());
        assert!(!state.github.issues_loading);
    }

    #[test]
    fn github_select_left_pane_switches_main_tab() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;

        reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubSelectLeftPane(GitHubLeftPane::Prs)),
        );

        assert_eq!(state.github.left_pane, GitHubLeftPane::Prs);
        assert_eq!(state.navigation.main_tab, MainTab::Prs);
    }

    #[test]
    fn navigating_to_prs_on_gh_left_tab_selects_prs_hub() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;
        state.github.left_pane = GitHubLeftPane::Issues;

        reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Prs,
            ))),
        );

        assert_eq!(state.github.left_pane, GitHubLeftPane::Prs);
    }

    #[test]
    fn github_open_selected_focuses_main_issues_tab_and_loads_detail() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.auth_checked = true;
        state.github.selected_issue = Some(42);

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubOpenSelected),
        );

        assert_eq!(state.navigation.main_tab, MainTab::Issues);
        assert_eq!(state.navigation.focus, FocusTarget::Main);
        assert!(state.github.issue_detail_loading);
        assert!(effects.contains(&SideEffect::SpawnGitHubIssueDetail { number: 42 }));
    }

    #[test]
    fn github_issue_detail_loaded_populates_detail() {
        use crate::github::{IssueDetail, IssueDetailLoadResult, IssueState};

        let mut state = test_state();
        state.github.issue_detail_number = Some(56);
        state.github.issue_detail_loading = true;

        reduce(
            &mut state,
            AppEvent::GitHubIssueDetailLoaded {
                number: 56,
                result: IssueDetailLoadResult {
                    detail: Some(IssueDetail {
                        number: 56,
                        title: "Issue detail view".to_string(),
                        state: IssueState::Open,
                        author: "pacificnm".to_string(),
                        labels: Vec::new(),
                        assignees: Vec::new(),
                        display_lines: vec!["#56 Issue detail view".to_string()],
                    }),
                    error: None,
                },
            },
        );

        assert!(!state.github.issue_detail_loading);
        assert_eq!(
            state
                .github
                .issue_detail
                .as_ref()
                .map(|detail| detail.number),
            Some(56)
        );
    }

    #[test]
    fn github_issue_detail_scroll_moves_offset() {
        use crate::github::{IssueDetail, IssueState};

        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        state.github.issue_detail = Some(IssueDetail {
            number: 1,
            title: "Test".to_string(),
            state: IssueState::Open,
            author: "user".to_string(),
            labels: Vec::new(),
            assignees: Vec::new(),
            display_lines: (0..100).map(|index| format!("line {index}")).collect(),
        });

        reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubIssueDetailScroll(5)),
        );

        assert_eq!(state.github.issue_detail_scroll_offset, 5);
    }

    #[test]
    fn github_open_selected_on_prs_tab_loads_pr_detail() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.auth_checked = true;
        state.github.left_pane = GitHubLeftPane::Prs;
        state.github.selected_pr = Some(60);

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubOpenSelected),
        );

        assert_eq!(state.navigation.main_tab, MainTab::Prs);
        assert_eq!(state.navigation.focus, FocusTarget::Main);
        assert!(state.github.pr_detail_loading);
        assert!(effects.contains(&SideEffect::SpawnGitHubPrDetail { number: 60 }));
    }

    #[test]
    fn github_pr_detail_loaded_populates_detail() {
        use crate::github::{PrDetail, PrDetailLoadResult, PrState};

        let mut state = test_state();
        state.github.pr_detail_number = Some(60);
        state.github.pr_detail_loading = true;

        reduce(
            &mut state,
            AppEvent::GitHubPrDetailLoaded {
                number: 60,
                result: PrDetailLoadResult {
                    detail: Some(PrDetail {
                        number: 60,
                        title: "PR detail view".to_string(),
                        state: PrState::Open,
                        author: "pacificnm".to_string(),
                        display_lines: vec!["#60 PR detail view".to_string()],
                    }),
                    error: None,
                },
            },
        );

        assert!(!state.github.pr_detail_loading);
        assert_eq!(
            state.github.pr_detail.as_ref().map(|detail| detail.number),
            Some(60)
        );
    }

    #[test]
    fn github_pr_detail_scroll_moves_offset() {
        use crate::github::{PrDetail, PrState};

        let mut state = test_state();
        state.navigation.main_tab = MainTab::Prs;
        state.github.pr_detail = Some(PrDetail {
            number: 1,
            title: "Test".to_string(),
            state: PrState::Open,
            author: "user".to_string(),
            display_lines: (0..100).map(|index| format!("line {index}")).collect(),
        });

        reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubPrDetailScroll(5)),
        );

        assert_eq!(state.github.pr_detail_scroll_offset, 5);
    }

    #[test]
    fn palette_prompt_submits_issue_comment() {
        use crate::state::PalettePrompt;

        let mut state = test_state();
        state.github.selected_issue = Some(42);
        state.palette.begin_prompt(
            PalettePrompt::GitHubIssueComment { number: 42 },
            FocusTarget::Main,
        );
        state.navigation.focus = FocusTarget::CommandPalette;
        state.palette.input = "Looks good".to_string();

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );

        assert!(!state.palette.open);
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::SpawnGitHubIssueComment { number: 42, body }
                if body == "Looks good"
            )
        }));
    }

    #[test]
    fn github_issue_comment_completed_refreshes_detail() {
        use crate::github::IssueActionResult;

        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.auth_checked = true;
        state.github.selected_issue = Some(42);
        state.github.issue_detail_number = Some(42);
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));

        let effects = reduce(
            &mut state,
            AppEvent::GitHubIssueCommentCompleted {
                number: 42,
                result: IssueActionResult {
                    success: true,
                    error: None,
                    detail: None,
                },
            },
        );

        assert!(state.github.issue_action_message.is_some());
        assert!(state.github.issue_detail.is_none());
        assert!(effects.contains(&SideEffect::SpawnGitHubIssueList));
        assert!(effects.contains(&SideEffect::SpawnGitHubIssueDetail { number: 42 }));
    }

    #[test]
    fn github_issue_create_branch_completed_refreshes_git() {
        use crate::github::IssueActionResult;

        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.github.auth_ok = true;
        state.github.selected_issue = Some(42);
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));

        let effects = reduce(
            &mut state,
            AppEvent::GitHubIssueCreateBranchCompleted {
                number: 42,
                result: IssueActionResult {
                    success: true,
                    error: None,
                    detail: Some("58-create-branch-from-issue".to_string()),
                },
            },
        );

        assert_eq!(
            state.github.issue_action_message.as_deref(),
            Some("58-create-branch-from-issue")
        );
        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
    }

    #[test]
    fn github_label_picker_apply_spawns_edit() {
        use crate::github::{apply_label_picker_load, LabelPickerState, RepoLabelsLoadResult};

        let mut state = test_state();
        let mut picker = LabelPickerState::new(9, vec!["bug".to_string()]);
        apply_label_picker_load(
            &mut picker,
            RepoLabelsLoadResult {
                labels: vec![
                    crate::github::labels::RepoLabel {
                        name: "bug".to_string(),
                        description: String::new(),
                    },
                    crate::github::labels::RepoLabel {
                        name: "docs".to_string(),
                        description: String::new(),
                    },
                ],
                error: None,
            },
            &["bug".to_string()],
        );
        picker.selected[1] = true;
        state.github.label_picker = Some(picker);

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubLabelPickerApply),
        );

        assert!(state
            .github
            .label_picker
            .as_ref()
            .is_some_and(|picker| picker.applying));
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::SpawnGitHubIssueLabelApply { number: 9, labels }
                if labels == &vec!["docs".to_string()]
            )
        }));
    }

    #[test]
    fn github_open_in_browser_spawns_side_effect() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.selected_issue = Some(42);
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubOpenInBrowser),
        );

        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::SpawnGitHubOpenBrowser {
                    target: crate::github::GitHubBrowserTarget::Issue(42)
                }
            )
        }));
    }

    #[test]
    fn palette_prompt_submits_pr_create() {
        use crate::github::PrCreateRequest;
        use crate::state::{GitHubPrCreatePrompt, PalettePrompt};

        let mut state = test_state();
        state.palette.begin_prompt(
            PalettePrompt::GitHubPrCreate(GitHubPrCreatePrompt::default()),
            FocusTarget::Main,
        );
        state.navigation.focus = FocusTarget::CommandPalette;
        state.palette.input = "Fix login".to_string();

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(state.palette.open);
        assert!(effects.is_empty());

        state.palette.input = "Fixes #42".to_string();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(state.palette.open);
        assert!(effects.is_empty());

        state.palette.input = "main".to_string();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(!state.palette.open);
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::SpawnGitHubPrCreate { request }
                if *request == PrCreateRequest {
                    title: "Fix login".to_string(),
                    body: "Fixes #42".to_string(),
                    base: Some("main".to_string()),
                }
            )
        }));
    }

    #[test]
    fn github_pr_create_completed_sets_status_message() {
        use crate::github::IssueActionResult;

        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::GitHubPrCreateCompleted {
                result: IssueActionResult {
                    success: true,
                    error: None,
                    detail: Some("https://github.com/org/repo/pull/99".to_string()),
                },
            },
        );

        assert_eq!(
            state.github.issue_action_message.as_deref(),
            Some("https://github.com/org/repo/pull/99")
        );
    }

    #[test]
    fn github_open_browser_completed_sets_status_message() {
        use crate::github::{GitHubBrowserTarget, IssueActionResult};

        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::GitHubOpenBrowserCompleted {
                target: GitHubBrowserTarget::Issue(42),
                result: IssueActionResult {
                    success: true,
                    error: None,
                    detail: None,
                },
            },
        );

        assert_eq!(
            state.github.issue_action_message.as_deref(),
            Some("Opened issue #42 in browser")
        );
    }

    #[test]
    fn quit_command_emits_side_effect() {
        let mut state = test_state();
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::Quit));
        assert!(effects.contains(&SideEffect::Quit));
    }

    #[test]
    fn terminal_resize_updates_layout() {
        let mut state = test_state();
        let before = state.layout;
        reduce(
            &mut state,
            AppEvent::TerminalResize {
                width: 160,
                height: 50,
            },
        );
        assert_ne!(state.layout, before);
        assert!(state.dirty);
    }

    #[test]
    fn agent_spawn_not_requested_off_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let effects = agent_spawn_effects_if_needed(&mut state);
        assert!(effects.is_empty());
    }

    #[test]
    fn agent_spawn_requested_on_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        state.agent.spawned = false;
        let effects = agent_spawn_effects_if_needed(&mut state);
        assert!(effects.contains(&SideEffect::SpawnAgent));
    }

    #[test]
    fn agent_spawn_requested_only_once() {
        let mut state = test_state();
        state.agent.spawned = true;
        let effects = agent_spawn_effects_if_needed(&mut state);
        assert!(effects.is_empty());
    }

    #[test]
    fn selecting_agent_tab_emits_spawn_side_effect() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Agent,
            ))),
        );
        assert!(effects.contains(&SideEffect::SpawnAgent));
    }

    #[test]
    fn terminal_resize_emits_shell_resize_when_running() {
        let mut state = test_state();
        state.shell.running = true;
        state.shell.cols = shell_pty_size(&state.layout.rects).0;
        state.shell.rows = shell_pty_size(&state.layout.rects).1;

        let effects = reduce(
            &mut state,
            AppEvent::TerminalResize {
                width: 160,
                height: 50,
            },
        );

        let (cols, rows) = shell_pty_size(&state.layout.rects);
        assert!(effects.contains(&SideEffect::ResizeShell { cols, rows }));
        assert_eq!(state.shell.cols, cols);
        assert_eq!(state.shell.rows, rows);
    }

    #[test]
    fn shell_output_appends_to_scrollback_and_sets_dirty() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::ShellOutput(b"hello\nworld".to_vec()));
        assert_eq!(state.shell.scrollback.line_count(), 2);
        assert!(state.dirty);
    }

    #[test]
    fn agent_output_appends_to_scrollback_and_sets_dirty() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::AgentOutput(b"agent line\n".to_vec()));
        assert_eq!(state.agent.scrollback.line_count(), 1);
        assert!(state.dirty);
    }

    #[test]
    fn agent_exited_clears_running_and_sets_dirty() {
        let mut state = test_state();
        state.agent.running = true;
        state.agent.status = AgentStatus::Executing;
        reduce(&mut state, AppEvent::AgentExited(1));
        assert!(!state.agent.running);
        assert_eq!(state.agent.status, AgentStatus::Error);
        assert!(state.dirty);
    }

    #[test]
    fn agent_output_updates_status_from_heuristics() {
        let mut state = test_state();
        state.agent.running = true;
        reduce(
            &mut state,
            AppEvent::AgentOutput(b"Thinking about the next step\n".to_vec()),
        );
        assert_eq!(state.agent.status, AgentStatus::Thinking);
    }

    #[test]
    fn agent_exited_zero_sets_success_status() {
        let mut state = test_state();
        state.agent.running = true;
        reduce(&mut state, AppEvent::AgentExited(0));
        assert_eq!(state.agent.status, AgentStatus::Success);
        assert_eq!(state.agent.exit_code, Some(0));
        assert!(state.agent.restart_hint.is_some());
    }

    #[test]
    fn agent_restart_emits_side_effect_on_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::AgentRestart));
        assert!(effects.contains(&SideEffect::RestartAgent));
        assert!(state.dirty);
    }

    #[test]
    fn agent_restart_ignored_off_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::AgentRestart));
        assert!(effects.is_empty());
    }

    #[test]
    fn agent_exited_sets_restart_hint_with_code() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::AgentExited(2));
        assert_eq!(state.agent.exit_code, Some(2));
        assert!(state
            .agent
            .restart_hint
            .as_deref()
            .is_some_and(|hint| hint.contains("code 2")));
    }

    #[test]
    fn shell_write_emits_side_effect() {
        let mut state = test_state();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::ShellWrite(b"ls\n".to_vec())),
        );
        assert!(effects.contains(&SideEffect::WriteShell(b"ls\n".to_vec())));
    }

    #[test]
    fn agent_write_emits_side_effect() {
        let mut state = test_state();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::AgentWrite(b"hello\n".to_vec())),
        );
        assert!(effects.contains(&SideEffect::WriteAgent(b"hello\n".to_vec())));
    }

    #[test]
    fn agent_scroll_moves_viewport_and_clears_follow_tail() {
        let mut state = test_state();
        for index in 0..40 {
            state
                .agent
                .scrollback
                .append_bytes(format!("line {index}\n").as_bytes());
        }

        reduce(&mut state, AppEvent::Command(AppCommand::AgentScroll(-1)));

        assert!(!state.agent.follow_tail);
        assert!(state.dirty);
    }

    #[test]
    fn palette_open_sets_state_and_focus() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        assert!(state.palette.open);
        assert_eq!(state.navigation.focus, FocusTarget::CommandPalette);
        assert!(!state.palette.matches.is_empty());
    }

    #[test]
    fn palette_close_restores_previous_focus() {
        let mut state = test_state();
        state.navigation.focus = FocusTarget::Shell;
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteClose));
        assert!(!state.palette.open);
        assert_eq!(state.navigation.focus, FocusTarget::Shell);
    }

    #[test]
    fn palette_fuzzy_query_matches_git_refresh() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('g')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('i')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('t')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar(' ')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('r')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('e')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('f')),
        );
        let first = state.palette.matches.first().copied().expect("match");
        assert_eq!(crate::commands::COMMANDS[first].id, "git.refresh");
    }

    #[test]
    fn palette_execute_selected_closes_palette() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(!state.palette.open);
    }

    #[test]
    fn palette_execute_git_refresh_sets_loading() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('g')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('i')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('t')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar(' ')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('r')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('e')),
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('f')),
        );
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
        assert!(!state.palette.open);
    }

    #[test]
    fn palette_execute_persists_history_when_enabled() {
        let mut state = test_state();
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(effects.contains(&SideEffect::SavePaletteHistory));
    }

    #[test]
    fn palette_history_up_uses_command_title() {
        let mut state = test_state();
        state.palette.history = vec!["git.refresh".to_string()];
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        reduce(&mut state, AppEvent::Command(AppCommand::PaletteHistoryUp));
        assert_eq!(state.palette.input, "Git: Refresh Status");
    }

    #[test]
    fn startup_file_tree_contains_root_only() {
        let state = test_state();
        assert_eq!(state.file_tree.nodes.len(), 1);
        assert!(state.file_tree.children.is_empty());
    }

    #[test]
    fn file_tree_expand_emits_load_side_effect() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeExpand(root.clone())),
        );
        assert!(effects.contains(&SideEffect::LoadDirectoryChildren(root)));
        assert!(state.file_tree.loading.contains(&state.file_tree.root));
    }

    #[test]
    fn file_tree_children_loaded_updates_state() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeExpand(root.clone())),
        );
        reduce(
            &mut state,
            AppEvent::FileTreeChildrenLoaded {
                parent: root.clone(),
                children: vec![DirectoryEntry {
                    path: root.join("src"),
                    name: "src".to_string(),
                    is_dir: true,
                }],
                error: None,
            },
        );
        assert_eq!(state.file_tree.children[&root].len(), 1);
        assert!(state.file_tree.nodes.contains_key(&root.join("src")));
    }

    #[test]
    fn file_tree_move_selection_changes_selected_row() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeExpand(root.clone())),
        );
        reduce(
            &mut state,
            AppEvent::FileTreeChildrenLoaded {
                parent: root.clone(),
                children: vec![
                    DirectoryEntry {
                        path: root.join("src"),
                        name: "src".to_string(),
                        is_dir: true,
                    },
                    DirectoryEntry {
                        path: root.join("README.md"),
                        name: "README.md".to_string(),
                        is_dir: false,
                    },
                ],
                error: None,
            },
        );
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeMoveSelection(1)),
        );
        assert_eq!(state.file_tree.selected, Some(root.join("src")));
    }

    #[test]
    fn preview_file_emits_load_side_effect_and_switches_tab() {
        let mut state = test_state();
        let path = PathBuf::from("src/main.rs");
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: None,
            }),
        );
        assert!(effects.contains(&SideEffect::LoadPreviewFile(path)));
        assert_eq!(state.navigation.main_tab, MainTab::Preview);
        assert_eq!(state.navigation.focus, FocusTarget::Main);
        assert!(state.preview.loading);
    }

    #[test]
    fn preview_loaded_applies_content() {
        let mut state = test_state();
        let path = PathBuf::from("README.md");
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: None,
            }),
        );
        reduce(
            &mut state,
            AppEvent::PreviewLoaded {
                path: path.clone(),
                result: PreviewLoadResult {
                    lines: vec!["hello".to_string()],
                    truncated: false,
                    oversize: false,
                    binary: false,
                    lossy_utf8: false,
                    file_size: 5,
                    modified_at: None,
                    error: None,
                },
            },
        );
        assert!(!state.preview.loading);
        assert_eq!(state.preview.lines, vec!["hello".to_string()]);
    }

    #[test]
    fn preview_file_scrolls_to_requested_line() {
        let mut state = test_state();
        let path = PathBuf::from("/tmp/repo/src/main.rs");
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: Some(25),
            }),
        );
        reduce(
            &mut state,
            AppEvent::PreviewLoaded {
                path: path.clone(),
                result: PreviewLoadResult {
                    lines: (1..=50).map(|index| format!("line {index}")).collect(),
                    truncated: false,
                    oversize: false,
                    binary: false,
                    lossy_utf8: false,
                    file_size: 1,
                    modified_at: None,
                    error: None,
                },
            },
        );
        assert_eq!(state.preview.scroll_offset, 24);
        assert_eq!(state.navigation.main_tab, MainTab::Preview);
    }

    #[test]
    fn search_selection_preview_command_includes_content_line() {
        let mut state = test_state();
        let path = PathBuf::from("src/main.rs");
        state.search.results = vec![SearchResult::content(
            path.clone(),
            12,
            "fn main()".to_string(),
        )];
        state.search.selected = 0;

        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: Some(12),
            }),
        );

        assert!(effects.contains(&SideEffect::LoadPreviewFile(path)));
        assert_eq!(state.navigation.main_tab, MainTab::Preview);
    }

    #[test]
    fn preview_scroll_clamps_within_file() {
        let mut state = test_state();
        state.preview.lines = (0..100).map(|index| format!("line {index}")).collect();
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewScroll(200)),
        );
        assert!(state.preview.scroll_offset > 0);
        reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewScroll(-500)),
        );
        assert_eq!(state.preview.scroll_offset, 0);
    }

    #[test]
    fn search_set_query_schedules_debounce_and_cancels() {
        let mut state = test_state();
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::SearchSetQuery("main".to_string())),
        );
        assert!(effects.contains(&SideEffect::CancelSearch));
        assert!(state.search.debounce_scheduled);
        assert_eq!(state.search.generation, 1);
    }

    #[test]
    fn search_execute_emits_run_side_effect() {
        let mut state = test_state();
        state.search.query = "main".to_string();
        state.search.generation = 1;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::SearchExecute));
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::RunSearch {
                mode: SearchMode::Files,
                query,
                generation: 1,
            } if query == "main"
        )));
        assert!(state.search.running);
    }

    #[test]
    fn search_completed_ignores_stale_generation() {
        let mut state = test_state();
        state.search.generation = 2;
        reduce(
            &mut state,
            AppEvent::SearchCompleted {
                generation: 1,
                results: vec![],
                truncated: false,
                error: None,
            },
        );
        assert!(state.search.results.is_empty());
    }

    #[test]
    fn search_clear_cancels_running_query() {
        let mut state = test_state();
        state.search.query = "abc".to_string();
        state.search.running = true;
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::SearchClear));
        assert!(effects.contains(&SideEffect::CancelSearch));
        assert!(state.search.query.is_empty());
        assert!(!state.search.running);
    }

    #[test]
    fn open_editor_emits_launch_side_effect() {
        let mut state = test_state();
        let path = PathBuf::from("src/main.rs");
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::OpenEditor {
                path: path.clone(),
                line: None,
            }),
        );
        assert!(effects.contains(&SideEffect::LaunchEditor { path, line: None }));
    }

    #[test]
    fn editor_launched_records_log_entry() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::EditorLaunched {
                path: PathBuf::from("/tmp/a.rs"),
                command: "nvim".to_string(),
            },
        );
        assert_eq!(state.logs.entries.len(), 1);
        assert!(state.logs.entries[0].message.contains("nvim"));
    }

    #[test]
    fn editor_launch_failed_shows_modal_for_missing_command() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::EditorLaunchFailed {
                path: PathBuf::from("/tmp/a.rs"),
                error: "Editor command not found: missing".to_string(),
                show_modal: true,
            },
        );
        assert!(state.notifications.modal.is_some());
        assert_eq!(state.logs.entries.len(), 1);
    }

    #[test]
    fn modal_dismiss_clears_modal_state() {
        let mut state = test_state();
        state.notifications.show_modal("Title", "Message");
        reduce(&mut state, AppEvent::Command(AppCommand::ModalDismiss));
        assert!(state.notifications.modal.is_none());
    }

    #[test]
    fn clipboard_paste_into_agent_emits_write_side_effect() {
        let mut state = test_state();
        state.agent.running = true;
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Agent));
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Main));
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PasteText("hello".to_string())),
        );
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::WriteAgent(bytes) if bytes == b"hello"
            )
        }));
    }

    #[test]
    fn clipboard_paste_multiline_into_agent_uses_bracketed_paste() {
        let mut state = test_state();
        state.agent.running = true;
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Agent));
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Main));
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PasteText("line\n".to_string())),
        );
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::WriteAgent(bytes) if bytes.starts_with(b"\x1b[200~")
            )
        }));
    }

    #[test]
    fn clipboard_copy_prefers_mouse_selection() {
        use crate::selection::{SelectionPane, TextPosition, TextSelection};

        let mut state = test_state();
        state.preview.lines = vec!["selected text".to_string()];
        state.text_selection = TextSelection {
            pane: Some(SelectionPane::Preview),
            anchor: TextPosition { line: 0, col: 0 },
            cursor: TextPosition { line: 0, col: 8 },
            dragging: false,
        };
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));

        let effects = reduce(&mut state, AppEvent::Command(AppCommand::ClipboardCopy));
        assert!(effects.contains(&SideEffect::CopyToClipboard("selected".to_string())));
    }

    #[test]
    fn clipboard_copy_issue_detail_selection() {
        use crate::github::{IssueDetail, IssueState};
        use crate::selection::{SelectionPane, TextPosition, TextSelection};

        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));
        state.github.issue_detail = Some(IssueDetail {
            number: 42,
            title: "Bug".to_string(),
            state: IssueState::Open,
            author: "dev".to_string(),
            labels: vec![],
            assignees: vec![],
            display_lines: vec!["copy this text".to_string()],
        });
        state.text_selection = TextSelection {
            pane: Some(SelectionPane::IssueDetail),
            anchor: TextPosition { line: 0, col: 5 },
            cursor: TextPosition { line: 0, col: 9 },
            dragging: false,
        };

        let effects = reduce(&mut state, AppEvent::Command(AppCommand::ClipboardCopy));
        assert!(effects.contains(&SideEffect::CopyToClipboard("this".to_string())));
    }

    #[test]
    fn clipboard_copy_emits_side_effect_for_preview_line() {
        let mut state = test_state();
        state.preview.lines = vec!["line one".to_string()];
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));
        let effects = reduce(&mut state, AppEvent::Command(AppCommand::ClipboardCopy));
        assert!(effects.contains(&SideEffect::CopyToClipboard("line one".to_string())));
    }

    #[test]
    fn clipboard_paste_into_search_query_schedules_search() {
        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Left));
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Search));
        let effects = reduce(
            &mut state,
            AppEvent::Command(AppCommand::PasteText("main".to_string())),
        );
        assert_eq!(state.search.query, "main");
        assert!(state.search.debounce_scheduled);
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, SideEffect::CancelSearch)));
    }

    #[test]
    fn fs_changed_invalidates_expanded_file_tree_directory() {
        let root = PathBuf::from("/repo");
        let mut state = test_state();
        state.repo_root = root.clone();
        state.file_tree = FileTreeState::at_root(root.clone());
        let _ = state.file_tree.expand(&root);
        state.file_tree.apply_children_loaded(
            &root,
            vec![DirectoryEntry {
                path: root.join("src"),
                name: "src".to_string(),
                is_dir: true,
            }],
            None,
        );
        state
            .file_tree
            .nodes
            .get_mut(&root.join("src"))
            .expect("src")
            .expanded = true;
        state.file_tree.apply_children_loaded(
            &root.join("src"),
            vec![DirectoryEntry {
                path: root.join("src/main.rs"),
                name: "main.rs".to_string(),
                is_dir: false,
            }],
            None,
        );
        state.file_tree.select(root.join("src/main.rs"));

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![root.join("src/new.rs")],
            },
        );

        assert!(effects.contains(&SideEffect::LoadDirectoryChildren(root.join("src"))));
        assert!(!state.file_tree.nodes[&root.join("src")].children_loaded);
        assert_eq!(state.file_tree.selected, Some(root.join("src/main.rs")));
    }

    #[test]
    fn fs_changed_reloads_matching_preview_file() {
        let mut state = test_state();
        let path = PathBuf::from("/tmp/repo/src/main.rs");
        state.preview.path = Some(path.clone());
        state.preview.lines = vec!["old".to_string()];
        state.preview.scroll_offset = 5;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/other.rs"), path.clone()],
            },
        );

        assert!(effects.contains(&SideEffect::LoadPreviewFile(path)));
        assert!(state.preview.loading);
        assert!(state.preview.preserve_scroll_on_load);
        assert_eq!(state.preview.scroll_offset, 5);
        assert_eq!(state.navigation.main_tab, MainTab::Agent);
    }

    #[test]
    fn fs_changed_skips_reload_when_mtime_unchanged() {
        use std::fs;
        use std::time::SystemTime;

        let temp = std::env::temp_dir().join(format!("kiwi-fs-changed-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let file = temp.join("main.rs");
        fs::write(&file, "unchanged").expect("write");
        let mtime = fs::metadata(&file).expect("metadata").modified().ok();

        let mut state = test_state();
        state.preview.path = Some(file.clone());
        state.preview.lines = vec!["unchanged".to_string()];
        state.preview.loaded_mtime = mtime;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![file.clone()],
            },
        );

        assert!(effects.is_empty());
        assert!(!state.preview.loading);

        fs::write(&file, "changed").expect("rewrite");
        // Ensure mtime advances on platforms with coarse timestamps.
        if let Ok(updated) = fs::metadata(&file).and_then(|metadata| metadata.modified()) {
            if updated == mtime.unwrap_or(SystemTime::UNIX_EPOCH) {
                std::thread::sleep(std::time::Duration::from_millis(1100));
            }
        }

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![file.clone()],
            },
        );

        assert!(effects.contains(&SideEffect::LoadPreviewFile(file)));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn fs_changed_ignores_unrelated_paths_and_in_flight_loads() {
        let mut state = test_state();
        let path = PathBuf::from("/tmp/repo/src/main.rs");
        state.preview.path = Some(path.clone());
        state.preview.loading = true;

        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/other.rs")],
            },
        );
        assert!(effects.is_empty());

        state.preview.loading = false;
        let effects = reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/other.rs")],
            },
        );
        assert!(effects.is_empty());
    }
}
