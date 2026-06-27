//! Viewport helpers for dock panels (SPEC-022).
//!
//! Lists virtualize inside egui_dock's tab [`ScrollArea`] — panels must not nest another
//! [`ScrollArea`] or scroll breaks (no bars, stale offsets, blank space at top).

use egui::{Align, Rect, Ui};

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
) -> usize {
    if total_rows == 0 {
        *scroll_row = 0;
        return 1;
    }

    let spacing_y = ui.spacing().item_spacing.y;
    let stride = row_height + spacing_y;
    let total_height = (stride * total_rows as f32 - spacing_y).max(row_height);
    let viewport_rows = row_count_for_height(ui.clip_rect().height(), row_height);
    let max_start = total_rows.saturating_sub(viewport_rows.max(1));
    *scroll_row = (*scroll_row).min(max_start);

    let visible_start = scroll_row_from_clip(ui, row_height);
    if visible_start.abs_diff(*scroll_row) > 1 {
        scroll_to_row(ui, row_height, *scroll_row);
    }

    ui.set_min_height(total_height);

    let start = scroll_row_from_clip(ui, row_height).min(max_start);
    let end = (start + viewport_rows + 2).min(total_rows);

    if start > 0 {
        ui.add_space(start as f32 * stride);
    }
    for row in start..end {
        render_row(ui, row);
    }

    *scroll_row = start;
    viewport_rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_count_fits_at_least_one_row() {
        assert_eq!(row_count_for_height(0.0, 18.0), 1);
        assert_eq!(row_count_for_height(36.0, 18.0), 2);
    }
}
