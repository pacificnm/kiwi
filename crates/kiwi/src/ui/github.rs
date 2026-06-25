use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::github::{
    issue_at_viewport, issue_selected_row_index, GitHubAuthErrorKind, Issue, IssueState,
};
use crate::navigation::MainTab;
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

const ISSUES_STATUS_ROWS: u16 = 1;
const ISSUE_DETAIL_STATUS_ROWS: u16 = 1;

pub fn issues_viewport_rows(area: Rect) -> usize {
    pane_inner(area)
        .map(|inner| inner.height.saturating_sub(ISSUES_STATUS_ROWS) as usize)
        .unwrap_or(0)
}

pub fn issue_detail_viewport_rows(area: Rect) -> usize {
    pane_inner(area)
        .map(|inner| inner.height.saturating_sub(ISSUE_DETAIL_STATUS_ROWS) as usize)
        .unwrap_or(0)
}

pub fn render_github_issues_list_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if state.github.loading && !state.github.auth_checked {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "Issues",
            vec![Line::from("Checking GitHub authentication…")],
        );
        return;
    }

    if let Some(message) = github_auth_message(state) {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "Issues",
            auth_error_lines(message, state.github.error_kind),
        );
        return;
    }

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(issues_list_title(state))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let Some(inner) = pane_inner(area) else {
        return;
    };

    let status_y = inner
        .y
        .saturating_add(inner.height.saturating_sub(ISSUES_STATUS_ROWS));
    let status_area = Rect {
        x: inner.x,
        y: status_y,
        width: inner.width,
        height: ISSUES_STATUS_ROWS.min(inner.height),
    };
    let list_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(status_area.height),
    };

    if list_area.height > 0 && list_area.width > 0 {
        render_issue_list(frame, list_area, focused, theme, state);
    }

    if status_area.height > 0 {
        render_issues_list_status_line(frame, status_area, state, theme);
    }
}

pub fn render_issue_detail_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if state.github.loading && !state.github.auth_checked {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "Issues",
            vec![Line::from("Checking GitHub authentication…")],
        );
        return;
    }

    if let Some(message) = github_auth_message(state) {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "Issues",
            auth_error_lines(message, state.github.error_kind),
        );
        return;
    }

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(issue_detail_title(state))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let Some(inner) = pane_inner(area) else {
        return;
    };

    let status_y = inner
        .y
        .saturating_add(inner.height.saturating_sub(ISSUE_DETAIL_STATUS_ROWS));
    let status_area = Rect {
        x: inner.x,
        y: status_y,
        width: inner.width,
        height: ISSUE_DETAIL_STATUS_ROWS.min(inner.height),
    };
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(status_area.height),
    };

    if content_area.height > 0 && content_area.width > 0 {
        render_issue_detail_content(frame, content_area, theme, state);
    }

    if status_area.height > 0 {
        render_issue_detail_status_line(frame, status_area, focused, state, theme);
    }
}

pub fn render_github_main_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
    main_tab: MainTab,
) {
    let title = main_tab.label();
    let lines = if state.github.loading && !state.github.auth_checked {
        vec![Line::from("Checking GitHub authentication…")]
    } else if let Some(message) = github_auth_message(state) {
        auth_error_lines(message, state.github.error_kind)
    } else {
        vec![Line::from(format!(
            "{title} view — press R to refresh (PR list in a later milestone)"
        ))]
    };

    render_github_pane(frame, area, focused, theme, state, title, lines);
}

fn issues_list_title(state: &AppState) -> String {
    let mut title = String::from("Issues");
    if state.github.issues_loading {
        title.push_str(" · loading");
    } else if state.github.issues_error.is_some() {
        title.push_str(" · error");
    } else if !state.github.issues.is_empty() {
        title.push_str(&format!(" · {}", state.github.issues.len()));
    }
    title
}

fn issue_detail_title(state: &AppState) -> String {
    let mut title = String::from("Issues");
    if state.github.issue_detail_loading {
        title.push_str(" · loading");
    } else if state.github.issue_detail_error.is_some() {
        title.push_str(" · error");
    } else if let Some(number) = state.github.selected_issue {
        title.push_str(&format!(" · #{number}"));
    }
    title
}

