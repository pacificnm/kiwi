//! Status bar snapshot and line formatting (SPEC-019).

use crate::agent::AgentStatus;
use crate::state::AppState;

pub const BRAND: &str = "Kiwi";
pub const SEPARATOR: &str = " | ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarSnapshot {
    pub remote_repo: Option<String>,
    pub root_name: String,
    pub branch: String,
    pub agent_label: String,
    pub git_label: String,
    pub issue_label: Option<String>,
    pub agent_status: AgentStatus,
    pub agent_running: bool,
    pub git_modified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarSegments {
    pub repo: Option<String>,
    pub root: String,
    pub branch: String,
    pub agent: String,
    pub git: String,
    pub issue: Option<String>,
}

#[must_use]
pub fn compute_status_bar(state: &AppState) -> StatusBarSnapshot {
    compute_status_bar_fields(
        &state.config,
        &state.git,
        &state.github,
        &state.status_bar,
        &state.agent_manager,
    )
}

#[must_use]
pub fn compute_status_bar_fields(
    config: &crate::config::ResolvedConfig,
    git: &crate::state::GitState,
    github: &crate::state::GitHubState,
    status_bar: &crate::state::StatusBarState,
    agent_manager: &crate::agent::AgentManager,
) -> StatusBarSnapshot {
    let git_modified = git.changed_count() > 0;

    StatusBarSnapshot {
        remote_repo: git.remote_repo.clone(),
        root_name: status_bar.root_name.clone(),
        branch: git.branch.clone().unwrap_or_else(|| "no git".to_string()),
        agent_label: agent_manager.status_bar_label().to_string(),
        git_label: git_label_from_count(git.changed_count()),
        issue_label: issue_label_from_state(config, git, github),
        agent_status: agent_manager.active_pty().status,
        agent_running: agent_manager.active_pty().running,
        git_modified,
    }
}

#[must_use]
pub fn format_status_line(snapshot: &StatusBarSnapshot, width: u16) -> String {
    join_status_bar_segments(&fit_status_bar_segments(snapshot, width))
}

#[must_use]
pub fn fit_status_bar_segments(snapshot: &StatusBarSnapshot, width: u16) -> StatusBarSegments {
    let width = width as usize;
    let mut repo = repo_segment(snapshot.remote_repo.as_deref());
    let mut root = root_segment(&snapshot.root_name);
    let mut branch = branch_segment(&snapshot.branch);
    let mut agent = snapshot.agent_label.clone();
    let mut git = snapshot.git_label.clone();
    let issue = snapshot.issue_label.clone();
    let mut include_issue = issue.is_some();

    if width == 0 {
        return StatusBarSegments {
            repo: None,
            root: String::new(),
            branch: String::new(),
            agent: String::new(),
            git: String::new(),
            issue: None,
        };
    }

    loop {
        let fitted = StatusBarSegments {
            repo: repo.clone(),
            root: root.clone(),
            branch: branch.clone(),
            agent: agent.clone(),
            git: git.clone(),
            issue: if include_issue { issue.clone() } else { None },
        };
        let line = join_status_bar_segments(&fitted);
        if display_width(&line) <= width {
            return fitted;
        }

        if include_issue {
            include_issue = false;
            continue;
        }

        let mut changed = false;
        for label in [&mut branch, &mut agent, &mut git] {
            if label.chars().count() > 4 {
                *label = truncate_to_chars(label, label.chars().count() - 1);
                changed = true;
            }
        }
        if changed {
            continue;
        }

        for label in [&mut root] {
            if label.chars().count() > 4 {
                *label = truncate_to_chars(label, label.chars().count() - 1);
                changed = true;
            }
        }
        if let Some(ref mut repo_label) = repo {
            if repo_label.chars().count() > 4 {
                *repo_label = truncate_to_chars(repo_label, repo_label.chars().count() - 1);
                changed = true;
            }
        }
        if changed {
            continue;
        }

        return StatusBarSegments {
            repo: repo.map(|value| truncate_to_chars(&value, width.min(value.chars().count()))),
            root: String::new(),
            branch: String::new(),
            agent: String::new(),
            git: String::new(),
            issue: None,
        };
    }
}

