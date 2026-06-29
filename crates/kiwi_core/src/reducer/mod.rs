
use crate::state::ReduceView;

use crate::events::{AgentEffect, AppCommand, AppEvent, ShellEffect, SideEffect};


mod github;
mod git;
mod shell;
mod agent;
mod search;
mod diff;
mod navigation;
mod plugins;
mod workspace;
mod settings;

pub use self::navigation::apply_navigation;
pub use self::agent::agent_spawn_effects_if_needed;
pub use self::agent::agent_new_effects;
pub use self::agent::agent_cycle_effects;
pub use self::workspace::file_tree_startup_effects;
pub use self::workspace::workspace_expand_pending_effects;
pub use self::workspace::workspace_restore_effects;
pub use self::git::git_refresh_effects;
pub use self::git::{branch_detail_access_effects, branch_list_access_effects};
pub use self::git::branch_checkout_effects;
pub use self::github::github_refresh_effects;
pub use self::github::github_first_access_effects;
pub use self::github::github_issue_list_effects;
pub use self::github::github_pr_list_effects;
pub use self::github::github_pr_list_access_effects;
pub use self::github::github_issue_detail_access_effects;
pub use self::github::github_pr_detail_access_effects;
pub use self::diff::diff_select_file_effects;
pub use self::diff::diff_move_file_effects;
pub use self::diff::diff_set_source_effects;