fn render_issue_detail_content(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &ThemePalette,
    state: &AppState,
) {
    let viewport_rows = area.height as usize;
    let max_width = area.width as usize;

    if state.github.selected_issue.is_none() {
        render_issue_detail_message(
            frame,
            area,
            theme,
            &[
                "No issue selected",
                "",
                "Select an issue in the GH left panel (Alt+4) and press Enter.",
            ],
        );
        return;
    }

    if state.github.issue_detail_loading && state.github.issue_detail.is_none() {
        render_issue_detail_message(frame, area, theme, &["Loading issue detail…"]);
        return;
    }

    if let Some(error) = &state.github.issue_detail_error {
        render_issue_detail_message(frame, area, theme, &[error.as_str()]);
        return;
    }

    let Some(detail) = &state.github.issue_detail else {
        render_issue_detail_message(
            frame,
            area,
            theme,
            &[
                "Issue detail not loaded",
                "",
                "Press Enter on an issue in the GH left panel.",
            ],
        );
        return;
    };

    let start = state.github.issue_detail_scroll_offset;
    let end = (start + viewport_rows).min(detail.display_lines.len());
    let fg = theme.get(SemanticRole::Fg);

    for (row, line_index) in (start..end).enumerate() {
        let line_text = &detail.display_lines[line_index];
        let display = truncate_line(line_text, max_width);
        let style = if line_index == 0 {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            fg
        };

        let row_area = Rect {
            x: area.x,
            y: area.y.saturating_add(row as u16),
            width: area.width,
            height: 1,
        };
        frame.render_widget(Clear, row_area);
        frame.render_widget(Paragraph::new(display).style(style), row_area);
    }
}