#[must_use]
pub fn join_status_bar_segments(segments: &StatusBarSegments) -> String {
    let mut parts = vec![BRAND.to_string()];
    if let Some(repo) = &segments.repo {
        parts.push(repo.clone());
    }
    parts.extend([
        segments.root.clone(),
        segments.branch.clone(),
        segments.agent.clone(),
        segments.git.clone(),
    ]);
    if let Some(issue) = &segments.issue {
        parts.push(issue.clone());
    }
    parts.join(SEPARATOR)
}

#[must_use]
pub fn display_width(value: &str) -> usize {
    value.chars().count()
}

fn labeled_segment(prefix: &str, value: &str) -> String {
    format!("{prefix}: {value}")
}

fn repo_segment(remote_repo: Option<&str>) -> Option<String> {
    remote_repo.map(|slug| labeled_segment("Repo", slug))
}

fn root_segment(root_name: &str) -> String {
    labeled_segment("Root", root_name)
}

fn branch_segment(branch: &str) -> String {
    labeled_segment("Branch", branch)
}

fn git_label_from_count(count: usize) -> String {
    match count {
        0 => "Clean".to_string(),
        1 => "1 Modified".to_string(),
        n => format!("{n} Modified"),
    }
}

fn issue_label_from_state(
    config: &crate::config::ResolvedConfig,
    git: &crate::state::GitState,
    github: &crate::state::GitHubState,
) -> Option<String> {
    if !config.status_bar.show_issue {
        return None;
    }

    if let Some(number) = github.selected_issue {
        return Some(format!("#{number}"));
    }

    git.branch
        .as_deref()
        .and_then(issue_from_branch)
        .map(|number| format!("#{number}"))
}

fn issue_from_branch(branch: &str) -> Option<u64> {
    let suffix = branch.rsplit('/').next()?;
    if suffix.chars().all(|ch| ch.is_ascii_digit()) {
        suffix.parse().ok()
    } else {
        None
    }
}

