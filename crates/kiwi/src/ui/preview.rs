use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::selection::{line_spans_with_selection, SelectionPane};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn render_preview_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let title = state
        .preview
        .path
        .as_ref()
        .and_then(|path| path.file_name())
        .map(|name| format!("Preview: {}", name.to_string_lossy()))
        .unwrap_or_else(|| "Preview".to_string());

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let footer_row = inner.height.saturating_sub(1);
    let content_height = footer_row as usize;
    let status = format_preview_status(state);
    render_status_line(frame, inner, footer_row, &status, theme);

    if content_height == 0 {
        return;
    }

    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: footer_row,
    };

    if state.preview.loading && state.preview.lines.is_empty() {
        render_message(
            frame,
            content_area,
            "Loading…",
            theme.get(SemanticRole::Muted),
        );
        return;
    }

    if let Some(error) = state.preview.load_error.as_deref() {
        render_message(
            frame,
            content_area,
            error,
            theme.get(SemanticRole::AgentError),
        );
        return;
    }

    if state.preview.binary {
        let message = format!("Binary file ({} bytes)", state.preview.file_size);
        render_message(
            frame,
            content_area,
            &message,
            theme.get(SemanticRole::Muted),
        );
        return;
    }

    if state.preview.oversize {
        let message = format!(
            "File too large to preview ({} bytes)",
            state.preview.file_size
        );
        render_message(
            frame,
            content_area,
            &message,
            theme.get(SemanticRole::Muted),
        );
        return;
    }

    if state.preview.lines.is_empty() {
        render_message(
            frame,
            content_area,
            "Empty file",
            theme.get(SemanticRole::Muted),
        );
        return;
    }

    render_virtualized_lines(frame, content_area, state, theme, content_height);
}

fn render_virtualized_lines(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &ThemePalette,
    visible_rows: usize,
) {
    let line_numbers = state.config.preview.line_numbers;
    let gutter_width = if line_numbers {
        line_number_width(state.preview.line_count()).max(4)
    } else {
        0
    };

    let text_width = area.width.saturating_sub(gutter_width as u16) as usize;
    if text_width == 0 {
        return;
    }

    let start = state.preview.scroll_offset;
    let end = (start + visible_rows).min(state.preview.lines.len());
    let fg = theme.get(SemanticRole::Fg);
    let gutter = theme.get(SemanticRole::Muted);

    for (row, line_index) in (start..end).enumerate() {
        let line_text = &state.preview.lines[line_index];
        let display = if state.config.preview.wrap {
            line_text.clone()
        } else {
            truncate_line(line_text, text_width)
        };

        let mut spans = Vec::new();
        if line_numbers {
            let number = format!("{:>width$} ", line_index + 1, width = gutter_width - 1);
            spans.push(Span::styled(number, gutter));
        }
        let text_line = line_spans_with_selection(
            &display,
            line_index,
            SelectionPane::Preview,
            &state.text_selection,
            fg,
            theme,
        );
        spans.extend(text_line.spans);

        let row_area = Rect {
            x: area.x,
            y: area.y.saturating_add(row as u16),
            width: area.width,
            height: 1,
        };
        frame.render_widget(Clear, row_area);
        frame.render_widget(Paragraph::new(Line::from(spans)), row_area);
    }
}

fn render_status_line(
    frame: &mut Frame<'_>,
    inner: Rect,
    row: u16,
    status: &str,
    theme: &ThemePalette,
) {
    let area = Rect {
        x: inner.x,
        y: inner.y.saturating_add(row),
        width: inner.width,
        height: 1,
    };
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(truncate_line(status, inner.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn render_message(frame: &mut Frame<'_>, area: Rect, message: &str, style: Style) {
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(truncate_line(message, area.width as usize)).style(style),
        area,
    );
}

fn format_preview_status(state: &AppState) -> String {
    let preview = &state.preview;
    if preview.loading && preview.lines.is_empty() {
        return "Loading…".to_string();
    }

    if preview.loading {
        let path = preview
            .path_display()
            .unwrap_or_else(|| "No file selected".to_string());
        return format!("{path} | reloading…");
    }

    let path = preview
        .path_display()
        .unwrap_or_else(|| "No file selected".to_string());

    if preview.binary {
        return format!("{path} | binary | {} bytes", preview.file_size);
    }

    if preview.oversize {
        return format!("{path} | too large | {} bytes", preview.file_size);
    }

    if preview.load_error.is_some() {
        return path;
    }

    let encoding = if preview.lossy_utf8 {
        "UTF-8 (lossy)"
    } else {
        "UTF-8"
    };
    let truncated = if preview.truncated {
        " | truncated"
    } else {
        ""
    };
    format!(
        "{path} | {} lines | {encoding}{truncated}",
        preview.line_count()
    )
}

fn line_number_width(line_count: usize) -> usize {
    let digits = line_count.max(1).ilog10() as usize + 1;
    digits + 1
}

fn truncate_line(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_width {
        return text.to_string();
    }

    if max_width <= 1 {
        return "…".to_string();
    }

    chars[..max_width - 1].iter().collect::<String>() + "…"
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
    use crate::layout::compute_layout;
    use crate::preview::PreviewState;
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state_with_lines(line_count: usize, scroll_offset: usize) -> AppState {
        let mut state = AppState::from_startup(
            PathBuf::from("."),
            false,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        );
        state.preview = PreviewState {
            path: Some(PathBuf::from("sample.txt")),
            lines: (0..line_count)
                .map(|index| format!("content line {index}"))
                .collect(),
            scroll_offset,
            ..PreviewState::default()
        };
        state
    }

    #[test]
    fn virtualized_render_shows_viewport_lines_only() {
        let state = test_state_with_lines(1000, 500);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let area = Rect::new(0, 0, 80, 20);

        terminal
            .draw(|frame| {
                render_preview_pane(frame, area, true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(content.contains("content line 500"));
        assert!(content.contains("content line 509"));
        assert!(!content.contains("content line 0"));
        assert!(!content.contains("content line 999"));
    }

    #[test]
    fn status_line_includes_path_and_encoding() {
        let mut state = test_state_with_lines(3, 0);
        state.preview.lossy_utf8 = true;
        let status = format_preview_status(&state);
        assert!(status.contains("sample.txt"));
        assert!(status.contains("3 lines"));
        assert!(status.contains("UTF-8 (lossy)"));
    }
}
