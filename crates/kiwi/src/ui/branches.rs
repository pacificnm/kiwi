use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::git::{branch_row_at_viewport, branch_selected_row_index};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

const STATUS_ROWS: u16 = 1;

pub fn branches_viewport_rows(area: Rect) -> usize {
    pane_inner(area)
        .map(|inner| inner.height.saturating_sub(STATUS_ROWS) as usize)
        .unwrap_or(0)
}

pub fn branch_interaction_at(state: &AppState, area: Rect, column: u16, row: u16) -> Option<usize> {
    if area.width == 0 || area.height == 0 {
        return None;
    }

    if column < area.x
        || column >= area.x.saturating_add(area.width)
        || row < area.y
        || row >= area.y.saturating_add(area.height)
    {
        return None;
    }

    let inner = pane_inner(area)?;

    let list_area = branches_list_area(inner);
    if column < list_area.x
        || column >= list_area.x.saturating_add(list_area.width)
        || row < list_area.y
        || row >= list_area.y.saturating_add(list_area.height)
    {
        return None;
    }

    let viewport_index = usize::from(row.saturating_sub(list_area.y));
    let row_index = state.branches.scroll_offset.saturating_add(viewport_index);
    if row_index >= state.branches.entries.len() {
        return None;
    }

    Some(row_index)
}

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
        .title("Branches")
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

    let list_area = branches_list_area(inner);
    if list_area.height > 0 && list_area.width > 0 {
        render_branch_list(frame, list_area, focused, theme, state);
    }

    if status_area.height > 0 {
        render_status_line(frame, status_area, state, theme);
    }
}

fn render_branch_list(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if !state.workspace_meta.is_git_repo {
        frame.render_widget(
            Paragraph::new("Not a git repository").style(theme.get(SemanticRole::Muted)),
            area,
        );
        return;
    }

    if state.branches.loading && state.branches.entries.is_empty() {
        frame.render_widget(
            Paragraph::new("Loading branches…").style(theme.get(SemanticRole::Muted)),
            area,
        );
        return;
    }

    if let Some(error) = &state.branches.error {
        frame.render_widget(
            Paragraph::new(truncate_line(error, area.width as usize))
                .style(theme.get(SemanticRole::AgentError)),
            area,
        );
        return;
    }

    if state.branches.entries.is_empty() {
        frame.render_widget(
            Paragraph::new("No local branches").style(theme.get(SemanticRole::Muted)),
            area,
        );
        return;
    }

    let viewport_rows = area.height as usize;
    let max_width = area.width as usize;
    let selected_row = branch_selected_row_index(&state.branches);
    let mut lines = Vec::new();

    for viewport_index in 0..viewport_rows {
        let Some(entry) = branch_row_at_viewport(&state.branches, viewport_index) else {
            break;
        };
        let row_index = state.branches.scroll_offset + viewport_index;
        let selected = focused && selected_row == Some(row_index);
        lines.push(render_branch_line(entry, selected, max_width, theme));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_branch_line(
    entry: &crate::git::BranchEntry,
    selected: bool,
    max_width: usize,
    theme: &ThemePalette,
) -> Line<'static> {
    let marker = if entry.is_current { "* " } else { "  " };
    let mut style = if entry.is_current {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Fg)
    };

    if selected {
        style = style.add_modifier(Modifier::REVERSED);
    }

    Line::from(Span::styled(
        truncate_line(&format!("{marker}{}", entry.name), max_width),
        style,
    ))
}

fn render_status_line(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &ThemePalette) {
    let status = if !state.workspace_meta.is_git_repo {
        "Git features disabled"
    } else if state.branches.checkout_loading {
        "Checking out…"
    } else if let Some(error) = &state.branches.checkout_error {
        error.as_str()
    } else if state.branches.loading {
        "Refreshing…"
    } else {
        "j/k move · Enter checkout · double-click checkout · R refresh"
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

fn branches_list_area(inner: Rect) -> Rect {
    Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(STATUS_ROWS),
    }
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
    use crate::git::BranchEntry;
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
            ..BranchState::default()
        };
        state
    }

    #[test]
    fn render_branches_pane_lists_local_branches() {
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
        assert!(text.contains("dev"));
    }
}
