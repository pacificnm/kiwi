use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::state::AppState;
use crate::theme::loader::BUILTIN_THEME_NAMES;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::scrollbar::{render_vertical_scrollbar, split_for_scrollbar};

const STATUS_ROWS: u16 = 1;
const HEADER_ROWS: u16 = 1;

pub fn settings_viewport_rows(area: Rect) -> usize {
    settings_list_area(area)
        .map(|inner| inner.height as usize)
        .unwrap_or(0)
}

pub fn settings_interaction_at(
    state: &AppState,
    area: Rect,
    column: u16,
    row: u16,
) -> Option<usize> {
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

    let list_area = settings_list_area(area)?;

    if column < list_area.x
        || column >= list_area.x.saturating_add(list_area.width)
        || row < list_area.y
        || row >= list_area.y.saturating_add(list_area.height)
    {
        return None;
    }

    let viewport_index = usize::from(row.saturating_sub(list_area.y));
    let row_index = state.settings.scroll_offset.saturating_add(viewport_index);
    if row_index >= BUILTIN_THEME_NAMES.len() {
        return None;
    }

    Some(row_index)
}

pub fn render_settings_pane(
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
        .title("Settings")
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let Some(inner) = pane_inner(area) else {
        return;
    };

    if let Some(header_area) = header_area(inner) {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Theme",
                theme.get(SemanticRole::Accent).add_modifier(Modifier::BOLD),
            ))),
            header_area,
        );
    }

    let Some(list_area) = settings_list_area(inner) else {
        return;
    };

    if list_area.height == 0 {
        return;
    }

    let active_theme = state.config.theme.name.as_str();
    let viewport_rows = list_area.height as usize;
    let count = BUILTIN_THEME_NAMES.len();
    let start = state.settings.scroll_offset.min(count);
    let end = (start + viewport_rows).min(count);
    if start > end {
        return;
    }

    let lines: Vec<Line<'_>> = BUILTIN_THEME_NAMES[start..end]
        .iter()
        .enumerate()
        .map(|(offset, name)| {
            let row_index = start + offset;
            let selected = row_index == state.settings.selected_index;
            let active = *name == active_theme;

            let mut label = String::from("  ");
            if selected && focused {
                label.push('▸');
                label.push(' ');
            } else {
                label.push_str("  ");
            }
            label.push_str(name);
            if active {
                label.push_str(" (active)");
            }

            let style = if selected && focused {
                theme.get(SemanticRole::Selection)
            } else if active {
                theme.get(SemanticRole::Accent)
            } else {
                chrome_style(theme)
            };

            Line::from(Span::styled(label, style))
        })
        .collect();

    let (content_area, scrollbar_area) = split_for_scrollbar(list_area);
    frame.render_widget(
        Paragraph::new(lines).style(chrome_style(theme)),
        content_area,
    );
    if let Some(scrollbar) = scrollbar_area {
        render_vertical_scrollbar(
            frame,
            scrollbar,
            state.settings.scroll_offset,
            BUILTIN_THEME_NAMES.len(),
            viewport_rows,
            focused,
            theme,
        );
    }

    if let Some(status_area) = status_area(inner) {
        let hint = "j/k move · click or Enter apply theme · ~/.config/kiwi/config.toml";
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                hint,
                theme.get(SemanticRole::Muted),
            ))),
            status_area,
        );
    }
}

pub(crate) fn settings_list_area(inner: Rect) -> Option<Rect> {
    if inner.height <= STATUS_ROWS + HEADER_ROWS {
        return None;
    }

    Some(Rect {
        x: inner.x,
        y: inner.y + HEADER_ROWS,
        width: inner.width,
        height: inner.height.saturating_sub(STATUS_ROWS + HEADER_ROWS),
    })
}

fn pane_inner(area: Rect) -> Option<Rect> {
    if area.width < 2 || area.height < 2 {
        return None;
    }

    Some(Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    })
}

fn header_area(inner: Rect) -> Option<Rect> {
    if inner.height <= STATUS_ROWS + HEADER_ROWS {
        return None;
    }

    Some(Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: HEADER_ROWS,
    })
}

fn status_area(inner: Rect) -> Option<Rect> {
    if inner.height <= STATUS_ROWS {
        return None;
    }

    Some(Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(STATUS_ROWS),
        width: inner.width,
        height: STATUS_ROWS,
    })
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
    use crate::layout::compute_layout;
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
    fn render_settings_pane_shows_theme_section() {
        let state = test_state();
        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_settings_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(content.contains("Theme"));
        assert!(content.contains("kiwi-dark"));
    }

    #[test]
    fn render_settings_pane_tolerates_stale_scroll_offset() {
        let mut state = test_state();
        state.settings.scroll_offset = 100;
        state.settings.selected_index = 7;

        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_settings_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");
    }
}
