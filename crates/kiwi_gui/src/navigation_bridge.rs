//! Maps TUI [`NavigationState`] to GUI [`KiwiTab`] dock actions (SPEC-022 / #187).

use kiwi_core::github::GitHubLeftPane;
use kiwi_core::navigation::{FocusTarget, LeftNavTab, MainTab, NavigationState};

use crate::dock::{DockShell, KiwiTab};

/// Open and focus dock tab(s) that correspond to the current navigation focus.
pub fn sync_dock_from_navigation(
    dock: &mut DockShell,
    nav: &NavigationState,
    gh_pane: GitHubLeftPane,
) {
    let Some(tab) = primary_tab_for_navigation(nav, gh_pane) else {
        return;
    };
    dock.show_tab(tab);
}

#[must_use]
pub fn primary_tab_for_navigation(
    nav: &NavigationState,
    gh_pane: GitHubLeftPane,
) -> Option<KiwiTab> {
    match nav.focus {
        FocusTarget::Shell => Some(KiwiTab::Terminal),
        FocusTarget::CommandPalette => None,
        FocusTarget::Left => kiwi_tab_for_left(nav.left_tab, nav.main_tab, gh_pane),
        FocusTarget::Main => kiwi_tab_for_main(nav.main_tab, gh_pane),
    }
}

#[must_use]
fn kiwi_tab_for_left(left: LeftNavTab, _main: MainTab, _gh_pane: GitHubLeftPane) -> Option<KiwiTab> {
    match left {
        LeftNavTab::Files => Some(KiwiTab::Explorer),
        LeftNavTab::Git => Some(KiwiTab::GitStatus),
        LeftNavTab::Gh => Some(KiwiTab::GitHubIssues),
        LeftNavTab::Search => Some(KiwiTab::Search),
    }
}

#[must_use]
fn kiwi_tab_for_main(main: MainTab, _gh_pane: GitHubLeftPane) -> Option<KiwiTab> {
    match main {
        MainTab::Agent => Some(KiwiTab::Agent),
        MainTab::Issues => Some(KiwiTab::Issues),
        MainTab::Prs => Some(KiwiTab::GitHubPrs),
        MainTab::Branches => Some(KiwiTab::GitHubIssues),
        MainTab::Diff => Some(KiwiTab::GitDiff),
        MainTab::Preview => Some(KiwiTab::Preview),
        MainTab::Logs => Some(KiwiTab::Logs),
        MainTab::Settings => Some(KiwiTab::Config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kiwi_core::navigation::NavCommand;

    fn nav_with(commands: &[NavCommand]) -> NavigationState {
        let mut nav = NavigationState::default();
        for command in commands {
            nav.apply(*command);
        }
        nav
    }

    #[test]
    fn shell_focus_opens_terminal() {
        let nav = nav_with(&[NavCommand::SetFocus(FocusTarget::Shell)]);
        assert_eq!(
            primary_tab_for_navigation(&nav, GitHubLeftPane::Issues),
            Some(KiwiTab::Terminal)
        );
    }

    #[test]
    fn goto_agent_opens_agent_tab() {
        let nav = nav_with(&[
            NavCommand::SelectMainTab(MainTab::Agent),
            NavCommand::SetFocus(FocusTarget::Main),
        ]);
        assert_eq!(
            primary_tab_for_navigation(&nav, GitHubLeftPane::Issues),
            Some(KiwiTab::Agent)
        );
    }

    #[test]
    fn goto_issues_opens_issues_detail_tab() {
        let nav = nav_with(&[
            NavCommand::SelectLeftTab(LeftNavTab::Gh),
            NavCommand::SelectMainTab(MainTab::Issues),
            NavCommand::SetFocus(FocusTarget::Main),
        ]);
        assert_eq!(
            primary_tab_for_navigation(&nav, GitHubLeftPane::Issues),
            Some(KiwiTab::Issues)
        );
    }

    #[test]
    fn gh_left_focus_opens_github_list_tab() {
        let nav = nav_with(&[
            NavCommand::SelectLeftTab(LeftNavTab::Gh),
            NavCommand::SetFocus(FocusTarget::Left),
        ]);
        assert_eq!(
            primary_tab_for_navigation(&nav, GitHubLeftPane::Issues),
            Some(KiwiTab::GitHubIssues)
        );
    }

    #[test]
    fn goto_files_opens_explorer() {
        let nav = nav_with(&[
            NavCommand::SelectLeftTab(LeftNavTab::Files),
            NavCommand::SetFocus(FocusTarget::Left),
        ]);
        assert_eq!(
            primary_tab_for_navigation(&nav, GitHubLeftPane::Issues),
            Some(KiwiTab::Explorer)
        );
    }

    #[test]
    fn goto_search_opens_search_tab() {
        let nav = nav_with(&[
            NavCommand::SelectLeftTab(LeftNavTab::Search),
            NavCommand::SetFocus(FocusTarget::Left),
        ]);
        assert_eq!(
            primary_tab_for_navigation(&nav, GitHubLeftPane::Issues),
            Some(KiwiTab::Search)
        );
    }

    #[test]
    fn sync_dock_opens_expected_tab() {
        let nav = nav_with(&[
            NavCommand::SelectMainTab(MainTab::Logs),
            NavCommand::SetFocus(FocusTarget::Main),
        ]);
        let mut dock = DockShell::new();
        dock.close_tab(KiwiTab::Logs);
        assert!(!dock.is_tab_open(KiwiTab::Logs));

        sync_dock_from_navigation(&mut dock, &nav, GitHubLeftPane::Issues);
        assert!(dock.is_tab_open(KiwiTab::Logs));
    }
}