use self::plugins::{
    reduce_plugin_install, reduce_plugin_install_finished, reduce_plugin_install_progress,
    reduce_plugin_reinstall, reduce_plugin_remove, reduce_plugin_set_enabled,
    reduce_set_agent,
};
use self::github::reduce_github_refresh_requested;
use self::github::reduce_github_auth_checked;
use self::github::reduce_github_issues_loaded;
use self::github::reduce_github_prs_loaded;
use self::github::reduce_github_issue_detail_loaded;
use self::github::reduce_github_pr_detail_loaded;
use self::github::reduce_github_move_issue_selection;
use self::github::reduce_github_select_issue;
use self::github::reduce_github_move_pr_selection;
use self::github::reduce_github_select_pr;
use self::github::reduce_github_open_selected;
use self::github::reduce_github_select_left_pane;
use self::github::reduce_github_issue_detail_scroll;
use self::github::reduce_github_issue_detail_page_scroll;
use self::github::reduce_github_pr_detail_scroll;
use self::github::reduce_github_pr_detail_page_scroll;
use self::github::reduce_github_issue_comment_completed;
use self::github::reduce_github_issue_create_branch_completed;
use self::github::reduce_github_repo_labels_loaded;
use self::github::reduce_github_issue_labels_applied;
use self::github::reduce_github_label_picker_move;
use self::github::reduce_github_label_picker_toggle;
use self::github::reduce_github_label_picker_apply;
use self::github::reduce_github_label_picker_cancel;
use self::github::reduce_github_repo_milestones_loaded;
use self::github::reduce_github_issue_milestone_assigned;
use self::github::reduce_github_milestone_picker_move;
use self::github::reduce_github_milestone_picker_apply;
use self::github::reduce_github_milestone_picker_cancel;
use self::github::reduce_github_context_menu_open;
use self::github::reduce_github_context_menu_move;
use self::github::reduce_github_context_menu_select;
use self::github::reduce_github_context_menu_execute;
use self::github::reduce_github_list_action;
use self::github::reduce_github_context_menu_cancel;
use self::github::reduce_github_open_in_browser;
use self::github::reduce_github_open_browser_completed;
use self::github::reduce_github_issue_create_completed;
use self::github::reduce_github_issue_create_cancel;
use self::github::reduce_github_issue_create_open;
use self::github::reduce_github_issue_create_submit;
use self::github::reduce_github_pr_create_completed;
use self::github::reduce_github_pr_merge_completed;
use self::git::reduce_git_refresh_requested;
use self::git::reduce_git_status_updated;
use self::git::reduce_branch_refresh;
use self::git::reduce_branch_move_selection;
use self::git::reduce_branch_select;
use self::git::reduce_branch_checkout_selected;
use self::git::{
    reduce_branch_checkout_completed, reduce_branch_detail_loaded, reduce_branch_detail_scroll,
    reduce_branch_list_loaded,
};
use self::shell::reduce_shell_output;
use self::shell::reduce_shell_exited;
use self::agent::reduce_agent_output;
use self::agent::reduce_agent_exited;
use self::agent::reduce_agent_new;
use self::agent::reduce_agent_set_active;
use self::agent::reduce_agent_cycle;
use self::agent::reduce_agent_restart;
use self::shell::reduce_shell_scroll;
use self::shell::reduce_shell_scroll_lines;
use self::agent::reduce_agent_scroll;
use self::agent::reduce_agent_scroll_lines;
use self::agent::{
    reduce_agent_user_send, reduce_agent_token_chunk, reduce_agent_tool_call_start,
    reduce_agent_tool_result, reduce_agent_turn_complete, reduce_agent_api_error,
    reduce_agent_toggle_tool_expand, reduce_agent_clear_history,
};
use self::navigation::reduce_palette_open;
use self::navigation::reduce_palette_close;
use self::navigation::reduce_palette_append_char;
use self::navigation::reduce_palette_backspace;
use self::navigation::reduce_palette_move_selection;
use self::navigation::reduce_palette_history_up;
use self::navigation::reduce_palette_history_down;
use self::navigation::reduce_palette_execute_selected;
use self::navigation::reduce_palette_execute_match;
use self::workspace::reduce_file_tree_expand;
use self::workspace::reduce_file_tree_collapse;
use self::workspace::reduce_file_tree_select;
use self::workspace::reduce_file_tree_move_selection;
use self::git::reduce_git_move_selection;
use self::git::reduce_git_select;
use self::git::reduce_git_open_selected;
use self::diff::reduce_diff_select_file;
use self::diff::reduce_diff_move_file;
use self::diff::reduce_diff_loaded;
use self::diff::reduce_diff_toggle_source;
use self::diff::reduce_diff_set_source;
use self::workspace::reduce_file_tree_refresh;
use self::workspace::reduce_file_tree_children_loaded;
use self::workspace::reduce_preview_file;
use self::workspace::reduce_preview_loaded;
use self::workspace::reduce_preview_scroll;
use self::workspace::reduce_preview_page_scroll;
use self::diff::reduce_diff_scroll;
use self::diff::reduce_diff_page_scroll;
use self::diff::reduce_diff_horizontal_scroll;
use self::search::reduce_search_set_query;
use self::search::reduce_search_append_char;
use self::search::reduce_search_backspace;
use self::search::reduce_search_clear;
use self::search::reduce_search_set_mode;
use self::search::reduce_search_execute;
use self::search::reduce_search_cancel;
use self::search::reduce_search_move_selection;
use self::search::reduce_search_select;
use self::search::reduce_search_completed;
use self::settings::reduce_open_editor;
use self::navigation::reduce_modal_dismiss;
use self::settings::reduce_editor_launched;
use self::settings::reduce_editor_launch_failed;
use self::settings::reduce_settings_move_selection;
use self::settings::reduce_settings_select;
use self::settings::reduce_settings_apply_theme;
use self::settings::reduce_set_theme;
use self::workspace::reduce_fs_changed;

