use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::git::branch_selected_name;
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::scrollbar::{render_vertical_scrollbar, split_for_scrollbar};

const STATUS_ROWS: u16 = 1;

pub fn render_branches_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
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
        .title(branch_detail_title(state))
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
        .saturating_add(inner.height.saturating_sub(STATUS_ROWS));
    let status_area = Rect {
        x: inner.x,
        y: status_y,
        width: inner.width,
        height: STATUS_ROWS.min(inner.height),
    };

    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(status_area.height),
    };

    if content_area.height > 0 && content_area.width > 0 {
        render_branch_detail_content(frame, content_area, focused, theme, state);
    }

    if status_area.height > 0 {
        render_status_line(frame, status_area, state, theme);
    }
}

fn branch_detail_title(state: &AppState) -> String {
    let mut title = String::from("Branches");
    if state.branches.detail_loading {
        title.push_str(" · loading");
    } else if state.branches.detail_error.is_some() {
        title.push_str(" · error");
    } else if let Some(name) = branch_selected_name(&state.branches) {
        title.push_str(&format!(" · {name}"));
    }
    title
}

fn render_branch_detail_content(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if !state.workspace_meta.is_git_repo {
        render_detail_message(
            frame,
            area,
            theme,
            &["Not a git repository"],
        );
        return;
    }

    if branch_selected_name(&state.branches).is_none() {
        render_detail_message(
            frame,
            area,
            theme,
            &["Select a branch in the GH panel and press Enter or double-click"],
        );
        return;
    }

    if state.branches.detail_loading && state.branches.detail.is_none() {
        render_detail_message(frame, area, theme, &["Loading branch details…"]);
        return;
    }

    if let Some(error) = &state.branches.detail_error {
        render_detail_message(frame, area, theme, &[error.as_str()]);
        return;
    }

    let Some(detail) = &state.branches.detail else {
        render_detail_message(
            frame,
            area,
            theme,
            &["Double-click a branch in the GH panel to view details"],
        );
        return;
    };

    let lines = detail.display_lines(state.git.ahead, state.git.behind);
    let (content, scrollbar) = split_for_scrollbar(area);
    let viewport_rows = content.height as usize;
    let max_width = content.width as usize;
    let start = state.branches.detail_scroll_offset;
    let mut rendered = Vec::new();

    for viewport_index in 0..viewport_rows {
        let line_index = start + viewport_index;
        let Some(line) = lines.get(line_index) else {
            break;
        };
        rendered.push(Line::from(Span::styled(
            truncate_line(line, max_width),
            theme.get(SemanticRole::Fg),
        )));
    }

    frame.render_widget(Paragraph::new(rendered), content);
    if let Some(scrollbar_area) = scrollbar {
        render_vertical_scrollbar(
            frame,
            scrollbar_area,
            start,
            lines.len(),
            viewport_rows,
            focused,
            theme,
        );
    }
}

fn render_detail_message(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &ThemePalette,
    lines: &[&str],
) {
    let max_width = area.width as usize;
    let rendered = lines
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                truncate_line(line, max_width),
                theme.get(SemanticRole::Muted),
            ))
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(rendered), area);
}

fn render_status_line(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &ThemePalette) {
    let status = if !state.workspace_meta.is_git_repo {
        "Git features disabled"
    } else if branch_selected_name(&state.branches).is_none() {
        "Select a branch in GH · Enter/double-click to open"
    } else if state.branches.detail_loading {
        "Loading branch details…"
    } else if state.branches.checkout_loading {
        "Checking out…"
    } else if let Some(error) = &state.branches.checkout_error {
        error.as_str()
    } else if state.branches.detail.is_some() {
        "j/k scroll · Enter checkout · R refresh"
    } else {
        "Double-click branch in GH · R refresh"
    };

    let style = if state.branches.checkout_error.is_some() {
        theme.get(SemanticRole::AgentError)
    } else {
        theme.get(SemanticRole::Muted)
    };

    frame.render_widget(
        Paragraph::new(truncate_line(status, area.width as usize)).style(style),
        area,
    );
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

    text.chars().take(max_width).collect()
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
    use std::path::PathBuf;

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::config::ResolvedConfig;
    use crate::git::{BranchDetail, BranchEntry};
    use crate::layout::compute_layout;
    use crate::state::{AppState, BranchState};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        let mut state = AppState::from_startup(
            PathBuf::from("."),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        );
        state.branches = BranchState {
            entries: vec![
                BranchEntry {
                    name: "main".to_string(),
                    is_current: true,
                },
                BranchEntry {
                    name: "dev".to_string(),
                    is_current: false,
                },
            ],
            selected_index: Some(0),
            detail: Some(BranchDetail {
                name: "main".to_string(),
                is_current: true,
                tip_sha: "abc1234".to_string(),
                tip_subject: "Initial commit".to_string(),
                tip_author: "Kiwi".to_string(),
                tip_date: "2026-01-01".to_string(),
            }),
            detail_for_branch: Some("main".to_string()),
            ..BranchState::default()
        };
        state
    }

    #[test]
    fn render_branches_pane_shows_branch_detail() {
        let state = test_state();
        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| {
                render_branches_pane(frame, Rect::new(0, 0, 80, 12), true, &state.theme, &state);
            })
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let text = buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("main"));
        assert!(text.contains("Initial commit"));
    }
}
