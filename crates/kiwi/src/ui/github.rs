use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::github::GitHubAuthErrorKind;
use crate::navigation::MainTab;
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn render_github_hub_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    render_github_pane(
        frame,
        area,
        focused,
        theme,
        state,
        "GH",
        github_hub_lines(state),
    );
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
            "{title} view — press R to refresh (issue list in a later milestone)"
        ))]
    };

    render_github_pane(frame, area, focused, theme, state, title, lines);
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

fn github_hub_lines(state: &AppState) -> Vec<Line<'static>> {
    if state.github.loading && !state.github.auth_checked {
        return vec![Line::from("Checking GitHub authentication…")];
    }

    if let Some(message) = github_auth_message(state) {
        return auth_error_lines(message, state.github.error_kind);
    }

    vec![
        Line::from(Span::styled(
            "GitHub hub",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Jump to main tabs:"),
        Line::from("  Issues — Alt+2 then 2"),
        Line::from("  PRs    — Alt+2 then 3"),
        Line::from(""),
        Line::from("Press R to refresh auth status."),
    ]
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
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::config::ResolvedConfig;
    use crate::github::GitHubAuthErrorKind;
    use crate::layout::compute_layout;
    use crate::navigation::MainTab;
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
                render_github_main_pane(
                    frame,
                    frame.area(),
                    true,
                    &state.theme,
                    &state,
                    MainTab::Issues,
                );
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
        assert!(content.contains("auth required"));
    }
}
