use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::agent::AgentStatus;
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub use kiwi_core::status_bar::{
    compute_status_bar_fields, fit_status_bar_segments, StatusBarSnapshot, BRAND, SEPARATOR,
};

pub fn compute_status_bar(state: &AppState) -> StatusBarSnapshot {
    compute_status_bar_fields(
        &state.config,
        &state.git,
        &state.github,
        &state.status_bar,
        &state.agent_manager,
    )
}

pub fn status_bar_line(
    snapshot: &StatusBarSnapshot,
    width: u16,
    theme: &ThemePalette,
) -> Line<'static> {
    let segments = fit_status_bar_segments(snapshot, width);
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

    let mut styled_segments = vec![(BRAND.to_string(), normal)];
    if let Some(repo) = segments.repo {
        styled_segments.push((repo, normal));
    }
    styled_segments.extend([
        (segments.root, normal),
        (segments.branch, normal),
        (segments.agent, agent_style),
        (segments.git, highlight_git),
    ]);

    let mut spans = Vec::new();

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
    use crate::git::{GitFileEntry, GitFileStatus};
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
    fn render_status_bar_fits_terminal_width() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut state = test_state();
        state.status_bar.root_name = "cityartwalks".to_string();
        state.git.remote_repo = Some("org/cityartwalks".to_string());
        state.git.branch = Some("feature/very-long-branch-name".to_string());
        state.git.file_entries = vec![
            GitFileEntry {
                path: "a.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "b.rs".to_string(),
                status: GitFileStatus::Modified,
            },
        ];
        state.active_agent_mut().running = true;
        state.active_agent_mut().status = AgentStatus::Executing;
        state.github.selected_issue = Some(99);

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_status_bar(frame, frame.area(), &state))
            .expect("draw");
    }
}
