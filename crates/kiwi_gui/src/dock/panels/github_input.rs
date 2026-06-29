//! GitHub navigation sync and keyboard input for dock tabs (#191).

use egui::{Context, Key};
use kiwi_core::events::AppCommand;
use kiwi_core::git::branch_selected_row_index;
use kiwi_core::github::{
    issue_selected_row_index, pr_selected_row_index, GitHubLeftPane,
};
use kiwi_core::navigation::{LeftNavTab, MainTab, NavCommand};
use kiwi_core::state::AppState;

use super::github_common::{select_branch_commands, select_issue_commands, select_pr_commands};
use crate::dock::tab::KiwiTab;

/// Sync TUI navigation state when a GitHub dock tab is focused (list fetch gating in core).
pub fn navigation_sync_commands(state: &AppState, tab: KiwiTab) -> Vec<AppCommand> {
    match tab {
        KiwiTab::GitHubIssues => github_issues_list_sync_commands(state),
        KiwiTab::Issues => github_issues_detail_sync_commands(state),
        KiwiTab::GitHubPrs => github_prs_detail_sync_commands(state),
        KiwiTab::GitLog => github_branches_detail_sync_commands(state),
        _ => Vec::new(),
    }
}

fn github_issues_list_sync_commands(state: &AppState) -> Vec<AppCommand> {
    let mut commands = Vec::new();
    if state.navigation.left_tab != LeftNavTab::Gh {
        commands.push(AppCommand::Navigation(NavCommand::SelectLeftTab(
            LeftNavTab::Gh,
        )));
    }
    // Preserve hub selection (Issues | PRs | Branches); do not force Issues here.
    commands
}

fn github_issues_detail_sync_commands(state: &AppState) -> Vec<AppCommand> {
    let mut commands = Vec::new();
    if state.navigation.main_tab != MainTab::Issues {
        commands.push(AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(
            MainTab::Issues,
        )));
    }
    if state.github.left_pane != GitHubLeftPane::Issues {
        commands.push(AppCommand::GitHubSelectLeftPane(
            GitHubLeftPane::Issues,
        ));
    }
    commands
}

fn github_branches_detail_sync_commands(state: &AppState) -> Vec<AppCommand> {
    let mut commands = Vec::new();
    if state.navigation.main_tab != MainTab::Branches {
        commands.push(AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(
            MainTab::Branches,
        )));
    }
    if state.github.left_pane != GitHubLeftPane::Branches {
        commands.push(AppCommand::GitHubSelectLeftPane(
            GitHubLeftPane::Branches,
        ));
    }
    commands
}

fn github_prs_detail_sync_commands(state: &AppState) -> Vec<AppCommand> {
    let mut commands = Vec::new();
    if state.navigation.main_tab != MainTab::Prs {
        commands.push(AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(
            MainTab::Prs,
        )));
    }
    if state.github.left_pane != GitHubLeftPane::Prs {
        commands.push(AppCommand::GitHubSelectLeftPane(GitHubLeftPane::Prs));
    }
    commands
}

