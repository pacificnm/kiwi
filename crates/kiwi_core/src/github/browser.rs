use crate::navigation::{LeftNavTab, MainTab, NavigationState};
use crate::state::GitHubState;

use super::types::{GitHubBrowserTarget, GitHubLeftPane};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitHubBrowserKind {
    Issue,
    PullRequest,
}

pub fn browser_target_kind(
    navigation: &NavigationState,
    github: &GitHubState,
) -> GitHubBrowserKind {
    if navigation.main_tab == MainTab::Prs
        || (navigation.left_tab == LeftNavTab::Gh && github.left_pane == GitHubLeftPane::Prs)
    {
        GitHubBrowserKind::PullRequest
    } else {
        GitHubBrowserKind::Issue
    }
}

pub fn resolve_browser_target(
    navigation: &NavigationState,
    github: &GitHubState,
) -> Option<GitHubBrowserTarget> {
    if !github.auth_ok {
        return None;
    }

    match browser_target_kind(navigation, github) {
        GitHubBrowserKind::PullRequest => {
            github.selected_pr.map(GitHubBrowserTarget::PullRequest)
        }
        GitHubBrowserKind::Issue => github.selected_issue.map(GitHubBrowserTarget::Issue),
    }
}

pub fn missing_browser_target_message(
    navigation: &NavigationState,
    github: &GitHubState,
) -> &'static str {
    match browser_target_kind(navigation, github) {
        GitHubBrowserKind::PullRequest => "Select a pull request in the GH left list first",
        GitHubBrowserKind::Issue => "Select an issue in the GH left list first",
    }
}

pub fn scroll_issue_detail(
    scroll_offset: &mut usize,
    delta: i32,
    line_count: usize,
    viewport_rows: usize,
) {
    if viewport_rows == 0 {
        return;
    }

    let max_offset = line_count.saturating_sub(viewport_rows);
    let current = *scroll_offset as i32;
    let next = (current + delta).clamp(0, max_offset as i32);
    *scroll_offset = usize::try_from(next).unwrap_or(0);
}

pub fn page_scroll_issue_detail(
    scroll_offset: &mut usize,
    delta: i32,
    line_count: usize,
    viewport_rows: usize,
) {
    if viewport_rows == 0 {
        return;
    }

    let page = i32::try_from(viewport_rows.saturating_sub(1).max(1)).unwrap_or(1);
    scroll_issue_detail(scroll_offset, delta * page, line_count, viewport_rows);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::{LeftNavTab, MainTab, NavCommand, NavigationState};
    use crate::state::GitHubState;

    #[test]
    fn scroll_issue_detail_clamps_to_content() {
        let mut offset = 0;
        scroll_issue_detail(&mut offset, 100, 10, 4);
        assert_eq!(offset, 6);
        scroll_issue_detail(&mut offset, -100, 10, 4);
        assert_eq!(offset, 0);
    }

    #[test]
    fn resolve_browser_target_returns_selected_issue() {
        let mut navigation = NavigationState::default();
        let mut github = GitHubState::default();
        github.auth_ok = true;
        github.selected_issue = Some(42);
        navigation.apply(NavCommand::SelectMainTab(MainTab::Issues));

        let target = resolve_browser_target(&navigation, &github).expect("target");
        assert_eq!(target, GitHubBrowserTarget::Issue(42));
    }

    #[test]
    fn resolve_browser_target_returns_selected_pr_on_prs_tab() {
        let mut navigation = NavigationState::default();
        let mut github = GitHubState::default();
        github.auth_ok = true;
        github.selected_pr = Some(17);
        navigation.apply(NavCommand::SelectMainTab(MainTab::Prs));

        let target = resolve_browser_target(&navigation, &github).expect("target");
        assert_eq!(target, GitHubBrowserTarget::PullRequest(17));
    }

    #[test]
    fn browser_target_kind_follows_gh_left_hub() {
        let mut navigation = NavigationState::default();
        let mut github = GitHubState::default();
        github.left_pane = GitHubLeftPane::Prs;
        navigation.apply(NavCommand::SelectLeftTab(LeftNavTab::Gh));
        assert_eq!(
            browser_target_kind(&navigation, &github),
            GitHubBrowserKind::PullRequest
        );
    }
}
