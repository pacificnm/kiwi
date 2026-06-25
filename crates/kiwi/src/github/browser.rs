use std::path::Path;
use std::process::Command;

use crate::navigation::{LeftNavTab, MainTab};
use crate::state::AppState;

use super::hub::GitHubLeftPane;
use super::issue::command_on_path;
use super::IssueActionResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitHubBrowserTarget {
    Issue(u32),
    PullRequest(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitHubBrowserKind {
    Issue,
    PullRequest,
}

impl GitHubBrowserTarget {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Issue(_) => "issue",
            Self::PullRequest(_) => "pull request",
        }
    }

    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::Issue(number) | Self::PullRequest(number) => number,
        }
    }
}

pub fn browser_target_kind(state: &AppState) -> GitHubBrowserKind {
    if state.navigation.main_tab == MainTab::Prs
        || (state.navigation.left_tab == LeftNavTab::Gh
            && state.github.left_pane == GitHubLeftPane::Prs)
    {
        GitHubBrowserKind::PullRequest
    } else {
        GitHubBrowserKind::Issue
    }
}

pub fn resolve_browser_target(state: &AppState) -> Option<GitHubBrowserTarget> {
    if !state.github.auth_ok {
        return None;
    }

    match browser_target_kind(state) {
        GitHubBrowserKind::PullRequest => state
            .github
            .selected_pr
            .and_then(|number| u32::try_from(number).ok())
            .map(GitHubBrowserTarget::PullRequest),
        GitHubBrowserKind::Issue => state
            .github
            .selected_issue
            .and_then(|number| u32::try_from(number).ok())
            .map(GitHubBrowserTarget::Issue),
    }
}

pub fn missing_browser_target_message(state: &AppState) -> &'static str {
    match browser_target_kind(state) {
        GitHubBrowserKind::PullRequest => "Select a pull request in the GH left list first",
        GitHubBrowserKind::Issue => "Select an issue in the GH left list first",
    }
}

pub fn open_in_browser(
    repo_root: &Path,
    command: &str,
    target: GitHubBrowserTarget,
) -> IssueActionResult {
    if !command_on_path(command) {
        return IssueActionResult {
            success: false,
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
            detail: None,
        };
    }

    let (kind, number) = match target {
        GitHubBrowserTarget::Issue(number) => ("issue", number),
        GitHubBrowserTarget::PullRequest(number) => ("pr", number),
    };

    let output = Command::new(command)
        .args([kind, "view", &number.to_string(), "--web"])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => IssueActionResult {
            success: true,
            error: None,
            detail: None,
        },
        Ok(result) => IssueActionResult {
            success: false,
            error: Some(format_browser_failure(kind, &result.stderr, &result.stdout)),
            detail: None,
        },
        Err(err) => IssueActionResult {
            success: false,
            error: Some(format!("Failed to run `{command} {kind} view --web`: {err}")),
            detail: None,
        },
    }
}

fn format_browser_failure(kind: &str, stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }

    format!("gh {kind} view --web failed")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::github::GitHubLeftPane;
    use crate::layout::compute_layout;
    use crate::navigation::{LeftNavTab, MainTab, NavCommand};
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

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
            compute_layout(120, 40, 30).expect("layout"),
        )
    }

    #[test]
    fn resolve_browser_target_returns_selected_issue() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.selected_issue = Some(42);
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));

        let target = resolve_browser_target(&state).expect("target");
        assert_eq!(target, GitHubBrowserTarget::Issue(42));
    }

    #[test]
    fn resolve_browser_target_returns_selected_pr_on_prs_tab() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.selected_pr = Some(17);
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Prs));

        let target = resolve_browser_target(&state).expect("target");
        assert_eq!(target, GitHubBrowserTarget::PullRequest(17));
    }

    #[test]
    fn browser_target_kind_follows_gh_left_hub() {
        let mut state = test_state();
        state.github.left_pane = GitHubLeftPane::Prs;
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Gh));
        assert_eq!(browser_target_kind(&state), GitHubBrowserKind::PullRequest);
    }

    #[test]
    fn open_in_browser_requires_gh_on_path() {
        let result = open_in_browser(
            Path::new("."),
            "gh-nonexistent-kiwi-test",
            GitHubBrowserTarget::Issue(1),
        );
        assert!(!result.success);
        assert!(result
            .error
            .as_ref()
            .is_some_and(|message| message.contains("not found on PATH")));
    }
}