fn truncate_to_chars(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_string();
    }

    if max_chars == 1 {
        return "…".to_string();
    }

    let prefix: String = value.chars().take(max_chars - 1).collect();
    format!("{prefix}…")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::agent::AgentStatus;
    use crate::config::ResolvedConfig;
    use crate::git::{GitFileEntry, GitFileStatus};
    use crate::state::{AppState, ViewportMetrics};
    use crate::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("/tmp/cityartwalks"),
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
    fn compute_status_bar_uses_domain_defaults() {
        let snapshot = compute_status_bar(&test_state());
        assert_eq!(snapshot.root_name, "cityartwalks");
        assert_eq!(snapshot.remote_repo, None);
        assert_eq!(snapshot.branch, "no git");
        assert_eq!(snapshot.agent_label, "Agent Idle");
        assert_eq!(snapshot.git_label, "Clean");
        assert_eq!(snapshot.issue_label, None);
    }

    #[test]
    fn compute_status_bar_reflects_git_agent_and_issue_state() {
        let mut state = test_state();
        state.git.remote_repo = Some("org/cityartwalks".to_string());
        state.git.branch = Some("feature/42".to_string());
        state.git.file_entries = vec![
            GitFileEntry {
                path: "a.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "b.rs".to_string(),
                status: GitFileStatus::Added,
            },
            GitFileEntry {
                path: "c.rs".to_string(),
                status: GitFileStatus::Untracked,
            },
        ];
        state.active_agent_mut().running = true;
        state.active_agent_mut().status = AgentStatus::Executing;
        state.active_agent_mut().refresh_status_bar_label();
        state.agent_manager.refresh_status_label();
        state.github.selected_issue = Some(42);

        let snapshot = compute_status_bar(&state);
        assert_eq!(snapshot.remote_repo.as_deref(), Some("org/cityartwalks"));
        assert_eq!(snapshot.branch, "feature/42");
        assert_eq!(snapshot.agent_label, "Agent Executing");
        assert_eq!(snapshot.git_label, "3 Modified");
        assert_eq!(snapshot.issue_label, Some("#42".to_string()));
    }

    #[test]
    fn issue_label_can_be_derived_from_branch() {
        let mut state = test_state();
        state.git.branch = Some("feature/42".to_string());

        let snapshot = compute_status_bar(&state);
        assert_eq!(snapshot.issue_label, Some("#42".to_string()));
    }

    #[test]
    fn show_issue_config_disables_issue_segment() {
        let mut state = test_state();
        state.git.branch = Some("feature/42".to_string());
        state.config.status_bar.show_issue = false;

        let snapshot = compute_status_bar(&state);
        assert_eq!(snapshot.issue_label, None);
    }

    #[test]
    fn format_status_line_omits_repo_when_no_remote() {
        let snapshot = StatusBarSnapshot {
            remote_repo: None,
            root_name: "kiwi".to_string(),
            branch: "main".to_string(),
            agent_label: "Agent Idle".to_string(),
            git_label: "Clean".to_string(),
            issue_label: None,
            agent_status: AgentStatus::Idle,
            agent_running: false,
            git_modified: false,
        };

        let line = format_status_line(&snapshot, 120);
        assert!(!line.contains("Repo:"));
        assert_eq!(
            line,
            "Kiwi | Root: kiwi | Branch: main | Agent Idle | Clean"
        );
    }

    #[test]
    fn format_status_line_matches_spec_example_at_120_cols() {
        let snapshot = StatusBarSnapshot {
            remote_repo: Some("org/cityartwalks".to_string()),
            root_name: "cityartwalks".to_string(),
            branch: "feature/42".to_string(),
            agent_label: "Agent Executing".to_string(),
            git_label: "3 Modified".to_string(),
            issue_label: Some("#42".to_string()),
            agent_status: AgentStatus::Executing,
            agent_running: true,
            git_modified: true,
        };

        let line = format_status_line(&snapshot, 120);
        assert_eq!(
            line,
            "Kiwi | Repo: org/cityartwalks | Root: cityartwalks | Branch: feature/42 | Agent Executing | 3 Modified | #42"
        );
    }

    #[test]
    fn format_status_line_distinguishes_brand_from_root_folder() {
        let snapshot = StatusBarSnapshot {
            remote_repo: Some("pacificnm/kiwi".to_string()),
            root_name: "kiwi".to_string(),
            branch: "main".to_string(),
            agent_label: "Agent Idle".to_string(),
            git_label: "Clean".to_string(),
            issue_label: None,
            agent_status: AgentStatus::Idle,
            agent_running: false,
            git_modified: false,
        };

        let line = format_status_line(&snapshot, 120);
        assert!(line.contains("Repo: pacificnm/kiwi"));
        assert!(line.contains("Root: kiwi"));
        assert!(line.contains("Branch: main"));
    }

    #[test]
    fn format_status_line_respects_display_width() {
        let snapshot = StatusBarSnapshot {
            remote_repo: Some("org/cityartwalks".to_string()),
            root_name: "cityartwalks".to_string(),
            branch: "feature/very-long-branch-name".to_string(),
            agent_label: "Agent Executing".to_string(),
            git_label: "2 Modified".to_string(),
            issue_label: Some("#99".to_string()),
            agent_status: AgentStatus::Executing,
            agent_running: true,
            git_modified: true,
        };

        let line = format_status_line(&snapshot, 80);
        assert!(display_width(&line) <= 80);
        assert!(!line.contains("#99"));
    }

    #[test]
    fn format_status_line_never_returns_empty_for_nonzero_width() {
        let snapshot = StatusBarSnapshot {
            remote_repo: None,
            root_name: "repo".to_string(),
            branch: "branch".to_string(),
            agent_label: "Agent Idle".to_string(),
            git_label: "Clean".to_string(),
            issue_label: None,
            agent_status: AgentStatus::Idle,
            agent_running: false,
            git_modified: false,
        };

        let line = format_status_line(&snapshot, 20);
        assert!(!line.is_empty());
        assert!(line.starts_with("Kiwi"));
    }
}
