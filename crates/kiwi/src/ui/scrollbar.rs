//! Vertical scrollbar gutter for scrollable panes (issue #148).

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

/// Reserve one column on the trailing edge for a scrollbar gutter.
#[must_use]
pub fn split_for_scrollbar(area: Rect) -> (Rect, Option<Rect>) {
    if area.width <= 1 || area.height == 0 {
        return (area, None);
    }

    let content = Rect {
        x: area.x,
        y: area.y,
        width: area.width.saturating_sub(1),
        height: area.height,
    };
    let scrollbar = Rect {
        x: area.x.saturating_add(content.width),
        y: area.y,
        width: 1,
        height: area.height,
    };
    (content, Some(scrollbar))
}

/// Compute thumb start row and height within the track.
#[must_use]
pub fn vertical_thumb_bounds(
    scroll_offset: usize,
    content_len: usize,
    viewport_len: usize,
    track_len: usize,
) -> Option<(usize, usize)> {
    if track_len == 0 || content_len <= viewport_len {
        return None;
    }

    let max_offset = content_len.saturating_sub(viewport_len);
    let offset = scroll_offset.min(max_offset);
    let thumb_len = (track_len * viewport_len / content_len)
        .max(1)
        .min(track_len);
    let thumb_start = if max_offset == 0 {
        0
    } else {
        offset
            .saturating_mul(track_len.saturating_sub(thumb_len))
            .checked_div(max_offset)
            .unwrap_or(0)
    };
    Some((thumb_start, thumb_len))
}

pub fn render_vertical_scrollbar(
    frame: &mut Frame<'_>,
    area: Rect,
    scroll_offset: usize,
    content_len: usize,
    viewport_len: usize,
    focused: bool,
    theme: &ThemePalette,
) {
    let Some((thumb_start, thumb_len)) = vertical_thumb_bounds(
        scroll_offset,
        content_len,
        viewport_len,
        area.height as usize,
    ) else {
        return;
    };

    let track_style = if focused {
        theme.get(SemanticRole::Border)
    } else {
        theme.get(SemanticRole::Muted)
    };
    let thumb_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Muted)
    };

    let track_len = area.height as usize;
    let mut spans = Vec::with_capacity(track_len);
    for row in 0..track_len {
        let in_thumb = row >= thumb_start && row < thumb_start.saturating_add(thumb_len);
        let ch = if in_thumb { '█' } else { '│' };
        let style = if in_thumb { thumb_style } else { track_style };
        spans.push(Span::styled(ch.to_string(), style));
    }

    for (row, span) in spans.into_iter().enumerate() {
        let row_area = Rect {
            x: area.x,
            y: area.y.saturating_add(row as u16),
            width: 1,
            height: 1,
        };
        frame.render_widget(Paragraph::new(Line::from(span)), row_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumb_at_top_when_offset_zero() {
        let (start, len) = vertical_thumb_bounds(0, 100, 10, 20).expect("thumb");
        assert_eq!(start, 0);
        assert!(len >= 1);
    }

    #[test]
    fn thumb_at_bottom_when_fully_scrolled() {
        let content_len = 100;
        let viewport_len = 10;
        let track_len = 20;
        let max_offset = content_len - viewport_len;
        let (start, len) =
            vertical_thumb_bounds(max_offset, content_len, viewport_len, track_len).expect("thumb");
        assert_eq!(start.saturating_add(len), track_len);
    }

    #[test]
    fn no_thumb_when_content_fits() {
        assert!(vertical_thumb_bounds(0, 5, 10, 20).is_none());
    }

    #[test]
    fn split_for_scrollbar_reserves_trailing_column() {
        let area = Rect::new(0, 0, 10, 5);
        let (content, scrollbar) = split_for_scrollbar(area);
        assert_eq!(content.width, 9);
        assert_eq!(scrollbar.expect("scrollbar").x, 9);
    }
}
