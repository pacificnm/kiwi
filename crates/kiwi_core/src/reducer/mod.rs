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
    format_issue_agent_prompt, format_pr_agent_prompt, issue_move_selection, issue_select_row,
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

pub fn reduce(state: &mut ReduceView<'_>, event: AppEvent) -> Vec<SideEffect> {
    match event {
        AppEvent::Command(command) => reduce_command(state, command),
        AppEvent::TerminalResize { .. } => Vec::new(),
        AppEvent::GitRefreshRequested => reduce_git_refresh_requested(state),
        AppEvent::GitStatusUpdated {
            branch,
            remote_repo,
            ahead,
            behind,
            file_entries,
            error,
        } => reduce_git_status_updated(
            state,
            branch,
            remote_repo,
            ahead,
            behind,
            file_entries,
            error,
        ),
        AppEvent::Quit => {
            state.set_dirty();
            vec![SideEffect::Quit]
        }
        AppEvent::ShellOutput(data) => reduce_shell_output(state, data),
        AppEvent::ShellExited(_code) => reduce_shell_exited(state),
        AppEvent::AgentOutput { agent_id, data } => reduce_agent_output(state, agent_id, data),
        AppEvent::AgentExited { agent_id, code } => reduce_agent_exited(state, agent_id, code),
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
        AppEvent::GitHubPrMergeCompleted { number, result } => {
            reduce_github_pr_merge_completed(state, number, result)
        }
        AppEvent::BranchListLoaded { entries, error } => {
            reduce_branch_list_loaded(state, entries, error)
        }
        AppEvent::BranchCheckoutCompleted { branch_name, error } => {
            reduce_branch_checkout_completed(state, branch_name, error)
        }
    }
}

pub fn reduce_command(state: &mut ReduceView<'_>, command: AppCommand) -> Vec<SideEffect> {
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
            state.set_dirty();
            if state.config.workspace.persist {
                vec![SideEffect::SaveWorkspace, SideEffect::Quit]
            } else {
                vec![SideEffect::Quit]
            }
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
        AppCommand::GitHubContextMenuOpen {
            anchor_x,
            anchor_y,
            target,
        } => reduce_github_context_menu_open(state, anchor_x, anchor_y, target),
        AppCommand::GitHubContextMenuMove(delta) => reduce_github_context_menu_move(state, delta),
        AppCommand::GitHubContextMenuExecute => reduce_github_context_menu_execute(state),
        AppCommand::GitHubContextMenuSelect(index) => {
            reduce_github_context_menu_select(state, index)
        }
        AppCommand::GitHubContextMenuCancel => reduce_github_context_menu_cancel(state),
        AppCommand::GitHubOpenInBrowser => reduce_github_open_in_browser(state),
        AppCommand::ShellWrite(data) => vec![SideEffect::WriteShell(data)],
        AppCommand::ShellScroll(delta) => reduce_shell_scroll(state, delta),
        AppCommand::ShellScrollLines(lines) => reduce_shell_scroll_lines(state, lines),
        AppCommand::AgentWrite(data) => vec![SideEffect::WriteAgent(data)],
        AppCommand::AgentScroll(delta) => reduce_agent_scroll(state, delta),
        AppCommand::AgentScrollLines(lines) => reduce_agent_scroll_lines(state, lines),
        AppCommand::AgentRestart => reduce_agent_restart(state),
        AppCommand::AgentNew => reduce_agent_new(state),
        AppCommand::AgentCycleNext => reduce_agent_cycle(state, 1),
        AppCommand::AgentCyclePrev => reduce_agent_cycle(state, -1),
        AppCommand::AgentSetActive(id) => reduce_agent_set_active(state, id),
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
        AppCommand::ClipboardCopy
        | AppCommand::ClipboardCut
        | AppCommand::ClipboardPaste
        | AppCommand::PasteText(_)
        | AppCommand::SelectionBegin { .. }
        | AppCommand::SelectionExtend { .. }
        | AppCommand::SelectionEnd
        | AppCommand::SelectionClear => Vec::new(),
        AppCommand::SettingsMoveSelection(delta) => reduce_settings_move_selection(state, delta),
        AppCommand::SettingsSelect(index) => reduce_settings_select(state, index),
        AppCommand::SettingsApplyTheme => reduce_settings_apply_theme(state),
        AppCommand::SetTheme(name) => reduce_set_theme(state, name),
    }
}

pub fn apply_navigation(state: &mut ReduceView<'_>, command: NavCommand) {
    let before = state.navigation.clone();
    state.navigation.apply(command);
    if *state.navigation != before {
        state.set_dirty();
    }
    if state.navigation.left_tab == LeftNavTab::Gh
        || matches!(state.navigation.main_tab, MainTab::Issues | MainTab::Prs)
    {
        sync_github_left_pane_from_main_tab(state);
    }
    if state.navigation.left_tab == LeftNavTab::Files {
        state.file_tree.ensure_selection();
    }
    if state.navigation.left_tab == LeftNavTab::Git {
        ensure_git_selection(state.git, state.config.git.show_untracked);
    }
    if state.navigation.main_tab == MainTab::Branches {
        ensure_branch_selection(state.branches);
    }
    if state.navigation.main_tab == MainTab::Settings && before.main_tab != MainTab::Settings {
        let viewport_rows = state.viewport.settings_rows;
        ensure_settings_selection(state.settings, &state.config.theme.name, viewport_rows);
    }
}

fn sync_github_left_pane_from_main_tab(state: &mut ReduceView<'_>) {
    state.github.left_pane = match state.navigation.main_tab {
        MainTab::Issues => GitHubLeftPane::Issues,
        MainTab::Prs => GitHubLeftPane::Prs,
        _ => state.github.left_pane,
    };
}

pub fn agent_spawn_effects_if_needed(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    agent_spawn_effects_for(state, state.agent_manager.active_id())
}

pub fn agent_new_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    reduce_agent_new(state)
}

pub fn agent_cycle_effects(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    reduce_agent_cycle(state, delta)
}

fn agent_spawn_effects_for(
    state: &mut ReduceView<'_>,
    id: crate::agent::AgentId,
) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    let needs_spawn = state.agent_manager.pty(id).is_some_and(|pty| !pty.spawned);
    if !needs_spawn {
        return Vec::new();
    }

    state.set_dirty();
    vec![SideEffect::SpawnAgent(id)]
}

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
                effects.push(SideEffect::LoadDirectoryChildren(path));
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
    effects.extend(github_issue_list_access_effects(state, false));
    effects.extend(github_pr_list_access_effects(state, false));
    effects.extend(branch_list_access_effects(state, false));
    effects
}

fn workspace_apply_pending_selection(state: &mut ReduceView<'_>) {
    let Some(path) = state.workspace_meta.pending_selected_path.clone() else {
        return;
    };

    if state.file_tree.nodes.contains_key(&path) {
        state.file_tree.select(path);
        state.workspace_meta.pending_selected_path = None;
        state.set_dirty();
    }
}

pub fn git_refresh_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.workspace_meta.is_git_repo || state.git.loading {
        return Vec::new();
    }

    state.set_dirty();
    state.git.loading = true;
    vec![SideEffect::SpawnGitRefresh]
}

pub fn branch_list_access_effects(state: &mut ReduceView<'_>, force: bool) -> Vec<SideEffect> {
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
    state.set_dirty();
    vec![SideEffect::SpawnBranchList]
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
    vec![SideEffect::SpawnBranchCheckout { name: branch_name }]
}

pub fn github_refresh_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.set_dirty();
    state.github.loading = true;
    state.github.auth_checked = false;
    state.github.issues_loaded_at = None;
    state.github.prs_loaded_at = None;
    clear_issue_detail_cache(state.github);
    clear_pr_detail_cache(state.github);
    vec![SideEffect::SpawnGitHubAuthCheck]
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
    vec![SideEffect::SpawnGitHubAuthCheck]
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
    vec![SideEffect::SpawnGitHubIssueList]
}