pub fn reduce(state: &mut ReduceView<'_>, event: AppEvent) -> Vec<SideEffect> {
    match event {
        AppEvent::Command(command) => reduce_command(state, command),
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
        AppEvent::AgentTokenChunk { agent_id, text } => {
            reduce_agent_token_chunk(state, agent_id, text)
        }
        AppEvent::AgentToolCallStart {
            agent_id,
            tool_use_id,
            tool_name,
            input_json,
        } => reduce_agent_tool_call_start(state, agent_id, tool_use_id, tool_name, input_json),
        AppEvent::AgentToolResult {
            agent_id,
            tool_use_id,
            content,
            is_error,
        } => reduce_agent_tool_result(state, agent_id, tool_use_id, content, is_error),
        AppEvent::AgentTurnComplete { agent_id } => reduce_agent_turn_complete(state, agent_id),
        AppEvent::AgentApiError { agent_id, message } => {
            reduce_agent_api_error(state, agent_id, message)
        }
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
        AppEvent::GitHubRepoMilestonesLoaded { result } => {
            reduce_github_repo_milestones_loaded(state, result)
        }
        AppEvent::GitHubIssueMilestoneAssigned { number, result } => {
            reduce_github_issue_milestone_assigned(state, number, result)
        }
        AppEvent::GitHubIssueCreateCompleted { outcome } => {
            reduce_github_issue_create_completed(state, outcome)
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
        AppEvent::BranchDetailLoaded { name, detail, error } => {
            reduce_branch_detail_loaded(state, name, detail, error)
        }
        AppEvent::BranchCheckoutCompleted { branch_name, error } => {
            reduce_branch_checkout_completed(state, branch_name, error)
        }
        AppEvent::PluginInstallProgress {
            message,
            step,
            total,
        } => reduce_plugin_install_progress(state, message, step, total),
        AppEvent::PluginInstallFinished { result, error } => {
            reduce_plugin_install_finished(state, result, error)
        }
    }
}

pub fn reduce_command(state: &mut ReduceView<'_>, command: AppCommand) -> Vec<SideEffect> {
    match command {
        AppCommand::Navigation(nav) => {
            apply_navigation(state, nav);
            let mut effects = agent_spawn_effects_if_needed(state);
            effects.extend(github_first_access_effects(state));
            effects.extend(github_issue_list_effects(state, false));
            effects.extend(github_pr_list_access_effects(state, false));
            effects.extend(github_issue_detail_access_effects(state, false));
            effects.extend(github_pr_detail_access_effects(state, false));
            effects.extend(branch_list_access_effects(state, false));
            effects.extend(branch_detail_access_effects(state, false));
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
        AppCommand::GitHubMilestonePickerMove(delta) => {
            reduce_github_milestone_picker_move(state, delta)
        }
        AppCommand::GitHubMilestonePickerApply => reduce_github_milestone_picker_apply(state),
        AppCommand::GitHubMilestonePickerCancel => reduce_github_milestone_picker_cancel(state),
        AppCommand::GitHubIssueCreateOpen => reduce_github_issue_create_open(state),
        AppCommand::GitHubIssueCreateCancel => reduce_github_issue_create_cancel(state),
        AppCommand::GitHubIssueCreateSubmit => reduce_github_issue_create_submit(state),
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
        AppCommand::GitHubListAction { target, action } => {
            reduce_github_list_action(state, target, action)
        }
        AppCommand::GitHubOpenInBrowser => reduce_github_open_in_browser(state),
        AppCommand::ShellWrite(data) => vec![SideEffect::Shell(ShellEffect::Write(data))],
        AppCommand::ShellScroll(delta) => reduce_shell_scroll(state, delta),
        AppCommand::ShellScrollLines(lines) => reduce_shell_scroll_lines(state, lines),
        AppCommand::AgentWrite(data) => vec![SideEffect::Agent(AgentEffect::Write(data))],
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
        AppCommand::BranchDetailScroll(delta) => reduce_branch_detail_scroll(state, delta),
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
        AppCommand::PluginSetEnabled { name, enabled } => {
            reduce_plugin_set_enabled(state, name, enabled)
        }
        AppCommand::PluginInstall { src_path } => reduce_plugin_install(state, src_path),
        AppCommand::PluginRemove { name } => reduce_plugin_remove(state, name),
        AppCommand::PluginReinstall { src_path } => reduce_plugin_reinstall(state, src_path),
        AppCommand::SetAgent { command, args, mode, provider, model, api_key_env, api_url, api_key } => {
            reduce_set_agent(state, command, args, mode, provider, model, api_key_env, api_url, api_key)
        }
        AppCommand::AgentUserSend { agent_id, text } => {
            reduce_agent_user_send(state, agent_id, text)
        }
        AppCommand::AgentToggleToolExpand { agent_id, tool_use_id } => {
            reduce_agent_toggle_tool_expand(state, agent_id, tool_use_id)
        }
        AppCommand::AgentClearHistory { agent_id } => reduce_agent_clear_history(state, agent_id),
    }
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
    use std::time::SystemTime;
    use crate::agent::{AgentId, AgentManager};
    use crate::events::{FsEffect, GitEffect, GitHubEffect, SearchEffect};
    use crate::file_tree::{DirectoryEntry, FileTreeState};
    use crate::github::GitHubLeftPane;
    use crate::preview::{PreviewLoadResult, PreviewState};
    use crate::search::{SearchMode, SearchResult, SearchState};
    use crate::state::{AppState, ViewportMetrics};
    use super::diff::{diff_gutter_width, max_lineno_width};

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
        assert_eq!(state.github.left_pane, GitHubLeftPane::Branches);
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
        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
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

        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnBranchList)));
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
            |effect| matches!(effect, SideEffect::Git(GitEffect::SpawnBranchCheckout { name }) if name == "dev")
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

        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnBranchList)));
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
                .filter(|effect| **effect == SideEffect::Git(GitEffect::SpawnRefresh))
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

        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
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

        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
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

        assert!(!effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
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

        assert!(!effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
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

        assert!(first.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
        assert!(!second.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
        assert!(state.git.loading);
    }

    #[test]
    fn request_git_refresh_sets_loading() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::RequestGitRefresh));

        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
        assert!(state.git.loading);
    }

    #[test]
    fn git_refresh_command_emits_side_effect() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        let effects = run_reduce(&mut state, AppEvent::Command(AppCommand::GitRefresh));
        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
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
                .any(|effect| matches!(effect, SideEffect::Fs(FsEffect::LoadFileDiff { path, .. }) if path == "src/main.rs"))
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
            SideEffect::Fs(FsEffect::LoadFileDiff { path, source })
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
            |effect| matches!(effect, SideEffect::Fs(FsEffect::LoadFileDiff { path, .. }) if path == "b.rs")
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
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnAuthCheck)));
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
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnAuthCheck)));
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
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnIssueList)));
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
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnPrList)));
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
    fn github_issues_loaded_does_not_trigger_pr_detail_on_prs_tab() {
        // Regression: reduce_github_issues_loaded previously branched on
        // MainTab::Prs and could emit SpawnGitHubPrDetail when issues finished
        // loading while the user was viewing PRs.
        use crate::github::{Issue, IssueListLoadResult, IssueState, PrState, PullRequest};

        let mut state = test_state();
        state.navigation.main_tab = MainTab::Prs;
        state.github.selected_pr = Some(42);
        state.github.prs = vec![PullRequest {
            number: 42,
            title: "Some PR".to_string(),
            state: PrState::Open,
            author: "octocat".to_string(),
            is_draft: false,
        }];

        let effects = run_reduce(
            &mut state,
            AppEvent::GitHubIssuesLoaded {
                result: IssueListLoadResult {
                    issues: vec![Issue {
                        number: 1,
                        title: "Some issue".to_string(),
                        state: IssueState::Open,
                        labels: Vec::new(),
                        assignees: Vec::new(),
                    }],
                    error: None,
                },
            },
        );

        let has_pr_detail = effects.iter().any(|e| {
            matches!(e, SideEffect::GitHub(GitHubEffect::SpawnPrDetail { .. }))
        });
        assert!(!has_pr_detail, "issues-loaded must not spawn PR detail effects");
    }

    #[test]
    fn github_open_selected_does_not_mutate_left_pane_on_issues() {
        // Regression: reduce_github_open_selected previously reassigned
        // left_pane = Issues inside the Issues arm — a no-op that implied
        // the arm handled other pane states. Verify left_pane is unchanged.
        use crate::github::{Issue, IssueState};

        let mut state = test_state();
        state.github.left_pane = GitHubLeftPane::Issues;
        state.github.selected_issue = Some(7);
        state.github.issues = vec![Issue {
            number: 7,
            title: "Test".to_string(),
            state: IssueState::Open,
            labels: Vec::new(),
            assignees: Vec::new(),
        }];
        state.github.auth_ok = true;

        run_reduce(&mut state, AppEvent::Command(AppCommand::GitHubOpenSelected));

        assert_eq!(state.github.left_pane, GitHubLeftPane::Issues);
        assert_eq!(state.navigation.main_tab, MainTab::Issues);
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

        let effects = super::github_issue_list_effects(
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
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnIssueDetail { number: 42 })));
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
                        body: None,
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
            body: None,
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
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnPrDetail { number: 60 })));
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
        assert_eq!(menu.items.len(), 7);
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
            .any(|effect| matches!(effect, SideEffect::GitHub(GitHubEffect::SpawnPrMerge { number: 17 }))));
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
            SideEffect::Agent(AgentEffect::Write(bytes)) if std::str::from_utf8(bytes).is_ok_and(|text| text.contains("#42"))
        )));
    }

    #[test]
    fn github_list_action_create_branch_spawns_side_effect() {
        use crate::github::{GhContextMenuAction, GhContextTarget, Issue, IssueState};

        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.issues = vec![Issue {
            number: 9,
            title: "Branch me".to_string(),
            state: IssueState::Open,
            labels: Vec::new(),
            assignees: Vec::new(),
        }];

        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubListAction {
                target: GhContextTarget::Issue { list_index: 0 },
                action: GhContextMenuAction::CreateBranch,
            }),
        );

        assert_eq!(state.github.selected_issue, Some(9));
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::GitHub(GitHubEffect::SpawnIssueCreateBranch { number: 9 })
        )));
    }

    #[test]
    fn github_list_action_view_uses_unpaired_main_tab() {
        use crate::github::{GhContextMenuAction, GhContextTarget, Issue, IssueState};
        use crate::navigation::LeftNavTab;

        let mut state = test_state();
        state.github.auth_ok = true;
        state.navigation.left_tab = LeftNavTab::Search;
        state.github.issues = vec![Issue {
            number: 3,
            title: "GUI view".to_string(),
            state: IssueState::Open,
            labels: Vec::new(),
            assignees: Vec::new(),
        }];

        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubListAction {
                target: GhContextTarget::Issue { list_index: 0 },
                action: GhContextMenuAction::View,
            }),
        );

        assert_eq!(state.navigation.main_tab, MainTab::Issues);
        assert_eq!(state.navigation.left_tab, LeftNavTab::Search);
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::GitHub(GitHubEffect::SpawnIssueDetail { number: 3 })
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
                SideEffect::GitHub(GitHubEffect::SpawnIssueComment { number: 42, body })
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
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnIssueList)));
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnIssueDetail { number: 42 })));
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
        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
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
                SideEffect::GitHub(GitHubEffect::SpawnIssueLabelApply { number: 9, labels })
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
                SideEffect::GitHub(GitHubEffect::SpawnOpenBrowser {
                    target: crate::github::GitHubBrowserTarget::Issue(42)
                })
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
                SideEffect::GitHub(GitHubEffect::SpawnPrCreate { request })
                if *request == PrCreateRequest {
                    title: "Fix login".to_string(),
                    body: "Fixes #42".to_string(),
                    base: Some("main".to_string()),
                }
            )
        }));
    }

    #[test]
    fn github_issue_create_submit_spawns_side_effect() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.issue_create_modal.open = true;
        state.github.issue_create_modal.title = "Bug report".to_string();
        state.github.issue_create_modal.body = "Details".to_string();

        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::GitHubIssueCreateSubmit),
        );

        assert!(state.github.issue_create_modal.submitting);
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::GitHub(GitHubEffect::SpawnIssueCreate { .. })
        )));
    }

    #[test]
    fn github_issue_create_completed_refreshes_issue_list() {
        use crate::github::{IssueActionResult, IssueCreateResult};

        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.issue_create_modal.open = true;
        state.github.issue_create_modal.submitting = true;
        state
            .navigation
            .apply(crate::navigation::NavCommand::SelectLeftTab(
                crate::navigation::LeftNavTab::Gh,
            ));

        let effects = run_reduce(
            &mut state,
            AppEvent::GitHubIssueCreateCompleted {
                outcome: IssueCreateResult {
                    result: IssueActionResult {
                        success: true,
                        error: None,
                        detail: Some("https://github.com/o/r/issues/7".to_string()),
                    },
                    number: Some(7),
                },
            },
        );

        assert!(!state.github.issue_create_modal.open);
        assert_eq!(state.github.selected_issue, Some(7));
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnIssueList)));
        assert!(effects.iter().any(|effect| matches!(
            effect,
            SideEffect::GitHub(GitHubEffect::SpawnIssueDetail { number: 7 })
        )));
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
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnPrList)));
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
        assert!(matches!(effects[0], SideEffect::Agent(AgentEffect::Spawn(_))));
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
        assert!(effects.contains(&SideEffect::Agent(AgentEffect::Spawn(second))));
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
        assert!(effects.contains(&SideEffect::Agent(AgentEffect::Spawn(second))));
    }

    #[test]
    fn agent_spawn_requested_on_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        state.active_agent_mut().spawned = false;
        let effects = agent_spawn_effects_if_needed(&mut ReduceView::from_app_state(&mut state));
        assert!(effects.contains(&SideEffect::Agent(AgentEffect::Spawn(AgentId::FIRST))));
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
        assert!(effects.contains(&SideEffect::Agent(AgentEffect::Spawn(AgentId::FIRST))));
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
    fn agent_output_does_not_update_status_via_scrollback_heuristic() {
        // PTY scrollback heuristics were removed in Phase 6 (#334); status stays Idle.
        let mut state = test_state();
        state.active_agent_mut().running = true;
        let payload = format!(
            "{}{}",
            "x".repeat(500),
            "Thinking about the next step\n"
        );
        run_reduce(
            &mut state,
            AppEvent::AgentOutput {
                agent_id: AgentId::FIRST,
                data: payload.into_bytes(),
            },
        );
        assert_eq!(state.active_agent().status, AgentStatus::Idle);
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
        assert!(effects.contains(&SideEffect::Agent(AgentEffect::Restart(AgentId::FIRST))));
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
        assert!(effects.contains(&SideEffect::Shell(ShellEffect::Write(b"ls\n".to_vec()))));
    }

    #[test]
    fn agent_write_emits_side_effect() {
        let mut state = test_state();
        let effects = run_reduce(
            &mut state,
            AppEvent::Command(AppCommand::AgentWrite(b"hello\n".to_vec())),
        );
        assert!(effects.contains(&SideEffect::Agent(AgentEffect::Write(b"hello\n".to_vec()))));
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

        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
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
        assert!(effects.contains(&SideEffect::Fs(FsEffect::LoadDirectoryChildren(root))));
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
        assert!(effects.contains(&SideEffect::Fs(FsEffect::LoadDirectoryChildren(src))));
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
        assert!(effects.contains(&SideEffect::Fs(FsEffect::LoadDirectoryChildren(root))));
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
        assert!(effects.contains(&SideEffect::Fs(FsEffect::LoadPreviewFile(path))));
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

        assert!(effects.contains(&SideEffect::Fs(FsEffect::LoadPreviewFile(path))));
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
        assert!(effects.contains(&SideEffect::Search(SearchEffect::Cancel)));
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
            SideEffect::Search(SearchEffect::Run {
                mode: SearchMode::Files,
                query,
                generation: 1,
            }) if query == "main"
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
        assert!(effects.contains(&SideEffect::Search(SearchEffect::Cancel)));
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
        assert!(effects.contains(&SideEffect::Fs(FsEffect::LaunchEditor { path, line: None })));
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

        assert!(effects.contains(&SideEffect::Fs(FsEffect::LoadDirectoryChildren(root.join("src")))));
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

        assert!(effects.contains(&SideEffect::Fs(FsEffect::LoadDirectoryChildren(root.join("src")))));
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

        assert!(effects.contains(&SideEffect::Fs(FsEffect::LoadPreviewFile(path))));
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

        assert!(effects.contains(&SideEffect::Fs(FsEffect::LoadPreviewFile(file))));
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

    #[test]
    fn max_lineno_width_zero_does_not_panic() {
        // git2 returns line number 0 for binary/synthetic hunks; must not panic
        assert_eq!(max_lineno_width([0u32].into_iter()), 1);
    }

    #[test]
    fn max_lineno_width_typical_values() {
        assert_eq!(max_lineno_width([1u32].into_iter()), 1);
        assert_eq!(max_lineno_width([9u32].into_iter()), 1);
        assert_eq!(max_lineno_width([10u32].into_iter()), 2);
        assert_eq!(max_lineno_width([99u32].into_iter()), 2);
        assert_eq!(max_lineno_width([100u32].into_iter()), 3);
        assert_eq!(max_lineno_width([1, 10, 100].into_iter()), 3);
    }

    #[test]
    fn max_lineno_width_empty_returns_one() {
        assert_eq!(max_lineno_width(std::iter::empty()), 1);
    }

    #[test]
    fn diff_gutter_width_zero_linenos_do_not_panic() {
        use crate::diff::{DiffLine, DiffLineKind};
        let lines = vec![DiffLine {
            kind: DiffLineKind::Context,
            content: String::new(),
            old_lineno: Some(0),
            new_lineno: Some(0),
        }];
        // Must not panic; gutter width should be > 0 since linenos are present
        let width = diff_gutter_width(&lines);
        assert!(width > 0);
    }
}
