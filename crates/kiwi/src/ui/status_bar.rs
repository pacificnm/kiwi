use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::agent::AgentStatus;
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

const BRAND: &str = "Kiwi";
const SEPARATOR: &str = " | ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarSnapshot {
    pub repo_name: String,
    pub branch: String,
    pub agent_label: String,
    pub git_label: String,
    pub issue_label: Option<String>,
    pub agent_status: AgentStatus,
    pub agent_running: bool,
    pub git_modified: bool,
}

pub fn compute_status_bar(state: &AppState) -> StatusBarSnapshot {
    let git_modified = !state.git.modified_files.is_empty();

    StatusBarSnapshot {
        repo_name: state.status_bar.repo_name.clone(),
        branch: branch_label(state),
        agent_label: agent_label(state),
        git_label: git_label(state),
        issue_label: issue_label(state),
        agent_status: state.agent.status,
        agent_running: state.agent.running,
        git_modified,
    }
}

fn branch_label(state: &AppState) -> String {
    state
        .git
        .branch
        .clone()
        .unwrap_or_else(|| "no git".to_string())
}

fn agent_label(state: &AppState) -> String {
    state
        .agent
        .status
        .status_bar_label(state.agent.running)
        .to_string()
}

fn git_label(state: &AppState) -> String {
    let count = state.git.modified_files.len();
    match count {
        0 => "Clean".to_string(),
        1 => "1 Modified".to_string(),
        n => format!("{n} Modified"),
    }
}

fn issue_label(state: &AppState) -> Option<String> {
    if !state.config.status_bar.show_issue {
        return None;
    }

    if let Some(number) = state.github.selected_issue {
        return Some(format!("#{number}"));
    }

    state
        .git
        .branch
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

pub(crate) fn display_width(value: &str) -> usize {
    value.chars().count()
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn format_status_line(snapshot: &StatusBarSnapshot, width: u16) -> String {
    join_segments(&fit_segments(snapshot, width))
}

struct FittedSegments {
    repo: String,
    branch: String,
    agent: String,
    git: String,
    issue: Option<String>,
}

fn fit_segments(snapshot: &StatusBarSnapshot, width: u16) -> FittedSegments {
    let width = width as usize;
    let mut repo = snapshot.repo_name.clone();
    let mut branch = snapshot.branch.clone();
    let mut agent = snapshot.agent_label.clone();
    let mut git = snapshot.git_label.clone();
    let issue = snapshot.issue_label.clone();
    let mut include_issue = issue.is_some();

    if width == 0 {
        return FittedSegments {
            repo: String::new(),
            branch: String::new(),
            agent: String::new(),
            git: String::new(),
            issue: None,
        };
    }

    loop {
        let fitted = FittedSegments {
            repo: repo.clone(),
            branch: branch.clone(),
            agent: agent.clone(),
            git: git.clone(),
            issue: if include_issue { issue.clone() } else { None },
        };
        let line = join_segments(&fitted);
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

        if repo.chars().count() > 4 {
            repo = truncate_to_chars(&repo, repo.chars().count() - 1);
            continue;
        }

        return FittedSegments {
            repo: truncate_to_chars(&repo, width.min(repo.chars().count())),
            branch: String::new(),
            agent: String::new(),
            git: String::new(),
            issue: None,
        };
    }
}

fn join_segments(segments: &FittedSegments) -> String {
    let mut parts = vec![
        BRAND,
        segments.repo.as_str(),
        segments.branch.as_str(),
        segments.agent.as_str(),
        segments.git.as_str(),
    ];
    if let Some(issue) = &segments.issue {
        parts.push(issue);
    }
    parts.join(SEPARATOR)
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

pub fn status_bar_line(
    snapshot: &StatusBarSnapshot,
    width: u16,
    theme: &ThemePalette,
) -> Line<'static> {
    let segments = fit_segments(snapshot, width);
    let mut base = theme.get(SemanticRole::Selection);
    if base.fg.is_none() {
        if let Some(fg) = theme.get(SemanticRole::Fg).fg {
            base = base.fg(fg);
        }
    }

    let muted = theme.get(SemanticRole::Muted);
    let accent = theme.get(SemanticRole::Accent);
    let normal = base;
    let highlight_git = if snapshot.git_modified {
        accent
    } else {
        normal
    };
    let agent_style = if snapshot.agent_status == AgentStatus::Idle {
        if snapshot.agent_running {
            accent
        } else {
            normal
        }
    } else {
        let role_style = theme.get(snapshot.agent_status.semantic_role());
        if role_style.fg.is_some() {
            role_style
        } else {
            accent
        }
    };

    let mut spans = Vec::new();
    let styled_segments: [(String, _); 5] = [
        (BRAND.to_string(), normal),
        (segments.repo, normal),
        (segments.branch, normal),
        (segments.agent, agent_style),
        (segments.git, highlight_git),
    ];

    for (index, (label, style)) in styled_segments.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(SEPARATOR, muted));
        }
        spans.push(Span::styled(label.clone(), *style));
    }

    if let Some(issue) = segments.issue {
        spans.push(Span::styled(SEPARATOR, muted));
        spans.push(Span::styled(issue, accent));
    }

    Line::from(spans)
}

pub fn render_status_bar(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let snapshot = compute_status_bar(state);
    let line = status_bar_line(&snapshot, area.width, &state.theme);
    frame.render_widget(Paragraph::new(line), area);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::agent::AgentStatus;
    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

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
            compute_layout(120, 40, 30).expect("layout"),
        )
    }

    #[test]
    fn compute_status_bar_uses_domain_defaults() {
        let snapshot = compute_status_bar(&test_state());
        assert_eq!(snapshot.repo_name, "cityartwalks");
        assert_eq!(snapshot.branch, "no git");
        assert_eq!(snapshot.agent_label, "Agent Idle");
        assert_eq!(snapshot.git_label, "Clean");
        assert_eq!(snapshot.issue_label, None);
    }

    #[test]
    fn compute_status_bar_reflects_git_agent_and_issue_state() {
        let mut state = test_state();
        state.git.branch = Some("feature/42".to_string());
        state.git.modified_files = vec!["a.rs".to_string(), "b.rs".to_string(), "c.rs".to_string()];
        state.agent.running = true;
        state.agent.status = AgentStatus::Executing;
        state.github.selected_issue = Some(42);

        let snapshot = compute_status_bar(&state);
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
    fn format_status_line_matches_spec_example_at_120_cols() {
        let snapshot = StatusBarSnapshot {
            repo_name: "cityartwalks".to_string(),
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
            "Kiwi | cityartwalks | feature/42 | Agent Executing | 3 Modified | #42"
        );
    }

    #[test]
    fn format_status_line_respects_display_width() {
        let snapshot = StatusBarSnapshot {
            repo_name: "cityartwalks".to_string(),
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
    fn render_status_bar_fits_terminal_width() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut state = test_state();
        state.status_bar.repo_name = "cityartwalks".to_string();
        state.git.branch = Some("feature/very-long-branch-name".to_string());
        state.git.modified_files = vec!["a.rs".to_string(), "b.rs".to_string()];
        state.agent.running = true;
        state.agent.status = AgentStatus::Executing;
        state.github.selected_issue = Some(99);

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_status_bar(frame, frame.area(), &state))
            .expect("draw");
    }

    #[test]
    fn format_status_line_never_returns_empty_for_nonzero_width() {
        let snapshot = StatusBarSnapshot {
            repo_name: "repo".to_string(),
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
