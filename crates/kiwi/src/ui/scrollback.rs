use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::shell::ScrollbackBuffer;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub struct ScrollbackPane<'a> {
    pub scrollback: &'a ScrollbackBuffer,
    pub follow_tail: bool,
    pub viewport_offset: usize,
    pub spawn_error: Option<&'a str>,
    pub idle_hint: Option<&'a str>,
}

pub fn render_scrollback_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    focused: bool,
    theme: &ThemePalette,
    chrome: Style,
    pane: ScrollbackPane<'_>,
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
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome);

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    frame.render_widget(Clear, inner);

    let visible_height = inner.height as usize;
    let max_width = inner.width as usize;
    let include_pending = pane.follow_tail;
    let start =
        pane.scrollback
            .viewport_start(visible_height, pane.follow_tail, pane.viewport_offset);
    let lines = pane
        .scrollback
        .viewport_lines(start, visible_height, max_width, include_pending);

    if lines.is_empty() {
        if let Some(error) = pane.spawn_error {
            render_clipped_line(frame, inner, 0, error, chrome);
            return;
        }
        if let Some(hint) = pane.idle_hint {
            render_clipped_line(frame, inner, 0, hint, chrome);
        }
        return;
    }

    for (row, line) in lines.iter().enumerate().take(visible_height) {
        render_clipped_line(frame, inner, row, line, chrome);
    }
}

fn render_clipped_line(frame: &mut Frame<'_>, inner: Rect, row: usize, text: &str, style: Style) {
    if row >= inner.height as usize {
        return;
    }

    let row_area = Rect {
        x: inner.x,
        y: inner.y.saturating_add(row as u16),
        width: inner.width,
        height: 1,
    };
    frame.render_widget(Clear, row_area);
    frame.render_widget(Paragraph::new(Line::from(text)).style(style), row_area);
}
