use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ansi::{ansi_line, ansi_line_with_cursor, pty_base_style, strip_ansi};
use crate::selection::{line_spans_with_selection, SelectionPane};
use crate::shell::ScrollbackBuffer;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::scrollbar::{render_vertical_scrollbar, split_for_scrollbar};

pub struct ScrollbackPane<'a> {
    pub scrollback: &'a ScrollbackBuffer,
    pub follow_tail: bool,
    pub viewport_offset: usize,
    pub spawn_error: Option<&'a str>,
    pub idle_hint: Option<&'a str>,
    pub footer: Option<&'a str>,
    pub selection_pane: Option<SelectionPane>,
    pub show_pty_cursor: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn render_scrollback_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    focused: bool,
    theme: &ThemePalette,
    hint_style: Style,
    pane: ScrollbackPane<'_>,
    selection: &crate::selection::TextSelection,
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
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let (content, scrollbar) = split_for_scrollbar(inner);
    fill_pty_background(frame, content);

    let footer_rows = usize::from(pane.footer.is_some());
    let visible_height = content.height.saturating_sub(footer_rows as u16) as usize;
    let max_width = content.width as usize;
    let include_pending = pane.follow_tail;
    let line_count = pane.scrollback.line_count();
    let start =
        pane.scrollback
            .viewport_start(visible_height, pane.follow_tail, pane.viewport_offset);
    let lines = pane
        .scrollback
        .viewport_lines(start, visible_height, max_width, include_pending);
    let cursor_col_on_row = pane
        .show_pty_cursor
        .then(|| {
            pane.scrollback
                .cursor_display_position(include_pending)
                .and_then(|(line_index, col)| {
                    let row = line_index.checked_sub(start)?;
                    if row < visible_height {
                        Some((row, col))
                    } else {
                        None
                    }
                })
        })
        .flatten();

    if lines.is_empty() {
        if let Some(footer) = pane.footer {
            render_hint_line(frame, content, 0, footer, hint_style);
            return;
        }
        if let Some(error) = pane.spawn_error {
            render_hint_line(frame, content, 0, error, hint_style);
            return;
        }
        if let Some(hint) = pane.idle_hint {
            render_hint_line(frame, content, 0, hint, hint_style);
        }
        return;
    }

    for (row, line) in lines.iter().enumerate().take(visible_height) {
        let cursor_col =
            cursor_col_on_row.and_then(
                |(cursor_row, col)| {
                    if cursor_row == row {
                        Some(col)
                    } else {
                        None
                    }
                },
            );
        render_pty_line(
            frame,
            content,
            row,
            line,
            max_width,
            pane.selection_pane,
            selection,
            theme,
            cursor_col,
        );
    }

    if let Some(footer) = pane.footer {
        render_hint_line(
            frame,
            content,
            content.height.saturating_sub(1) as usize,
            footer,
            hint_style,
        );
    }

    if let Some(scrollbar_area) = scrollbar {
        let scroll_track = Rect {
            height: visible_height as u16,
            ..scrollbar_area
        };
        render_vertical_scrollbar(
            frame,
            scroll_track,
            start,
            line_count,
            visible_height,
            focused,
            theme,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_pty_line(
    frame: &mut Frame<'_>,
    inner: Rect,
    row: usize,
    text: &str,
    max_width: usize,
    selection_pane: Option<SelectionPane>,
    selection: &crate::selection::TextSelection,
    theme: &ThemePalette,
    cursor_col: Option<usize>,
) {
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

    if let Some(pane) = selection_pane {
        if selection.applies_to(pane) {
            let plain = strip_ansi(text);
            let truncated = if plain.chars().count() > max_width {
                plain.chars().take(max_width).collect::<String>()
            } else {
                plain
            };
            let line = line_spans_with_selection(
                &truncated,
                row,
                pane,
                selection,
                pty_base_style(),
                theme,
            );
            frame.render_widget(Paragraph::new(line), row_area);
            return;
        }
    }

    let line = if cursor_col.is_some() {
        ansi_line_with_cursor(text, max_width, cursor_col)
    } else {
        ansi_line(text, max_width)
    };
    frame.render_widget(Paragraph::new(line).style(pty_base_style()), row_area);
}

fn fill_pty_background(frame: &mut Frame<'_>, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let style = pty_base_style();
    let blank = " ".repeat(area.width as usize);
    for row in 0..area.height {
        let row_area = Rect {
            x: area.x,
            y: area.y.saturating_add(row),
            width: area.width,
            height: 1,
        };
        frame.render_widget(Clear, row_area);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(blank.clone(), style))),
            row_area,
        );
    }
}

fn render_hint_line(frame: &mut Frame<'_>, inner: Rect, row: usize, text: &str, style: Style) {
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