pub fn github_issue_list_access_effects(
    state: &mut ReduceView<'_>,
    force: bool,
) -> Vec<SideEffect> {
    github_issue_list_effects(state, force)
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
    vec![SideEffect::SpawnGitHubPrList]
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

pub fn github_pr_detail_access_effects(state: &mut ReduceView<'_>, force: bool) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Prs {
        return Vec::new();
    }

    let Some(number) = selected_pr_number(state.github) else {
        return Vec::new();
    };

    github_pr_detail_effects(state, number, force)
}

fn selected_pr_number(github: &crate::state::GitHubState) -> Option<u32> {
    github
        .selected_pr
        .and_then(|number| u32::try_from(number).ok())
}

fn github_pr_detail_effects(
    state: &mut ReduceView<'_>,
    number: u32,
    force: bool,
) -> Vec<SideEffect> {
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
    state.set_dirty();

    vec![SideEffect::SpawnGitHubPrDetail { number }]
}

fn selected_issue_number(github: &crate::state::GitHubState) -> Option<u32> {
    github
        .selected_issue
        .and_then(|number| u32::try_from(number).ok())
}

fn github_issue_detail_effects(
    state: &mut ReduceView<'_>,
    number: u32,
    force: bool,
) -> Vec<SideEffect> {
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
    state.set_dirty();

    vec![SideEffect::SpawnGitHubIssueDetail { number }]
}

fn github_surface_active(state: &ReduceView<'_>) -> bool {
    state.navigation.left_tab == LeftNavTab::Gh
        || matches!(state.navigation.main_tab, MainTab::Issues | MainTab::Prs)
}

fn github_issue_list_surface_active(state: &ReduceView<'_>) -> bool {
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

fn github_pr_list_surface_active(state: &ReduceView<'_>) -> bool {
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

fn reduce_github_refresh_requested(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    github_refresh_effects(state)
}

fn reduce_github_auth_checked(
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

fn reduce_github_issues_loaded(
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

    if state.navigation.main_tab == MainTab::Prs {
        if let Some(number) = selected_pr_number(state.github) {
            return github_pr_detail_effects(state, number, true);
        }
    }

    Vec::new()
}

fn reduce_github_prs_loaded(
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

fn reduce_github_issue_detail_loaded(
    state: &mut ReduceView<'_>,
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
        clamp_issue_detail_scroll(state);
    }
    state.set_dirty();
    Vec::new()
}

fn reduce_github_pr_detail_loaded(
    state: &mut ReduceView<'_>,
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
        clamp_pr_detail_scroll(state);
    }
    state.set_dirty();
    Vec::new()
}

fn reduce_github_move_issue_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = issues_viewport_rows(state);
    issue_move_selection(state.github, delta, viewport_rows);
    state.set_dirty();
    Vec::new()
}

fn reduce_github_select_issue(state: &mut ReduceView<'_>, row_index: usize) -> Vec<SideEffect> {
    let viewport_rows = issues_viewport_rows(state);
    issue_select_row(state.github, row_index, viewport_rows);
    state.set_dirty();
    Vec::new()
}

fn issues_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.github_list_rows
}

fn reduce_github_move_pr_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = prs_viewport_rows(state);
    pr_move_selection(state.github, delta, viewport_rows);
    state.set_dirty();
    Vec::new()
}

fn reduce_github_select_pr(state: &mut ReduceView<'_>, row_index: usize) -> Vec<SideEffect> {
    let viewport_rows = prs_viewport_rows(state);
    pr_select_row(state.github, row_index, viewport_rows);
    state.set_dirty();
    Vec::new()
}

fn prs_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.github_list_rows
}