/// Collect keyboard commands when a GitHub-related dock tab is focused.
pub fn collect_github_keyboard(ctx: &Context, tab: KiwiTab, state: &AppState) -> Vec<AppCommand> {
    if state.palette.open || ctx.wants_keyboard_input() {
        return Vec::new();
    }

    let Some(action) = ctx.input(detect_github_key_action) else {
        return Vec::new();
    };

    match tab {
        KiwiTab::GitHubIssues => match state.github.left_pane {
            GitHubLeftPane::Issues => issues_list_keyboard(action, state),
            GitHubLeftPane::Prs => prs_list_keyboard(action, state),
            GitHubLeftPane::Branches => branches_list_keyboard(action, state),
        },
        KiwiTab::Issues => issues_detail_keyboard(action, state),
        KiwiTab::GitHubPrs => prs_detail_keyboard(action, state),
        KiwiTab::GitLog => branches_detail_keyboard(action, state),
        _ => Vec::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GithubKeyAction {
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    Enter,
    Refresh,
    OpenBrowser,
}

fn detect_github_key_action(input: &egui::InputState) -> Option<GithubKeyAction> {
    if input.modifiers.command && input.key_pressed(Key::Enter) {
        return Some(GithubKeyAction::OpenBrowser);
    }

    if input.modifiers.any() {
        return None;
    }

    if input.key_pressed(Key::ArrowDown) {
        return Some(GithubKeyAction::MoveDown);
    }
    if input.key_pressed(Key::ArrowUp) {
        return Some(GithubKeyAction::MoveUp);
    }
    if input.key_pressed(Key::PageDown) {
        return Some(GithubKeyAction::PageDown);
    }
    if input.key_pressed(Key::PageUp) {
        return Some(GithubKeyAction::PageUp);
    }
    if input.key_pressed(Key::Enter) {
        return Some(GithubKeyAction::Enter);
    }
    if input.key_pressed(Key::R) {
        return Some(GithubKeyAction::Refresh);
    }

    None
}

fn issues_list_keyboard(action: GithubKeyAction, state: &AppState) -> Vec<AppCommand> {
    match action {
        GithubKeyAction::MoveDown => vec![AppCommand::GitHubMoveIssueSelection(1)],
        GithubKeyAction::MoveUp => vec![AppCommand::GitHubMoveIssueSelection(-1)],
        GithubKeyAction::Enter => enter_issue_commands(state),
        GithubKeyAction::Refresh => vec![AppCommand::GitHubRefresh],
        GithubKeyAction::OpenBrowser => vec![AppCommand::GitHubOpenInBrowser],
        GithubKeyAction::PageDown | GithubKeyAction::PageUp => Vec::new(),
    }
}

fn branches_list_keyboard(action: GithubKeyAction, state: &AppState) -> Vec<AppCommand> {
    match action {
        GithubKeyAction::MoveDown => vec![AppCommand::BranchMoveSelection(1)],
        GithubKeyAction::MoveUp => vec![AppCommand::BranchMoveSelection(-1)],
        GithubKeyAction::Enter => enter_branch_commands(state),
        GithubKeyAction::Refresh => vec![AppCommand::BranchRefresh],
        GithubKeyAction::OpenBrowser => Vec::new(),
        GithubKeyAction::PageDown | GithubKeyAction::PageUp => Vec::new(),
    }
}

fn branches_detail_keyboard(action: GithubKeyAction, _state: &AppState) -> Vec<AppCommand> {
    match action {
        GithubKeyAction::MoveDown => vec![AppCommand::BranchDetailScroll(1)],
        GithubKeyAction::MoveUp => vec![AppCommand::BranchDetailScroll(-1)],
        GithubKeyAction::Enter => vec![AppCommand::BranchCheckoutSelected],
        GithubKeyAction::Refresh => vec![AppCommand::BranchRefresh],
        GithubKeyAction::OpenBrowser => Vec::new(),
        GithubKeyAction::PageDown | GithubKeyAction::PageUp => Vec::new(),
    }
}

fn prs_list_keyboard(action: GithubKeyAction, state: &AppState) -> Vec<AppCommand> {
    match action {
        GithubKeyAction::MoveDown => vec![AppCommand::GitHubMovePrSelection(1)],
        GithubKeyAction::MoveUp => vec![AppCommand::GitHubMovePrSelection(-1)],
        GithubKeyAction::Enter => enter_pr_commands(state),
        GithubKeyAction::Refresh => vec![AppCommand::GitHubRefresh],
        GithubKeyAction::OpenBrowser => vec![AppCommand::GitHubOpenInBrowser],
        GithubKeyAction::PageDown | GithubKeyAction::PageUp => Vec::new(),
    }
}

fn issues_detail_keyboard(action: GithubKeyAction, state: &AppState) -> Vec<AppCommand> {
    match action {
        GithubKeyAction::MoveDown => vec![AppCommand::GitHubIssueDetailScroll(1)],
        GithubKeyAction::MoveUp => vec![AppCommand::GitHubIssueDetailScroll(-1)],
        GithubKeyAction::PageDown => vec![AppCommand::GitHubIssueDetailPageScroll(1)],
        GithubKeyAction::PageUp => vec![AppCommand::GitHubIssueDetailPageScroll(-1)],
        GithubKeyAction::Enter => enter_issue_commands(state),
        GithubKeyAction::Refresh => vec![AppCommand::GitHubRefresh],
        GithubKeyAction::OpenBrowser => vec![AppCommand::GitHubOpenInBrowser],
    }
}

fn prs_detail_keyboard(action: GithubKeyAction, state: &AppState) -> Vec<AppCommand> {
    match action {
        GithubKeyAction::MoveDown => vec![AppCommand::GitHubPrDetailScroll(1)],
        GithubKeyAction::MoveUp => vec![AppCommand::GitHubPrDetailScroll(-1)],
        GithubKeyAction::PageDown => vec![AppCommand::GitHubPrDetailPageScroll(1)],
        GithubKeyAction::PageUp => vec![AppCommand::GitHubPrDetailPageScroll(-1)],
        GithubKeyAction::Enter => enter_pr_commands(state),
        GithubKeyAction::Refresh => vec![AppCommand::GitHubRefresh],
        GithubKeyAction::OpenBrowser => vec![AppCommand::GitHubOpenInBrowser],
    }
}

fn enter_issue_commands(state: &AppState) -> Vec<AppCommand> {
    let Some(row_index) = issue_selected_row_index(&state.github) else {
        return Vec::new();
    };
    select_issue_commands(row_index).into()
}

fn enter_branch_commands(state: &AppState) -> Vec<AppCommand> {
    let Some(row_index) = branch_selected_row_index(&state.branches) else {
        return Vec::new();
    };
    select_branch_commands(row_index).to_vec()
}

fn enter_pr_commands(state: &AppState) -> Vec<AppCommand> {
    let Some(row_index) = pr_selected_row_index(&state.github) else {
        return Vec::new();
    };
    select_pr_commands(row_index).into()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::github::{Issue, IssueState};
    use kiwi_core::navigation::FocusTarget;
    use kiwi_core::state::{AppState, ViewportMetrics};
    use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        )
    }

    #[test]
    fn github_issues_list_sync_selects_gh_left_and_preserves_hub_pane() {
        let mut state = test_state();
        state.github.left_pane = GitHubLeftPane::Branches;
        let commands = navigation_sync_commands(&state, KiwiTab::GitHubIssues);
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Gh))
        )));
        assert!(!commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectMainTab(_))
                | AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(_))
                | AppCommand::GitHubSelectLeftPane(_)
        )));
    }

    #[test]
    fn github_issues_list_sync_does_not_reset_prs_hub_selection() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;
        state.github.left_pane = GitHubLeftPane::Prs;
        let commands = navigation_sync_commands(&state, KiwiTab::GitHubIssues);
        assert!(commands.is_empty());
    }

    #[test]
    fn issues_detail_sync_preserves_left_tab() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Search;
        let commands = navigation_sync_commands(&state, KiwiTab::Issues);
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(MainTab::Issues))
        )));
        assert!(!commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectLeftTab(_))
        )));
    }

    #[test]
    fn list_focus_arrow_down_moves_issue_selection() {
        let state = test_state();
        let commands = issues_list_keyboard(GithubKeyAction::MoveDown, &state);
        assert_eq!(commands, vec![AppCommand::GitHubMoveIssueSelection(1)]);
    }

    #[test]
    fn detail_tab_arrow_down_scrolls_issue_detail() {
        let state = test_state();
        let commands = issues_detail_keyboard(GithubKeyAction::MoveDown, &state);
        assert_eq!(commands, vec![AppCommand::GitHubIssueDetailScroll(1)]);
    }

    #[test]
    fn enter_on_selected_issue_opens_issues_main_tab() {
        let mut state = test_state();
        state.github.issues = vec![Issue {
            number: 7,
            title: "Bug".to_string(),
            state: IssueState::Open,
            labels: Vec::new(),
            assignees: Vec::new(),
        }];
        state.github.selected_issue = Some(7);
        let commands = enter_issue_commands(&state);
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Main))
        )));
    }
}
