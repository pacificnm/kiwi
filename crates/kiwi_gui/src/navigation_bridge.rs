//! Maps TUI [`NavigationState`] to GUI [`KiwiTab`] dock actions (SPEC-022 / #187).

use kiwi_core::events::AppCommand;
use kiwi_core::github::GitHubLeftPane;
use kiwi_core::navigation::{FocusTarget, LeftNavTab, MainTab, NavCommand, NavigationState};

use crate::dock::{DockShell, KiwiTab};

/// Navigation commands to apply when the user selects a dock tab.
#[must_use]
pub fn navigation_commands_for_dock_tab(tab: KiwiTab) -> Vec<AppCommand> {
    match tab {
        KiwiTab::Explorer => left_focus(LeftNavTab::Files),
        KiwiTab::GitStatus => left_focus(LeftNavTab::Git),
        KiwiTab::GitHubIssues => left_focus(LeftNavTab::Gh),
        KiwiTab::Search => left_focus(LeftNavTab::Search),
        KiwiTab::Agent => main_focus(MainTab::Agent),
        KiwiTab::Issues => main_focus(MainTab::Issues),
        KiwiTab::GitHubPrs => main_focus(MainTab::Prs),
        KiwiTab::GitDiff => main_focus(MainTab::Diff),
        KiwiTab::Preview => main_focus(MainTab::Preview),
        KiwiTab::Logs => main_focus(MainTab::Logs),
        KiwiTab::Config => main_focus(MainTab::Settings),
        KiwiTab::Plugins => main_focus(MainTab::Plugins),
        KiwiTab::Terminal => vec![AppCommand::Navigation(NavCommand::SetFocus(
            FocusTarget::Shell,
        ))],
        KiwiTab::GitLog => Vec::new(),
    }
}

fn left_focus(left: LeftNavTab) -> Vec<AppCommand> {
    vec![
        AppCommand::Navigation(NavCommand::SelectLeftTab(left)),
        AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Left)),
    ]
}

fn main_focus(main: MainTab) -> Vec<AppCommand> {
    vec![
        AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(main)),
        AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Main)),
    ]
}

/// Ensure the dock tab for the focused region is open; do not manage the other region.
pub fn sync_dock_from_navigation(
    dock: &mut DockShell,
    nav: &NavigationState,
    gh_pane: GitHubLeftPane,
) {
    match nav.focus {
        FocusTarget::Shell => {
            dock.ensure_tab(KiwiTab::Terminal, true);
        }
        FocusTarget::CommandPalette => {}
        FocusTarget::Left => {
            if let Some(tab) = kiwi_tab_for_left(nav.left_tab, nav.main_tab, gh_pane) {
                dock.ensure_tab(tab, true);
            }
        }
        FocusTarget::Main => {
            if let Some(tab) = kiwi_tab_for_main(nav.main_tab, gh_pane) {
                dock.ensure_tab(tab, true);
            }
        }
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
        MainTab::Branches => Some(KiwiTab::GitLog),
        MainTab::Diff => Some(KiwiTab::GitDiff),
        MainTab::Preview => Some(KiwiTab::Preview),
        MainTab::Logs => Some(KiwiTab::Logs),
        MainTab::Settings => Some(KiwiTab::Config),
        MainTab::Plugins => Some(KiwiTab::Plugins),
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

    fn sync_opens(nav: NavigationState, tab: KiwiTab) {
        let mut dock = DockShell::new();
        dock.close_tab(tab);
        assert!(!dock.is_tab_open(tab));
        sync_dock_from_navigation(&mut dock, &nav, GitHubLeftPane::Issues);
        assert!(dock.is_tab_open(tab));
    }

    #[test]
    fn shell_focus_opens_terminal() {
        sync_opens(nav_with(&[NavCommand::SetFocus(FocusTarget::Shell)]), KiwiTab::Terminal);
    }

    #[test]
    fn goto_agent_opens_agent_tab() {
        sync_opens(
            nav_with(&[NavCommand::SelectMainTab(MainTab::Agent), NavCommand::SetFocus(FocusTarget::Main)]),
            KiwiTab::Agent,
        );
    }

    #[test]
    fn goto_issues_opens_issues_detail_tab() {
        sync_opens(
            nav_with(&[
                NavCommand::SelectLeftTab(LeftNavTab::Gh),
                NavCommand::SelectMainTab(MainTab::Issues),
                NavCommand::SetFocus(FocusTarget::Main),
            ]),
            KiwiTab::Issues,
        );
    }

    #[test]
    fn gh_left_focus_opens_github_list_tab() {
        sync_opens(
            nav_with(&[NavCommand::SelectLeftTab(LeftNavTab::Gh), NavCommand::SetFocus(FocusTarget::Left)]),
            KiwiTab::GitHubIssues,
        );
    }

    #[test]
    fn goto_files_opens_explorer() {
        sync_opens(
            nav_with(&[NavCommand::SelectLeftTab(LeftNavTab::Files), NavCommand::SetFocus(FocusTarget::Left)]),
            KiwiTab::Explorer,
        );
    }

    #[test]
    fn goto_branches_opens_git_log_tab() {
        sync_opens(
            nav_with(&[NavCommand::SelectMainTab(MainTab::Branches), NavCommand::SetFocus(FocusTarget::Main)]),
            KiwiTab::GitLog,
        );
    }

    #[test]
    fn goto_search_opens_search_tab() {
        sync_opens(
            nav_with(&[NavCommand::SelectLeftTab(LeftNavTab::Search), NavCommand::SetFocus(FocusTarget::Left)]),
            KiwiTab::Search,
        );
    }

    #[test]
    fn search_dock_tab_sets_left_focus() {
        let commands = navigation_commands_for_dock_tab(KiwiTab::Search);
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Search))
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Left))
        )));
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

    #[test]
    fn issues_dock_tab_uses_unpaired_main_select() {
        let commands = navigation_commands_for_dock_tab(KiwiTab::Issues);
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(MainTab::Issues))
        )));
    }

    #[test]
    fn left_focus_only_opens_left_region_tab() {
        let nav = nav_with(&[
            NavCommand::SelectMainTabUnpaired(MainTab::Issues),
            NavCommand::SelectLeftTab(LeftNavTab::Search),
            NavCommand::SetFocus(FocusTarget::Left),
        ]);
        let mut dock = DockShell::new();
        dock.close_tab(KiwiTab::Search);
        assert!(!dock.is_tab_open(KiwiTab::Search));

        sync_dock_from_navigation(&mut dock, &nav, GitHubLeftPane::Issues);
        assert!(dock.is_tab_open(KiwiTab::Search));
        assert_eq!(dock.focused_tab(), Some(KiwiTab::Search));
    }
}