fn render_issue_detail_message(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &ThemePalette,
    lines: &[&str],
) {
    frame.render_widget(
        Paragraph::new(
            lines
                .iter()
                .map(|line| Line::from((*line).to_string()))
                .collect::<Vec<_>>(),
        )
        .style(theme.get(SemanticRole::Fg))
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_issue_detail_status_line(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    state: &AppState,
    theme: &ThemePalette,
) {
    let status = if state.github.issue_detail_loading {
        "Loading detail…"
    } else if state.github.selected_issue.is_none() {
        "Enter on GH left list to view"
    } else if state.github.issue_detail.is_some() && focused {
        "j/k scroll · PgUp/PgDn page · R refresh"
    } else if state.github.issue_detail.is_some() {
        "j/k scroll · R refresh"
    } else {
        "Enter on GH left list · R refresh"
    };

    frame.render_widget(
        Paragraph::new(truncate_line(status, area.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn render_issue_list(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let viewport_rows = area.height as usize;
    let max_width = area.width as usize;
    let selected_row = issue_selected_row_index(&state.github);
    let mut lines = Vec::new();

    for viewport_index in 0..viewport_rows {
        let Some(issue) = issue_at_viewport(&state.github, viewport_index) else {
            break;
        };
        let row_index = state.github.issues_scroll_offset + viewport_index;
        lines.push(render_issue_line(
            issue,
            row_index,
            selected_row,
            max_width,
            focused,
            theme,
        ));
    }

    if lines.is_empty() {
        let hint = if state.github.issues_loading {
            "Loading issues…"
        } else if let Some(error) = &state.github.issues_error {
            error.as_str()
        } else {
            "No open issues · R refresh"
        };
        lines.push(Line::from(Span::styled(
            truncate_line(hint, max_width),
            theme.get(SemanticRole::Muted),
        )));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_issue_line(
    issue: &Issue,
    row_index: usize,
    selected_row: Option<usize>,
    max_width: usize,
    focused: bool,
    theme: &ThemePalette,
) -> Line<'static> {
    let selected = focused && selected_row == Some(row_index);
    let state_style = match issue.state {
        IssueState::Open => theme.get(SemanticRole::IssueOpen),
        IssueState::Closed => theme.get(SemanticRole::IssueClosed),
    };

    let mut spans = vec![
        Span::styled(format!("#{} ", issue.number), state_style),
        Span::styled(
            truncate_line(&issue.title, max_width.saturating_sub(8)),
            if selected {
                theme.get(SemanticRole::Accent)
            } else {
                theme.get(SemanticRole::Fg)
            },
        ),
    ];

    if issue.state == IssueState::Closed {
        spans.push(Span::styled(
            format!(" [{}]", issue.state.label()),
            theme.get(SemanticRole::IssueClosed),
        ));
    }

    let mut line = Line::from(spans);
    if selected {
        line = line.style(Style::default().add_modifier(Modifier::BOLD));
    }
    line
}

fn render_issues_list_status_line(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &ThemePalette,
) {
    let status = if state.github.issues_loading {
        "Loading issues…"
    } else if state.github.issues_error.is_some() {
        "j/k move · R refresh"
    } else if state.github.issues.is_empty() {
        "R refresh"
    } else {
        "j/k move · Enter view · R refresh"
    };

    frame.render_widget(
        Paragraph::new(truncate_line(status, area.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn render_github_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
    title: &str,
    lines: Vec<Line<'_>>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(title_with_auth_status(title, state))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.get(SemanticRole::Fg))
            .wrap(Wrap { trim: false }),
        inner,
    );
}

fn title_with_auth_status(title: &str, state: &AppState) -> String {
    if state.github.loading {
        return format!("{title} · checking auth");
    }

    if !state.github.auth_checked {
        return title.to_string();
    }

    if state.github.auth_ok {
        format!("{title} · authenticated")
    } else {
        format!("{title} · auth required")
    }
}

fn github_auth_message(state: &AppState) -> Option<&str> {
    if state.github.auth_ok {
        return None;
    }

    if !state.github.auth_checked && !state.github.loading {
        return None;
    }

    state
        .github
        .error
        .as_deref()
        .filter(|message| !message.is_empty())
}

fn auth_error_lines(message: &str, kind: Option<GitHubAuthErrorKind>) -> Vec<Line<'static>> {
    let heading = match kind {
        Some(GitHubAuthErrorKind::NotInstalled) => "GitHub CLI required",
        Some(GitHubAuthErrorKind::NotAuthenticated) => "GitHub login required",
        Some(GitHubAuthErrorKind::CommandFailed) | None => "GitHub auth check failed",
    };

    let mut lines = vec![Line::from(Span::styled(
        heading,
        Style::default().add_modifier(Modifier::BOLD),
    ))];

    for paragraph in message.split("\n\n") {
        lines.push(Line::from(""));
        for line in paragraph.lines() {
            lines.push(Line::from(line.to_string()));
        }
    }

    lines
}

fn pane_inner(area: Rect) -> Option<Rect> {
    if area.width < 2 || area.height < 2 {
        return None;
    }

    Some(Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    })
}

fn truncate_line(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let char_count = text.chars().count();
    if char_count <= max_width {
        return text.to_string();
    }

    if max_width <= 1 {
        return "…".to_string();
    }

    text.chars()
        .take(max_width.saturating_sub(1))
        .collect::<String>()
        + "…"
}

fn chrome_style(theme: &ThemePalette) -> Style {
    let mut style = Style::default();
    if let Some(bg) = theme.get(SemanticRole::Bg).bg {
        style = style.bg(bg);
    }
    if let Some(fg) = theme.get(SemanticRole::Fg).fg {
        style = style.fg(fg);
    }
    style
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::config::ResolvedConfig;
    use crate::github::{GitHubAuthErrorKind, Issue, IssueDetail, IssueState};
    use crate::layout::compute_layout;
    use crate::navigation::{LeftNavTab, MainTab};
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            std::path::PathBuf::from("."),
            false,
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
    fn issues_list_pane_renders_in_left_gh_panel() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.issues = vec![Issue {
            number: 55,
            title: "Issue list via gh json".to_string(),
            state: IssueState::Open,
            labels: vec!["epic-e14".to_string()],
            assignees: vec!["octocat".to_string()],
        }];
        state.github.selected_issue = Some(55);
        state.github.issues_loaded_at = Some(SystemTime::now());

        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_github_issues_list_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("#55"));
        assert!(content.contains("Issue list via gh json"));
    }

    #[test]
    fn issue_detail_pane_shows_loaded_body_and_comments() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.selected_issue = Some(56);
        state.github.issue_detail_number = Some(56);
        state.github.issue_detail = Some(IssueDetail {
            number: 56,
            title: "Issue detail view".to_string(),
            state: IssueState::Open,
            author: "pacificnm".to_string(),
            labels: vec!["epic-e14".to_string()],
            assignees: Vec::new(),
            display_lines: vec![
                "#56 Issue detail view".to_string(),
                "State: open · Author: pacificnm".to_string(),
                "Labels: epic-e14".to_string(),
                String::new(),
                "— Body —".to_string(),
                String::new(),
                "Detailed body text".to_string(),
                String::new(),
                "— Comments (1) —".to_string(),
                String::new(),
                "@reviewer · 2026-06-24".to_string(),
                "Ship it".to_string(),
            ],
        });

        let backend = TestBackend::new(100, 16);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_issue_detail_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("#56 Issue detail view"));
        assert!(content.contains("Detailed body text"));
        assert!(content.contains("@reviewer"));
        assert!(content.contains("Ship it"));
    }

    #[test]
    fn main_pane_shows_install_instructions_when_gh_missing() {
        let mut state = test_state();
        state.github.auth_checked = true;
        state.github.auth_ok = false;
        state.github.error_kind = Some(GitHubAuthErrorKind::NotInstalled);
        state.github.error = Some("GitHub CLI (gh) not found.".to_string());

        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_issue_detail_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("GitHub CLI required"));
    }
}
