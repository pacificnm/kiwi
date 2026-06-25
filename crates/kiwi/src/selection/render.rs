use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::state::{SelectionPane, TextSelection};

pub fn line_spans_with_selection(
    text: &str,
    abs_line: usize,
    pane: SelectionPane,
    selection: &TextSelection,
    base_style: Style,
    theme: &ThemePalette,
) -> Line<'static> {
    if !selection.applies_to(pane) {
        return Line::from(Span::styled(text.to_string(), base_style));
    }

    let (start, end) = selection.normalized();
    if abs_line < start.line || abs_line > end.line {
        return Line::from(Span::styled(text.to_string(), base_style));
    }

    let selected_style = theme.get(SemanticRole::Selection);
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let col_start = if abs_line == start.line {
        start.col.min(len)
    } else {
        0
    };
    let col_end = if abs_line == end.line {
        end.col.min(len)
    } else {
        len
    };

    if col_start >= col_end {
        return Line::from(Span::styled(text.to_string(), base_style));
    }

    let before: String = chars[..col_start].iter().collect();
    let middle: String = chars[col_start..col_end].iter().collect();
    let after: String = chars[col_end..].iter().collect();

    let mut spans = Vec::new();
    if !before.is_empty() {
        spans.push(Span::styled(before, base_style));
    }
    if !middle.is_empty() {
        spans.push(Span::styled(middle, selected_style));
    }
    if !after.is_empty() {
        spans.push(Span::styled(after, base_style));
    }

    Line::from(spans)
}