fn reduce_github_open_selected(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    match state.github.left_pane {
        GitHubLeftPane::Issues => {
            let Some(number) = selected_issue_number(state.github) else {
                return Vec::new();
            };

            state.github.left_pane = GitHubLeftPane::Issues;
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

            state.github.left_pane = GitHubLeftPane::Prs;
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

fn reduce_github_select_left_pane(
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

fn reduce_github_issue_detail_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
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

fn reduce_github_issue_detail_page_scroll(
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

fn issue_detail_line_count(github: &crate::state::GitHubState) -> usize {
    github
        .issue_detail
        .as_ref()
        .map(|detail| detail.display_lines.len())
        .unwrap_or(0)
}

fn clamp_issue_detail_scroll(state: &mut ReduceView<'_>) {
    let line_count = issue_detail_line_count(state.github);
    let viewport_rows = issue_detail_viewport_rows(state);
    let max_offset = line_count.saturating_sub(viewport_rows);
    if state.github.issue_detail_scroll_offset > max_offset {
        state.github.issue_detail_scroll_offset = max_offset;
    }
}

fn issue_detail_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.github_detail_rows
}

fn reduce_github_pr_detail_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
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

fn reduce_github_pr_detail_page_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
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

fn pr_detail_line_count(github: &crate::state::GitHubState) -> usize {
    github
        .pr_detail
        .as_ref()
        .map(|detail| detail.display_lines.len())
        .unwrap_or(0)
}

fn clamp_pr_detail_scroll(state: &mut ReduceView<'_>) {
    let line_count = pr_detail_line_count(state.github);
    let viewport_rows = pr_detail_viewport_rows(state);
    let max_offset = line_count.saturating_sub(viewport_rows);
    if state.github.pr_detail_scroll_offset > max_offset {
        state.github.pr_detail_scroll_offset = max_offset;
    }
}

fn pr_detail_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.github_detail_rows
}

fn reduce_github_issue_comment_completed(
    state: &mut ReduceView<'_>,
    number: u32,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if state.github.selected_issue != Some(u64::from(number)) {
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

fn reduce_github_issue_create_branch_completed(
    state: &mut ReduceView<'_>,
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
        state.set_dirty();
        return git_refresh_effects(state);
    }

    state.github.issue_action_message = result.error.clone();
    state.set_dirty();
    Vec::new()
}

fn reduce_github_repo_labels_loaded(
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

fn reduce_github_issue_labels_applied(
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

fn reduce_github_label_picker_move(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if let Some(picker) = state.github.label_picker.as_mut() {
        picker.move_cursor(delta);
        state.set_dirty();
    }
    Vec::new()
}

fn reduce_github_label_picker_toggle(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if let Some(picker) = state.github.label_picker.as_mut() {
        if !picker.loading && !picker.applying {
            picker.toggle_cursor();
            state.set_dirty();
        }
    }
    Vec::new()
}

fn reduce_github_label_picker_apply(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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
    vec![SideEffect::SpawnGitHubIssueLabelApply { number, labels }]
}

fn reduce_github_label_picker_cancel(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.github.label_picker.is_some() {
        state.github.label_picker = None;
        state.set_dirty();
    }
    Vec::new()
}

fn reduce_github_context_menu_open(
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

fn reduce_github_context_menu_move(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if let Some(menu) = state.github.context_menu.as_mut() {
        menu.move_cursor(delta);
        state.set_dirty();
    }
    Vec::new()
}

fn reduce_github_context_menu_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    if let Some(menu) = state.github.context_menu.as_mut() {
        if index < menu.items.len() {
            menu.cursor = index;
            state.set_dirty();
        }
    }
    reduce_github_context_menu_execute(state)
}

fn reduce_github_context_menu_execute(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(menu) = state.github.context_menu.take() else {
        return Vec::new();
    };

    let Some(action) = menu.selected_action() else {
        state.set_dirty();
        return Vec::new();
    };

    match action {
        GhContextMenuAction::View => match menu.target {
            GhContextTarget::Issue { list_index } => {
                let viewport_rows = issues_viewport_rows(state);
                issue_select_row(state.github, list_index, viewport_rows);
                reduce_github_open_selected(state)
            }
            GhContextTarget::PullRequest { list_index } => {
                let viewport_rows = prs_viewport_rows(state);
                pr_select_row(state.github, list_index, viewport_rows);
                reduce_github_open_selected(state)
            }
        },
        GhContextMenuAction::CreateBranch => {
            github_issue_create_branch_effects(state, selected_issue_number(state.github))
        }
        GhContextMenuAction::Comment => github_issue_comment_prompt_effects(state),
        GhContextMenuAction::AddLabels => github_issue_label_picker_effects(state),
        GhContextMenuAction::Merge => github_pr_merge_effects(state),
        GhContextMenuAction::OpenInBrowser => github_open_in_browser_effects(state),
        GhContextMenuAction::SendToAgent => match menu.target {
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

fn reduce_github_context_menu_cancel(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.github.context_menu.is_some() {
        state.github.context_menu = None;
        state.set_dirty();
    }
    Vec::new()
}

fn github_issue_create_branch_effects(
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
    vec![SideEffect::SpawnGitHubIssueCreateBranch { number }]
}

fn github_issue_comment_prompt_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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

fn github_issue_label_picker_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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
    vec![SideEffect::SpawnGitHubRepoLabels]
}

fn github_open_in_browser_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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
    vec![SideEffect::SpawnGitHubOpenBrowser { target }]
}

fn github_pr_merge_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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
    vec![SideEffect::SpawnGitHubPrMerge { number }]
}

fn issue_labels_for_number(github: &crate::state::GitHubState, number: u32) -> Vec<String> {
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

fn github_send_issue_to_agent_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(number) = selected_issue_number(state.github) else {
        state
            .notifications
            .show_toast("Select an issue in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    let title = issue_title_for_number(state.github, number)
        .unwrap_or_else(|| "Untitled issue".to_string());
    let prompt = format_issue_agent_prompt(number, &title);
    github_send_prompt_to_agent_effects(state, prompt)
}

fn github_send_pr_to_agent_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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

fn github_send_prompt_to_agent_effects(
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
    effects.push(SideEffect::WriteAgent(prompt.into_bytes()));
    effects
}

fn issue_title_for_number(github: &crate::state::GitHubState, number: u32) -> Option<String> {
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

fn pr_title_for_number(github: &crate::state::GitHubState, number: u32) -> Option<String> {
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

fn reduce_github_open_in_browser(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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
    vec![SideEffect::SpawnGitHubOpenBrowser { target }]
}

fn reduce_github_open_browser_completed(
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

fn reduce_github_pr_create_completed(
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

fn reduce_github_pr_merge_completed(
    state: &mut ReduceView<'_>,
    number: u32,
    result: crate::github::IssueActionResult,
) -> Vec<SideEffect> {
    if state.github.selected_pr != Some(u64::from(number)) {
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

fn reduce_git_refresh_requested(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    git_refresh_effects(state)
}

fn reduce_git_status_updated(
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
        ensure_git_selection(state.git, state.config.git.show_untracked);
        clamp_git_scroll(
            state.git,
            git_viewport_rows(state).max(1),
            state.config.git.show_untracked,
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

fn reduce_branch_refresh(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    branch_list_access_effects(state, true)
}

fn reduce_branch_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = state.viewport.branches_rows;
    branch_move_selection(state.branches, delta, viewport_rows);
    state.set_dirty();
    Vec::new()
}

fn reduce_branch_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    let viewport_rows = state.viewport.branches_rows;
    branch_select_row(state.branches, index, viewport_rows);
    state.set_dirty();
    Vec::new()
}

fn reduce_branch_checkout_selected(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(branch_name) = branch_selected_name(state.branches).map(str::to_string) else {
        return Vec::new();
    };

    branch_checkout_effects(state, branch_name)
}

fn reduce_branch_list_loaded(
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

fn reduce_branch_checkout_completed(
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

fn sync_git_status_patch_to_file_tree(
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

fn sync_git_statuses_to_file_tree(state: &mut ReduceView<'_>) {
    let entries = state.git.file_entries.clone();
    state
        .file_tree
        .apply_git_statuses(state.repo_root, &entries, state.config.git.show_untracked);
}

fn reduce_shell_output(state: &mut ReduceView<'_>, data: Vec<u8>) -> Vec<SideEffect> {
    state.shell.scrollback.set_cols(state.shell.cols);
    state.shell.scrollback.append_bytes(&data);
    state.set_dirty();
    Vec::new()
}

fn reduce_shell_exited(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.shell.running = false;
    state.set_dirty();
    Vec::new()
}

fn reduce_agent_output(
    state: &mut ReduceView<'_>,
    agent_id: crate::agent::AgentId,
    data: Vec<u8>,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };

    let cols = pty.cols;
    pty.scrollback.set_cols(cols);
    pty.scrollback.append_bytes(&data);
    if let Some(status) = infer_status_from_scrollback(&pty.scrollback) {
        pty.status = status;
    }
    state.set_dirty();
    Vec::new()
}

fn reduce_agent_exited(
    state: &mut ReduceView<'_>,
    agent_id: crate::agent::AgentId,
    code: i32,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };

    pty.apply_exit(code);
    state.set_dirty();
    Vec::new()
}

fn reduce_agent_new(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    let linked_issue = state.github.selected_issue;
    match state.agent_manager.create_agent(None, linked_issue) {
        Ok(id) => {
            state.set_dirty();
            vec![SideEffect::SpawnAgent(id)]
        }
        Err(crate::agent::AgentManagerError::AtCapacity) => {
            state
                .notifications
                .show_toast("Agent limit reached (max 3 sessions)");
            Vec::new()
        }
        Err(_) => Vec::new(),
    }
}

fn reduce_agent_set_active(
    state: &mut ReduceView<'_>,
    id: crate::agent::AgentId,
) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    if state.agent_manager.set_active(id).is_err() {
        return Vec::new();
    }

    state.set_dirty();
    agent_spawn_effects_for(state, id)
}

fn reduce_agent_cycle(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    if state.agent_manager.session_count() <= 1 {
        return Vec::new();
    }

    let id = state.agent_manager.cycle_active(delta);
    state.set_dirty();
    agent_spawn_effects_for(state, id)
}

fn reduce_agent_restart(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    state.set_dirty();
    vec![SideEffect::RestartAgent(state.agent_manager.active_id())]
}

fn reduce_shell_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.shell_rows;
    state.shell.scroll_by(delta, page_size);
    state.set_dirty();
    Vec::new()
}

fn reduce_shell_scroll_lines(state: &mut ReduceView<'_>, lines: i32) -> Vec<SideEffect> {
    if lines == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.shell_rows;
    state.shell.scroll_by_lines(lines, page_size);
    state.set_dirty();
    Vec::new()
}

fn reduce_agent_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.agent_rows;
    state.active_agent_mut().scroll_by(delta, page_size);
    state.set_dirty();
    Vec::new()
}

fn reduce_agent_scroll_lines(state: &mut ReduceView<'_>, lines: i32) -> Vec<SideEffect> {
    if lines == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.agent_rows;
    state.active_agent_mut().scroll_by_lines(lines, page_size);
    state.set_dirty();
    Vec::new()
}

fn reduce_palette_open(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.palette.open_with_focus(state.navigation.focus);
    state.navigation.focus = FocusTarget::CommandPalette;
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

fn reduce_palette_close(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.navigation.focus = state.palette.close();
    state.set_dirty();
    Vec::new()
}

fn reduce_palette_append_char(state: &mut ReduceView<'_>, ch: char) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_cursor = None;
    state.palette.input.push(ch);
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

fn reduce_palette_backspace(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_cursor = None;
    state.palette.input.pop();
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

fn reduce_palette_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if !state.palette.open || state.palette.prompt.is_some() {
        return Vec::new();
    }

    state.palette.move_selection(delta as isize);
    state.set_dirty();
    Vec::new()
}

fn reduce_palette_history_up(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    if let Some(command_id) = state.palette.history_up() {
        state.palette.input = history_input_for_id(state, &command_id);
    }
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

fn reduce_palette_history_down(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    match state.palette.history_down() {
        None => {}
        Some(None) => state.palette.input.clear(),
        Some(Some(command_id)) => state.palette.input = history_input_for_id(state, &command_id),
    }
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

fn reduce_palette_execute_selected(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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

fn reduce_palette_prompt_submit(
    state: &mut ReduceView<'_>,
    prompt: PalettePrompt,
) -> Vec<SideEffect> {
    match prompt {
        PalettePrompt::GitHubIssueComment { number } => {
            let body = state.palette.input.trim().to_string();
            if body.is_empty() {
                state.notifications.show_toast("Comment cannot be empty");
                state.set_dirty();
                return Vec::new();
            }

            state.navigation.focus = state.palette.close();
            state.set_dirty();
            vec![SideEffect::SpawnGitHubIssueComment { number, body }]
        }
        PalettePrompt::GitHubPrCreate(prompt) => {
            let input = state.palette.input.clone();
            match advance_pr_create_prompt(prompt, &input) {
                Ok(PrCreatePromptAdvance::Continue(next)) => {
                    state.palette.input.clear();
                    state.palette.prompt = Some(PalettePrompt::GitHubPrCreate(next));
                    state.set_dirty();
                    Vec::new()
                }
                Ok(PrCreatePromptAdvance::Submit(request)) => {
                    state.navigation.focus = state.palette.close();
                    state.set_dirty();
                    vec![SideEffect::SpawnGitHubPrCreate { request }]
                }
                Err(message) => {
                    state.notifications.show_toast(message);
                    state.set_dirty();
                    Vec::new()
                }
            }
        }
    }
}

fn reduce_palette_execute_match(state: &mut ReduceView<'_>, match_index: usize) -> Vec<SideEffect> {
    if !state.palette.open || state.palette.prompt.is_some() {
        return Vec::new();
    }

    let Some(registry_index) = state.palette.matches.get(match_index).copied() else {
        return Vec::new();
    };

    execute_command(state, registry_index)
}

fn reduce_file_tree_expand(state: &mut ReduceView<'_>, path: PathBuf) -> Vec<SideEffect> {
    match state.file_tree.expand(&path) {
        Ok(ExpandAction::NeedsLoad) => {
            state.set_dirty();
            vec![SideEffect::LoadDirectoryChildren(path)]
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

fn reduce_file_tree_collapse(state: &mut ReduceView<'_>, path: PathBuf) -> Vec<SideEffect> {
    state.file_tree.collapse(&path);
    state.set_dirty();
    Vec::new()
}

fn reduce_file_tree_select(state: &mut ReduceView<'_>, path: PathBuf) -> Vec<SideEffect> {
    state.file_tree.select(path);
    state.set_dirty();
    Vec::new()
}

fn file_tree_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.file_tree_rows
}

fn reduce_file_tree_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    state
        .file_tree
        .move_selection(delta, file_tree_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

fn git_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.git_rows
}

fn reduce_git_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let viewport_rows = git_viewport_rows(state);
    let show_untracked = state.config.git.show_untracked;
    git_move_selection(state.git, delta, viewport_rows, show_untracked);
    state.set_dirty();
    Vec::new()
}

fn reduce_git_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    let viewport_rows = git_viewport_rows(state);
    let show_untracked = state.config.git.show_untracked;
    git_select_row(state.git, index, viewport_rows, show_untracked);
    state.set_dirty();
    Vec::new()
}

fn reduce_git_open_selected(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(path) = state.git.selected_path.clone() else {
        return Vec::new();
    };

    apply_navigation(state, NavCommand::SelectMainTab(MainTab::Diff));
    state
        .navigation
        .apply(NavCommand::SetFocus(FocusTarget::Main));
    diff_select_file_effects(state, path)
}

fn sync_git_selection_for_path(state: &mut ReduceView<'_>, path: &str) {
    let viewport_rows = git_viewport_rows(state);
    let show_untracked = state.config.git.show_untracked;
    let rows = build_panel_rows(&state.git.file_entries, show_untracked);
    if let Some(row) = row_for_path(&rows, path) {
        git_select_row(state.git, row, viewport_rows, show_untracked);
    } else {
        state.git.selected_path = Some(path.to_string());
    }
}

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

fn reduce_diff_select_file(state: &mut ReduceView<'_>, path: String) -> Vec<SideEffect> {
    diff_select_file_effects(state, path)
}

fn reduce_diff_move_file(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
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

fn reduce_diff_loaded(
    state: &mut ReduceView<'_>,
    result: crate::diff::FileDiffLoadResult,
) -> Vec<SideEffect> {
    state.diff.apply_loaded(result);
    state.set_dirty();
    Vec::new()
}

fn reduce_diff_toggle_source(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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

fn reduce_diff_set_source(
    state: &mut ReduceView<'_>,
    source: crate::diff::DiffSource,
) -> Vec<SideEffect> {
    diff_set_source_effects(state, source)
}

fn reduce_file_tree_refresh(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
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

    state.set_dirty();
    effects
}

fn reduce_file_tree_children_loaded(
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

fn preview_viewport_rows(state: &ReduceView<'_>) -> usize {
    diff_viewport_rows(state)
}

fn diff_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.preview_rows
}

fn diff_text_width(state: &ReduceView<'_>) -> usize {
    let area_width = state.viewport.preview_cols;
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

fn reduce_preview_file(
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
    vec![SideEffect::LoadPreviewFile(path)]
}

fn reduce_preview_loaded(
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

fn reduce_preview_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.preview.scroll(delta, preview_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

fn reduce_preview_page_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state
        .preview
        .page_scroll(delta, preview_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

fn reduce_diff_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.diff.scroll(delta, diff_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

fn reduce_diff_page_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state.diff.page_scroll(delta, diff_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

fn reduce_diff_horizontal_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 || state.config.diff.word_wrap {
        return Vec::new();
    }

    state.diff.scroll_horizontal(delta, diff_text_width(state));
    state.set_dirty();
    Vec::new()
}

fn reduce_search_set_query(state: &mut ReduceView<'_>, query: String) -> Vec<SideEffect> {
    state.search.schedule_query(query);
    state.set_dirty();
    vec![SideEffect::CancelSearch]
}

fn reduce_search_append_char(state: &mut ReduceView<'_>, ch: char) -> Vec<SideEffect> {
    let mut query = state.search.query.clone();
    query.push(ch);
    reduce_search_set_query(state, query)
}

fn reduce_search_backspace(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.search.query.is_empty() {
        return Vec::new();
    }

    let mut query = state.search.query.clone();
    query.pop();
    reduce_search_set_query(state, query)
}

fn reduce_search_clear(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.search.clear_query();
    state.set_dirty();
    vec![SideEffect::CancelSearch]
}

fn reduce_search_set_mode(
    state: &mut ReduceView<'_>,
    mode: crate::search::SearchMode,
) -> Vec<SideEffect> {
    state.search.set_mode(mode);
    state.set_dirty();
    vec![SideEffect::CancelSearch]
}

fn reduce_search_execute(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let generation = state.search.begin_execute();
    state.set_dirty();

    if state.search.query.is_empty() {
        return vec![SideEffect::CancelSearch];
    }

    vec![SideEffect::RunSearch {
        mode: state.search.mode,
        query: state.search.query.clone(),
        generation,
    }]
}

fn reduce_search_cancel(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.search.cancel();
    state.set_dirty();
    vec![SideEffect::CancelSearch]
}

fn search_viewport_rows(state: &ReduceView<'_>) -> usize {
    state.viewport.search_rows
}

fn reduce_search_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    state
        .search
        .move_selection(delta, search_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

fn reduce_search_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    state
        .search
        .select_index(index, search_viewport_rows(state));
    state.set_dirty();
    Vec::new()
}

fn reduce_search_completed(
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

fn reduce_open_editor(
    state: &mut ReduceView<'_>,
    path: PathBuf,
    line: Option<u32>,
) -> Vec<SideEffect> {
    state.set_dirty();
    vec![SideEffect::LaunchEditor { path, line }]
}

fn reduce_modal_dismiss(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.notifications.modal.is_some() {
        state.notifications.dismiss_modal();
        state.set_dirty();
    }
    Vec::new()
}

fn reduce_editor_launched(
    state: &mut ReduceView<'_>,
    path: PathBuf,
    command: String,
) -> Vec<SideEffect> {
    state.logs.push_info(format!(
        "Launched editor `{command}` for {}",
        path.display()
    ));
    state.set_dirty();
    Vec::new()
}

fn reduce_editor_launch_failed(
    state: &mut ReduceView<'_>,
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
    state.set_dirty();
    Vec::new()
}

fn reduce_settings_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = state.viewport.settings_rows;
    settings_move_selection(state.settings, delta, viewport_rows);
    state.set_dirty();
    Vec::new()
}

fn reduce_settings_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    let viewport_rows = state.viewport.settings_rows;
    settings_select_row(state.settings, index, viewport_rows);
    state.set_dirty();
    Vec::new()
}

fn reduce_settings_apply_theme(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(name) = crate::theme::BUILTIN_THEME_NAMES
        .get(state.settings.selected_index)
        .copied()
    else {
        return Vec::new();
    };

    reduce_set_theme(state, name.to_string())
}

fn reduce_set_theme(state: &mut ReduceView<'_>, name: String) -> Vec<SideEffect> {
    if name == state.config.theme.name && state.config.theme.custom.is_none() {
        return Vec::new();
    }

    let settings = ThemeSettings {
        name: name.clone(),
        custom: None,
    };

    let palette = match load_theme_with_capabilities(&settings, *state.terminal_capabilities) {
        Ok(palette) => palette,
        Err(err) => {
            state
                .notifications
                .show_toast(format!("Failed to load theme: {err}"));
            return Vec::new();
        }
    };

    *state.theme = palette;
    state.config.theme = settings;
    state.set_dirty();
    let viewport_rows = state.viewport.settings_rows;
    ensure_settings_selection(state.settings, &name, viewport_rows);

    let mut message = format!("Theme set to {name}");
    if project_has_theme_override(state.repo_root) {
        message.push_str("; saved to user config (project .kiwi.toml may override on restart)");
    } else {
        message.push_str("; saved to user config");
    }
    state.notifications.show_toast(message);

    vec![SideEffect::PersistUserTheme { name }]
}

fn reduce_fs_changed(state: &mut ReduceView<'_>, paths: Vec<PathBuf>) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    let reload_dirs = state
        .file_tree
        .apply_fs_invalidation(state.repo_root, &paths);
    if !reload_dirs.is_empty() {
        state.set_dirty();
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
            state.set_dirty();
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
    use crate::navigation::{FocusTarget, LeftNavTab, MainTab, NavCommand};
    use crate::theme::load_theme_with_capabilities;
    use crate::theme::TerminalCapabilities;

    use super::*;
    use crate::agent::{AgentId, AgentManager};
    use crate::file_tree::{DirectoryEntry, FileTreeState};
    use crate::preview::{PreviewLoadResult, PreviewState};
    use crate::search::{SearchMode, SearchResult, SearchState};
    use crate::state::{AppState, ViewportMetrics};

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("."),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics {
                settings_rows: 10,
                github_list_rows: 10,
                github_detail_rows: 20,
                branches_rows: 10,
                git_rows: 10,
                file_tree_rows: 10,
                preview_rows: 20,
                preview_cols: 80,
                search_rows: 10,
                shell_rows: 20,
                shell_cols: 80,
                agent_rows: 15,
                agent_cols: 100,
            },
        )
    }

    fn run_reduce(state: &mut AppState, event: AppEvent) -> Vec<SideEffect> {
        reduce(&mut ReduceView::from_app_state(state), event)
    }

    #[test]
    fn navigation_command_sets_dirty() {
        let mut state = test_state();
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectLeftTab(
                LeftNavTab::Git,
            ))),
        );
        assert!(state.dirty);
        assert_eq!(state.navigation.left_tab, LeftNavTab::Git);
    }

    #[test]
    fn main_tab_select_pairs_left_tab_for_issues() {
        let mut state = test_state();
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Issues,
            ))),
        );
        assert_eq!(state.navigation.left_tab, LeftNavTab::Gh);
        assert_eq!(state.navigation.main_tab, MainTab::Issues);
        assert_eq!(state.github.left_pane, GitHubLeftPane::Issues);
    }

    #[test]
    fn main_tab_select_pairs_left_tab_for_prs_preview_diff_and_branches() {
        let mut state = test_state();

        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Prs,
            ))),
        );
        assert_eq!(state.navigation.left_tab, LeftNavTab::Gh);
        assert_eq!(state.github.left_pane, GitHubLeftPane::Prs);

        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Preview,
            ))),
        );
        assert_eq!(state.navigation.left_tab, LeftNavTab::Files);

        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Diff,
            ))),
        );
        assert_eq!(state.navigation.left_tab, LeftNavTab::Git);

        state.github.left_pane = GitHubLeftPane::Prs;
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Branches,
            ))),
        );
        assert_eq!(state.navigation.left_tab, LeftNavTab::Gh);
        assert_eq!(state.github.left_pane, GitHubLeftPane::Prs);
    }

    #[test]
    fn main_tab_select_does_not_force_left_tab_for_agent_logs_or_settings() {
        let mut state = test_state();
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectLeftTab(
                LeftNavTab::Git,
            ))),
        );

        for tab in [MainTab::Agent, MainTab::Logs, MainTab::Settings] {
            run_reduce(
                &mut state,
                AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(tab))),
            );
            assert_eq!(state.navigation.left_tab, LeftNavTab::Git);
            assert_eq!(state.navigation.main_tab, tab);
        }
    }

    #[test]
    fn set_theme_updates_runtime_palette_and_emits_persist_effect() {
        let mut state = test_state();
        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::SetTheme("dracula".to_string())),
        );

        assert_eq!(state.config.theme.name, "dracula");
        assert_eq!(state.config.theme.name, "dracula");
        assert_eq!(
            effects,
            vec![SideEffect::PersistUserTheme {
                name: "dracula".to_string()
            }]
        );
    }

    #[test]
    fn settings_tab_syncs_selection_to_active_theme() {
        let mut state = test_state();
        state.config.theme.name = "nord".to_string();

        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Settings,
            ))),
        );

        assert_eq!(state.settings.selected_index, 6);
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

        run_reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                remote_repo: None,
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

        run_reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                remote_repo: None,
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
        run_reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                remote_repo: None,
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
        let effects = run_reduce(&mut state, AppEvent::GitRefreshRequested);
        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn git_refresh_requested_skips_non_git_repo() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = false;
        let effects = run_reduce(&mut state, AppEvent::GitRefreshRequested);
        assert!(effects.is_empty());
        assert!(!state.git.loading);
    }

    #[test]
    fn git_status_updated_sets_branch_and_tracking_counts() {
        let mut state = test_state();
        run_reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: Some("feature/42".to_string()),
                remote_repo: Some("org/repo".to_string()),
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

        run_reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                remote_repo: None,
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
        let effects = run_reduce(
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

        let effects = run_reduce(
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

        let effects = run_reduce(
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

        run_reduce(
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

        let effects = run_reduce(
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

        run_reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: Some("main".to_string()),
                remote_repo: None,
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
        let effects = run_reduce(&mut state, AppEvent::FsChanged { paths });

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

        let effects = run_reduce(
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

        let effects = run_reduce(
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

        let effects = run_reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/main.rs")],
            },
        );

        assert!(!effects.contains(&SideEffect::SpawnGitRefresh));
    }

    #[test]
    fn fs_changed_skips_git_refresh_when_already_loading() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = true;
        state.git.loading = true;

        let effects = run_reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/main.rs")],
            },
        );

        assert!(!effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn git_refresh_skips_when_already_loading() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.git.loading = true;

        let effects = run_reduce(&mut state, AppEvent::GitRefreshRequested);

        assert!(effects.is_empty());
        assert!(state.git.loading);
    }

    #[test]
    fn fs_changed_after_git_refresh_requested_spawns_single_refresh() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.config.git.watch = true;

        let first = run_reduce(&mut state, AppEvent::GitRefreshRequested);
        let second = run_reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/repo/src/main.rs")],
            },
        );

        assert!(first.contains(&SideEffect::SpawnGitRefresh));
        assert!(!second.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn request_git_refresh_sets_loading() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::RequestGitRefresh));

        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(state.git.loading);
    }

    #[test]
    fn git_refresh_command_emits_side_effect() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::GitRefresh));
        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
    }

    #[test]
    fn git_open_selected_switches_to_main_diff_tab() {
        let mut state = test_state();
        state.git.selected_path = Some("src/main.rs".to_string());

        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::GitOpenSelected));

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

        run_reduce(
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

        run_reduce(&mut state, AppEvent::Command(AppCommand::DiffScroll(10)));
        assert_eq!(state.diff.scroll_offset, 10);

        run_reduce(&mut state, AppEvent::Command(AppCommand::DiffScroll(-20)));
        assert_eq!(state.diff.scroll_offset, 0);
    }

    #[test]
    fn diff_toggle_source_switches_and_reloads() {
        let mut state = test_state();
        state.diff.selected_path = Some("src/main.rs".to_string());
        state.diff.source = crate::diff::DiffSource::Unstaged;

        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::DiffToggleSource));

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

        let effects = run_reduce(
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

        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::DiffToggleSource));

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

        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::DiffNextFile));

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

        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::DiffPrevFile));

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

        run_reduce(&mut state, AppEvent::Command(AppCommand::DiffNextFile));
        run_reduce(&mut state, AppEvent::Command(AppCommand::DiffPrevFile));

        assert_eq!(state.diff.selected_path.as_deref(), Some("a.rs"));
        assert_eq!(state.diff.scroll_offset, 42);
        assert_eq!(state.diff.horizontal_scroll_offset, 7);
    }

    #[test]
    fn navigating_to_issues_triggers_github_auth_check() {
        let mut state = test_state();

        let effects = run_reduce(
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

        run_reduce(
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

        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::GitHubRefresh));

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

        let effects = run_reduce(
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

        let effects = run_reduce(
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

        run_reduce(
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

        let effects = super::github_pr_list_access_effects(
            &mut ReduceView::from_app_state(&mut state),
            false,
        );

        assert!(effects.is_empty());
        assert!(!state.github.prs_loading);
    }

    #[test]
    fn github_issues_loaded_populates_list_and_selection() {
        use crate::github::{Issue, IssueListLoadResult, IssueState};

        let mut state = test_state();
        state.github.issues_loading = true;

        run_reduce(
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

        let effects = super::github_issue_list_access_effects(
            &mut ReduceView::from_app_state(&mut state),
            false,
        );

        assert!(effects.is_empty());
        assert!(!state.github.issues_loading);
    }

    #[test]
    fn github_select_left_pane_switches_main_tab() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;

        run_reduce(
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

        run_reduce(
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

        let effects = run_reduce(
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

        run_reduce(
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

        run_reduce(
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

        let effects = run_reduce(
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

        run_reduce(
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

        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubPrDetailScroll(5)),
        );

        assert_eq!(state.github.pr_detail_scroll_offset, 5);
    }

    #[test]
    fn github_context_menu_open_selects_issue_and_stores_menu() {
        use crate::github::{GhContextTarget, Issue, IssueState};

        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.issues = vec![Issue {
            number: 42,
            title: "Context menu".to_string(),
            state: IssueState::Open,
            labels: Vec::new(),
            assignees: Vec::new(),
        }];

        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubContextMenuOpen {
                anchor_x: 12,
                anchor_y: 8,
                target: GhContextTarget::Issue { list_index: 0 },
            }),
        );

        assert_eq!(state.github.selected_issue, Some(42));
        let menu = state.github.context_menu.as_ref().expect("menu open");
        assert_eq!(menu.anchor_x, 12);
        assert_eq!(menu.items.len(), 6);
    }

    #[test]
    fn github_context_menu_open_includes_merge_for_mergeable_pr() {
        use crate::github::{GhContextMenuAction, GhContextTarget, PrState, PullRequest};

        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.prs = vec![PullRequest {
            number: 17,
            title: "Ready".to_string(),
            state: PrState::Open,
            author: "dev".to_string(),
            is_draft: false,
        }];

        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubContextMenuOpen {
                anchor_x: 4,
                anchor_y: 6,
                target: GhContextTarget::PullRequest { list_index: 0 },
            }),
        );

        assert_eq!(state.github.selected_pr, Some(17));
        let menu = state.github.context_menu.as_ref().expect("menu open");
        assert!(menu.items.contains(&GhContextMenuAction::Merge));
    }

    #[test]
    fn github_context_menu_merge_spawns_merge_side_effect() {
        use crate::github::{
            GhContextMenuAction, GhContextMenuState, GhContextTarget, PrState, PullRequest,
        };

        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.prs = vec![PullRequest {
            number: 17,
            title: "Ready".to_string(),
            state: PrState::Open,
            author: "dev".to_string(),
            is_draft: false,
        }];
        state.github.selected_pr = Some(17);
        state.github.context_menu = Some(GhContextMenuState {
            target: GhContextTarget::PullRequest { list_index: 0 },
            anchor_x: 1,
            anchor_y: 2,
            items: vec![
                GhContextMenuAction::View,
                GhContextMenuAction::Merge,
                GhContextMenuAction::OpenInBrowser,
                GhContextMenuAction::SendToAgent,
            ],
            cursor: 1,
        });

        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubContextMenuExecute),
        );

        assert!(state.github.context_menu.is_none());
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, SideEffect::SpawnGitHubPrMerge { number: 17 })));
    }

    #[test]
    fn github_context_menu_send_to_agent_writes_prompt() {
        use crate::github::{
            GhContextMenuAction, GhContextMenuState, GhContextTarget, Issue, IssueState,
        };

        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.issues = vec![Issue {
            number: 42,
            title: "Context menu".to_string(),
            state: IssueState::Open,
            labels: Vec::new(),
            assignees: Vec::new(),
        }];
        state.github.selected_issue = Some(42);
        state.github.context_menu = Some(GhContextMenuState {
            target: GhContextTarget::Issue { list_index: 0 },
            anchor_x: 1,
            anchor_y: 2,
            items: vec![
                GhContextMenuAction::View,
                GhContextMenuAction::CreateBranch,
                GhContextMenuAction::Comment,
                GhContextMenuAction::AddLabels,
                GhContextMenuAction::OpenInBrowser,
                GhContextMenuAction::SendToAgent,
            ],
            cursor: 5,
        });

        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubContextMenuExecute),
        );

        assert!(state.github.context_menu.is_none());
        assert_eq!(state.navigation.main_tab, MainTab::Agent);
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::WriteAgent(bytes) if std::str::from_utf8(bytes).is_ok_and(|text| text.contains("#42"))
        )));
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

        let effects = run_reduce(
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

        let effects = run_reduce(
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

        let effects = run_reduce(
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
                    crate::github::RepoLabel {
                        name: "bug".to_string(),
                        description: String::new(),
                    },
                    crate::github::RepoLabel {
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

        let effects = run_reduce(
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

        let effects = run_reduce(
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

        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(state.palette.open);
        assert!(effects.is_empty());

        state.palette.input = "Fixes #42".to_string();
        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(state.palette.open);
        assert!(effects.is_empty());

        state.palette.input = "main".to_string();
        let effects = run_reduce(
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
        run_reduce(
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
    fn github_pr_merge_completed_refreshes_pr_list() {
        use crate::github::IssueActionResult;

        let mut state = test_state();
        state.github.auth_ok = true;
        state.navigation.main_tab = MainTab::Prs;
        state.github.selected_pr = Some(17);
        let effects = run_reduce(
            &mut state,
            AppEvent::GitHubPrMergeCompleted {
                number: 17,
                result: IssueActionResult {
                    success: true,
                    error: None,
                    detail: Some("Merged pull request #17".to_string()),
                },
            },
        );

        assert_eq!(
            state.github.issue_action_message.as_deref(),
            Some("Merged pull request #17")
        );
        assert!(effects.contains(&SideEffect::SpawnGitHubPrList));
    }

    #[test]
    fn github_open_browser_completed_sets_status_message() {
        use crate::github::{GitHubBrowserTarget, IssueActionResult};

        let mut state = test_state();
        run_reduce(
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
    fn quit_command_emits_save_and_quit_when_persistence_enabled() {
        let mut state = test_state();
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::Quit));
        assert!(effects.contains(&SideEffect::SaveWorkspace));
        assert!(effects.contains(&SideEffect::Quit));
    }

    #[test]
    fn quit_command_skips_save_when_persistence_disabled() {
        let mut state = test_state();
        state.config.workspace.persist = false;
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::Quit));
        assert!(!effects.contains(&SideEffect::SaveWorkspace));
        assert!(effects.contains(&SideEffect::Quit));
    }

    #[test]
    fn agent_spawn_not_requested_off_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let effects = agent_spawn_effects_if_needed(&mut ReduceView::from_app_state(&mut state));
        assert!(effects.is_empty());
    }

    #[test]
    fn agent_new_creates_session_and_spawn_effect() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::AgentNew));
        assert_eq!(state.agent_manager.session_count(), 2);
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], SideEffect::SpawnAgent(_)));
    }

    #[test]
    fn agent_output_routes_to_matching_session() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        run_reduce(&mut state, AppEvent::Command(AppCommand::AgentNew));
        let second = state.agent_manager.active_id();
        state
            .agent_manager
            .set_active(AgentId::FIRST)
            .expect("switch");

        run_reduce(
            &mut state,
            AppEvent::AgentOutput {
                agent_id: second,
                data: b"background agent\n".to_vec(),
            },
        );

        assert_eq!(
            state
                .agent_manager
                .pty(second)
                .expect("second")
                .scrollback
                .line_count(),
            1
        );
        assert_eq!(state.active_agent().scrollback.line_count(), 0);
    }

    #[test]
    fn agent_cycle_switches_active_session() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        run_reduce(&mut state, AppEvent::Command(AppCommand::AgentNew));
        state
            .agent_manager
            .pty_mut(AgentId::FIRST)
            .expect("first")
            .spawned = true;
        let second = state.agent_manager.active_id();
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::AgentCycleNext));
        assert_eq!(state.agent_manager.active_id(), AgentId::FIRST);
        assert!(effects.is_empty());

        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::AgentCyclePrev));
        assert_eq!(state.agent_manager.active_id(), second);
        assert!(effects.contains(&SideEffect::SpawnAgent(second)));
    }

    #[test]
    fn agent_set_active_switches_session_and_spawns_if_needed() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        run_reduce(&mut state, AppEvent::Command(AppCommand::AgentNew));
        let second = state.agent_manager.active_id();
        state
            .agent_manager
            .set_active(AgentId::FIRST)
            .expect("switch");

        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::AgentSetActive(second)),
        );
        assert_eq!(state.agent_manager.active_id(), second);
        assert!(effects.contains(&SideEffect::SpawnAgent(second)));
    }

    #[test]
    fn agent_spawn_requested_on_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        state.active_agent_mut().spawned = false;
        let effects = agent_spawn_effects_if_needed(&mut ReduceView::from_app_state(&mut state));
        assert!(effects.contains(&SideEffect::SpawnAgent(AgentId::FIRST)));
    }

    #[test]
    fn agent_spawn_requested_only_once() {
        let mut state = test_state();
        state.active_agent_mut().spawned = true;
        let effects = agent_spawn_effects_if_needed(&mut ReduceView::from_app_state(&mut state));
        assert!(effects.is_empty());
    }

    #[test]
    fn selecting_agent_tab_emits_spawn_side_effect() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::Navigation(NavCommand::SelectMainTab(
                MainTab::Agent,
            ))),
        );
        assert!(effects.contains(&SideEffect::SpawnAgent(AgentId::FIRST)));
    }

    #[test]
    fn shell_output_appends_to_scrollback_and_sets_dirty() {
        let mut state = test_state();
        run_reduce(&mut state, AppEvent::ShellOutput(b"hello\nworld".to_vec()));
        assert_eq!(state.shell.scrollback.line_count(), 2);
        assert!(state.dirty);
    }

    #[test]
    fn agent_output_appends_to_scrollback_and_sets_dirty() {
        let mut state = test_state();
        run_reduce(
            &mut state,
            AppEvent::AgentOutput {
                agent_id: AgentId::FIRST,
                data: b"agent line\n".to_vec(),
            },
        );
        assert_eq!(state.active_agent().scrollback.line_count(), 1);
        assert!(state.dirty);
    }

    #[test]
    fn agent_exited_clears_running_and_sets_dirty() {
        let mut state = test_state();
        state.active_agent_mut().running = true;
        state.active_agent_mut().status = AgentStatus::Executing;
        run_reduce(
            &mut state,
            AppEvent::AgentExited {
                agent_id: AgentId::FIRST,
                code: 1,
            },
        );
        assert!(!state.active_agent().running);
        assert_eq!(state.active_agent().status, AgentStatus::Error);
        assert!(state.dirty);
    }

    #[test]
    fn agent_output_updates_status_from_heuristics() {
        let mut state = test_state();
        state.active_agent_mut().running = true;
        run_reduce(
            &mut state,
            AppEvent::AgentOutput {
                agent_id: AgentId::FIRST,
                data: b"Thinking about the next step\n".to_vec(),
            },
        );
        assert_eq!(state.active_agent().status, AgentStatus::Thinking);
    }

    #[test]
    fn agent_exited_zero_sets_success_status() {
        let mut state = test_state();
        state.active_agent_mut().running = true;
        run_reduce(
            &mut state,
            AppEvent::AgentExited {
                agent_id: AgentId::FIRST,
                code: 0,
            },
        );
        assert_eq!(state.active_agent().status, AgentStatus::Success);
        assert_eq!(state.active_agent().exit_code, Some(0));
        assert!(state.active_agent().restart_hint.is_some());
    }

    #[test]
    fn agent_restart_emits_side_effect_on_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::AgentRestart));
        assert!(effects.contains(&SideEffect::RestartAgent(AgentId::FIRST)));
        assert!(state.dirty);
    }

    #[test]
    fn agent_restart_ignored_off_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::AgentRestart));
        assert!(effects.is_empty());
    }

    #[test]
    fn agent_exited_sets_restart_hint_with_code() {
        let mut state = test_state();
        run_reduce(
            &mut state,
            AppEvent::AgentExited {
                agent_id: AgentId::FIRST,
                code: 2,
            },
        );
        assert_eq!(state.active_agent().exit_code, Some(2));
        assert!(state
            .active_agent()
            .restart_hint
            .as_deref()
            .is_some_and(|hint| hint.contains("code 2")));
    }

    #[test]
    fn shell_write_emits_side_effect() {
        let mut state = test_state();
        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::ShellWrite(b"ls\n".to_vec())),
        );
        assert!(effects.contains(&SideEffect::WriteShell(b"ls\n".to_vec())));
    }

    #[test]
    fn agent_write_emits_side_effect() {
        let mut state = test_state();
        let effects = run_reduce(
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
                .active_agent_mut()
                .scrollback
                .append_bytes(format!("line {index}\n").as_bytes());
        }

        run_reduce(&mut state, AppEvent::Command(AppCommand::AgentScroll(-1)));

        assert!(!state.active_agent().follow_tail);
        assert!(state.dirty);
    }

    #[test]
    fn palette_open_sets_state_and_focus() {
        let mut state = test_state();
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        assert!(state.palette.open);
        assert_eq!(state.navigation.focus, FocusTarget::CommandPalette);
        assert!(!state.palette.matches.is_empty());
    }

    #[test]
    fn palette_close_restores_previous_focus() {
        let mut state = test_state();
        state.navigation.focus = FocusTarget::Shell;
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteClose));
        assert!(!state.palette.open);
        assert_eq!(state.navigation.focus, FocusTarget::Shell);
    }

    #[test]
    fn palette_fuzzy_query_matches_git_refresh() {
        let mut state = test_state();
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('g')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('i')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('t')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar(' ')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('r')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('e')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('f')),
        );
        let first = state.palette.matches.first().copied().expect("match");
        assert_eq!(crate::commands::COMMANDS[first].id, "git.refresh");
    }

    #[test]
    fn palette_execute_selected_closes_palette() {
        let mut state = test_state();
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(!state.palette.open);
    }

    #[test]
    fn palette_execute_git_refresh_sets_loading() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('g')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('i')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('t')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar(' ')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('r')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('e')),
        );
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteAppendChar('f')),
        );
        let effects = run_reduce(
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
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(effects.contains(&SideEffect::SaveWorkspace));
        assert!(state.palette.history.last().is_some());
    }

    #[test]
    fn palette_execute_skips_save_when_persistence_disabled() {
        let mut state = test_state();
        state.config.workspace.persist = false;
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PaletteExecuteSelected),
        );
        assert!(!effects.contains(&SideEffect::SaveWorkspace));
        assert!(state.palette.history.last().is_some());
    }

    #[test]
    fn palette_history_up_uses_command_title() {
        let mut state = test_state();
        state.palette.history = vec!["git.refresh".to_string()];
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteOpen));
        run_reduce(&mut state, AppEvent::Command(AppCommand::PaletteHistoryUp));
        assert_eq!(state.palette.input, "Git: Refresh Status");
    }

    #[test]
    fn startup_file_tree_contains_root_only() {
        let state = test_state();
        assert_eq!(state.file_tree.nodes.len(), 1);
        assert!(state.file_tree.children.is_empty());
    }

    #[test]
    fn file_tree_startup_effects_expand_root() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        assert!(!state.file_tree.nodes[&root].expanded);

        let effects = file_tree_startup_effects(&mut ReduceView::from_app_state(&mut state));

        assert!(state.file_tree.nodes[&root].expanded);
        assert!(state.file_tree.loading.contains(&root));
        assert!(effects.contains(&SideEffect::LoadDirectoryChildren(root)));
    }

    #[test]
    fn workspace_expand_pending_emits_load_for_known_directory() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        let src = root.join("src");
        state.file_tree.nodes.insert(
            src.clone(),
            crate::file_tree::FileNode {
                path: src.clone(),
                name: "src".to_string(),
                is_dir: true,
                expanded: false,
                children_loaded: false,
                load_error: None,
                git_status: None,
            },
        );
        state.workspace_meta.pending_expanded_paths = vec![src.clone()];

        let effects = workspace_expand_pending_effects(&mut ReduceView::from_app_state(&mut state));

        assert!(state.file_tree.nodes[&src].expanded);
        assert!(effects.contains(&SideEffect::LoadDirectoryChildren(src)));
        assert!(state.workspace_meta.pending_expanded_paths.is_empty());
    }

    #[test]
    fn workspace_expand_pending_keeps_missing_paths() {
        let mut state = test_state();
        let missing = state.file_tree.root.join("missing");
        state.workspace_meta.pending_expanded_paths = vec![missing.clone()];

        let effects = workspace_expand_pending_effects(&mut ReduceView::from_app_state(&mut state));

        assert!(effects.is_empty());
        assert_eq!(state.workspace_meta.pending_expanded_paths, vec![missing]);
    }

    #[test]
    fn file_tree_children_loaded_retries_pending_expansion_and_selection() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        let src = root.join("src");
        state.workspace_meta.pending_expanded_paths = vec![src.clone()];
        state.workspace_meta.pending_selected_path = Some(src.join("main.rs"));

        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeExpand(root.clone())),
        );
        run_reduce(
            &mut state,
            AppEvent::FileTreeChildrenLoaded {
                parent: root.clone(),
                children: vec![
                    DirectoryEntry {
                        path: src.clone(),
                        name: "src".to_string(),
                        is_dir: true,
                    },
                    DirectoryEntry {
                        path: src.join("main.rs"),
                        name: "main.rs".to_string(),
                        is_dir: false,
                    },
                ],
                error: None,
            },
        );

        assert!(state.file_tree.nodes[&src].expanded);
        assert_eq!(
            state.file_tree.selected.as_ref(),
            Some(&src.join("main.rs"))
        );
        assert!(state.workspace_meta.pending_selected_path.is_none());
    }

    #[test]
    fn file_tree_expand_emits_load_side_effect() {
        let mut state = test_state();
        let root = state.file_tree.root.clone();
        let effects = run_reduce(
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
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeExpand(root.clone())),
        );
        run_reduce(
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
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeExpand(root.clone())),
        );
        run_reduce(
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
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::FileTreeMoveSelection(1)),
        );
        assert_eq!(state.file_tree.selected, Some(root.join("src")));
    }

    #[test]
    fn preview_file_emits_load_side_effect_and_switches_tab() {
        let mut state = test_state();
        let path = PathBuf::from("src/main.rs");
        let effects = run_reduce(
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
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: None,
            }),
        );
        run_reduce(
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
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewFile {
                path: path.clone(),
                line: Some(25),
            }),
        );
        run_reduce(
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

        let effects = run_reduce(
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
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewScroll(200)),
        );
        assert!(state.preview.scroll_offset > 0);
        run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::PreviewScroll(-500)),
        );
        assert_eq!(state.preview.scroll_offset, 0);
    }

    #[test]
    fn search_set_query_schedules_debounce_and_cancels() {
        let mut state = test_state();
        let effects = run_reduce(
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
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::SearchExecute));
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
        run_reduce(
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
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::SearchClear));
        assert!(effects.contains(&SideEffect::CancelSearch));
        assert!(state.search.query.is_empty());
        assert!(!state.search.running);
    }

    #[test]
    fn open_editor_emits_launch_side_effect() {
        let mut state = test_state();
        let path = PathBuf::from("src/main.rs");
        let effects = run_reduce(
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
        run_reduce(
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
        run_reduce(
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
        run_reduce(&mut state, AppEvent::Command(AppCommand::ModalDismiss));
        assert!(state.notifications.modal.is_none());
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

        let effects = run_reduce(
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
    fn fs_changed_invalidates_on_file_delete() {
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

        let effects = run_reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![root.join("src/main.rs")],
            },
        );

        assert!(effects.contains(&SideEffect::LoadDirectoryChildren(root.join("src"))));
        assert!(!state
            .file_tree
            .nodes
            .contains_key(&root.join("src/main.rs")));
    }

    #[test]
    fn fs_changed_reloads_matching_preview_file() {
        let mut state = test_state();
        let path = PathBuf::from("/tmp/repo/src/main.rs");
        state.preview.path = Some(path.clone());
        state.preview.lines = vec!["old".to_string()];
        state.preview.scroll_offset = 5;

        let effects = run_reduce(
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
        state.config.git.watch = false;
        state.preview.path = Some(file.clone());
        state.preview.lines = vec!["unchanged".to_string()];
        state.preview.loaded_mtime = mtime;

        let effects = run_reduce(
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

        let effects = run_reduce(
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
        state.config.git.watch = false;
        let path = PathBuf::from("/tmp/repo/src/main.rs");
        state.preview.path = Some(path.clone());
        state.preview.loading = true;

        let effects = run_reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/other.rs")],
            },
        );
        assert!(effects.is_empty());

        state.preview.loading = false;
        let effects = run_reduce(
            &mut state,
            AppEvent::FsChanged {
                paths: vec![PathBuf::from("/tmp/repo/src/other.rs")],
            },
        );
        assert!(effects.is_empty());
    }
}
