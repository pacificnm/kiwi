//! Viewport helpers for dock panels (SPEC-022).
//!
//! Lists virtualize inside egui_dock's tab [`ScrollArea`] — panels must not nest another
//! [`ScrollArea`] or scroll breaks (no bars, stale offsets, blank space at top).

use egui::{Align, CursorIcon, Rect, Response, Ui, WidgetText};

use super::ansi::{max_cols_for_ui, monospace_font_id};

const MIN_PTY_COLS: u16 = 2;
const MIN_PTY_ROWS: u16 = 1;

/// Monospace row height for PTY scrollback virtualization.
pub const PTY_ROW_HEIGHT: f32 = 18.0;

/// Clickable list row label (Git / explorer / GH list pattern) with pointer cursor on hover.
#[must_use]
pub fn selectable_label(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    ui.add(
        egui::Label::new(text).sense(egui::Sense::click()),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

/// Like [`selectable_label`] but truncates overflowing text (issue/PR titles).
#[must_use]
pub fn selectable_label_truncate(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    ui.add(
        egui::Label::new(text)
            .truncate()
            .sense(egui::Sense::click()),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

/// Result of [`render_virtual_rows`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualRowsLayout {
    pub viewport_rows: usize,
    pub max_start: usize,
}

/// Row count that fits in a region of the given height.
#[must_use]
pub fn row_count_for_height(height: f32, row_height: f32) -> usize {
    let height = height.max(row_height);
    usize::try_from((height / row_height).floor() as i64)
        .unwrap_or(1)
        .max(1)
}

/// First visible row index from the dock tab scroll position.
fn scroll_row_from_clip(ui: &Ui, row_height: f32) -> usize {
    let stride = row_height + ui.spacing().item_spacing.y;
    let scroll_y = (ui.clip_rect().top() - ui.max_rect().top()).max(0.0);
    (scroll_y / stride).floor().max(0.0) as usize
}

/// Last row index that can be aligned to the top of the viewport (pixel-accurate).
#[must_use]
pub fn max_scroll_row(ui: &Ui, row_height: f32, total_rows: usize) -> usize {
    if total_rows == 0 {
        return 0;
    }

    let spacing_y = ui.spacing().item_spacing.y;
    let stride = row_height + spacing_y;
    let total_height = (stride * total_rows as f32 - spacing_y).max(row_height);
    let clip_height = ui.clip_rect().height().max(row_height);
    if total_height <= clip_height {
        return 0;
    }

    let max_scroll_y = total_height - clip_height;
    ((max_scroll_y / stride).floor() as usize).min(total_rows.saturating_sub(1))
}

/// Scroll so `scroll_row` is at the top of the viewport (keyboard navigation).
fn scroll_to_row(ui: &mut Ui, row_height: f32, scroll_row: usize) {
    let stride = row_height + ui.spacing().item_spacing.y;
    let y = ui.max_rect().top() + scroll_row as f32 * stride;
    let rect = Rect::from_min_max(
        egui::pos2(ui.max_rect().left(), y),
        egui::pos2(ui.max_rect().right(), y + row_height),
    );
    ui.scroll_to_rect(rect, Some(Align::TOP));
}

/// Virtualized rows inside the dock tab scroll area (one scroll layer for the whole tab).
#[must_use]
pub fn render_virtual_rows(
    ui: &mut Ui,
    row_height: f32,
    total_rows: usize,
    scroll_row: &mut usize,
    mut render_row: impl FnMut(&mut Ui, usize),
) -> VirtualRowsLayout {
    if total_rows == 0 {
        *scroll_row = 0;
        return VirtualRowsLayout {
            viewport_rows: 1,
            max_start: 0,
        };
    }

    let spacing_y = ui.spacing().item_spacing.y;
    let stride = row_height + spacing_y;
    let total_height = (stride * total_rows as f32 - spacing_y).max(row_height);
    let viewport_rows = row_count_for_height(ui.clip_rect().height(), row_height);
    let max_start = max_scroll_row(ui, row_height, total_rows);

    let visible_start = scroll_row_from_clip(ui, row_height).min(max_start);
    let requested = (*scroll_row).min(max_start);

    let start = if requested < visible_start.saturating_sub(1) {
        // Keyboard / state moved up — bring the viewport to the stored row.
        scroll_to_row(ui, row_height, requested);
        scroll_row_from_clip(ui, row_height).min(max_start)
    } else if visible_start > requested.saturating_add(1) {
        // Mouse / touch scrolled ahead of stale stored offset — trust the viewport.
        visible_start
    } else if requested > visible_start.saturating_add(1) {
        // Keyboard / state moved down — bring the viewport to the stored row.
        scroll_to_row(ui, row_height, requested);
        scroll_row_from_clip(ui, row_height).min(max_start)
    } else {
        visible_start
    };

    ui.set_min_height(total_height);

    let end = (start + viewport_rows + 2).min(total_rows);

    if start > 0 {
        ui.add_space(start as f32 * stride);
    }
    for row in start..end {
        render_row(ui, row);
    }

    *scroll_row = start;
    VirtualRowsLayout {
        viewport_rows,
        max_start,
    }
}

/// Measure PTY columns and rows from the current panel clip rect.
#[must_use]
pub fn pty_dimensions_from_ui(ui: &Ui, theme_font_size: f32) -> (u16, u16) {
    let font_id = monospace_font_id(ui, theme_font_size);
    let cols = max_cols_for_ui(ui, &font_id).max(usize::from(MIN_PTY_COLS)) as u16;
    let rows = row_count_for_height(ui.clip_rect().height(), PTY_ROW_HEIGHT)
        .max(usize::from(MIN_PTY_ROWS)) as u16;
    (cols, rows)
}

/// Truncate `text` to `max_width` Unicode scalar values, appending `…` if truncated.
pub fn truncate_line(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut iter = text.char_indices();
    if let Some((byte_pos, _)) = iter.nth(max_width - 1) {
        if iter.next().is_some() {
            if max_width <= 1 {
                return "…".to_string();
            }
            return text[..byte_pos].to_string() + "…";
        }
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_count_fits_at_least_one_row() {
        assert_eq!(row_count_for_height(0.0, 18.0), 1);
        assert_eq!(row_count_for_height(36.0, 18.0), 2);
    }

    #[test]
    fn truncate_line_ellipsis_when_too_long() {
        assert_eq!(truncate_line("hello world", 5), "hell…");
    }

    #[test]
    fn truncate_line_exact_width_no_ellipsis() {
        assert_eq!(truncate_line("hello", 5), "hello");
    }

    #[test]
    fn truncate_line_zero_width_empty() {
        assert_eq!(truncate_line("hello", 0), "");
    }

    #[test]
    fn truncate_line_width_one_returns_ellipsis() {
        assert_eq!(truncate_line("hello", 1), "…");
    }
}
