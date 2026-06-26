use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::diff::{DiffLine, DiffLineKind, DiffSource};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::scrollbar::{render_vertical_scrollbar, split_for_scrollbar};

pub fn render_diff_pane(
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
        .diff
        .selected_path
        .as_deref()
        .and_then(|path| std::path::Path::new(path).file_name())
        .map(|name| format!("Diff: {}", name.to_string_lossy()))
        .unwrap_or_else(|| "Diff".to_string());

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
    render_status_line(frame, inner, footer_row, &format_diff_status(state), theme);

    if content_height == 0 {
        return;
    }

    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: footer_row,
    };

    let (scroll_content, scrollbar) = split_for_scrollbar(content_area);
    let scroll_visible_rows = scroll_content.height as usize;

    if state.diff.selected_path.is_none() {
        render_message(
            frame,
            scroll_content,
            "Select a changed file from the Git panel",
            theme.get(SemanticRole::Muted),
        );
        return;
    }

    if state.diff.loading && state.diff.lines.is_empty() {
        render_message(
            frame,
            scroll_content,
            "Loading…",
            theme.get(SemanticRole::Muted),
        );
        return;
    }

    if let Some(error) = state.diff.error.as_deref() {
        render_message(
            frame,
            scroll_content,
            error,
            theme.get(SemanticRole::AgentError),
        );
        return;
    }

    if state.diff.is_binary {
        render_message(
            frame,
            scroll_content,
            "Binary diff not supported",
            theme.get(SemanticRole::Muted),
        );
        return;
    }

    if state.diff.lines.is_empty() {
        render_message(
            frame,
            scroll_content,
            empty_diff_message(state.diff.source),
            theme.get(SemanticRole::Muted),
        );
        return;
    }

    render_virtualized_lines(frame, scroll_content, state, theme, scroll_visible_rows);
    if let Some(scrollbar_area) = scrollbar {
        render_vertical_scrollbar(
            frame,
            scrollbar_area,
            state.diff.scroll_offset,
            state.diff.lines.len(),
            scroll_visible_rows,
            focused,
            theme,
        );
    }
}

fn empty_diff_message(source: DiffSource) -> &'static str {
    match source {
        DiffSource::Staged => {
            "No staged changes for this file (s toggles view; use shell to git add)"
        }
        DiffSource::Unstaged => "No unstaged changes for this file (s toggles view)",
    }
}

fn render_virtualized_lines(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &ThemePalette,
    visible_rows: usize,
) {
    let gutter_width = gutter_width(&state.diff.lines);
    let text_width = area.width.saturating_sub(gutter_width as u16) as usize;
    if text_width == 0 {
        return;
    }

    let old_width = max_lineno_digits(state.diff.lines.iter().filter_map(|line| line.old_lineno));
    let new_width = max_lineno_digits(state.diff.lines.iter().filter_map(|line| line.new_lineno));
    let gutter_style = theme.get(SemanticRole::Muted);
    let word_wrap = state.config.diff.word_wrap;
    let horizontal_offset = if word_wrap {
        0
    } else {
        state.diff.horizontal_scroll_offset
    };

    let start = state.diff.scroll_offset;
    let end = (start + visible_rows).min(state.diff.lines.len());

    for (row, line_index) in (start..end).enumerate() {
        let line = &state.diff.lines[line_index];
        let mut spans = Vec::new();

        if gutter_width > 0 {
            spans.push(Span::styled(
                format_gutter(line.old_lineno, line.new_lineno, old_width, new_width),
                gutter_style,
            ));
        }

        let display = format_line_content(line, text_width, horizontal_offset, word_wrap);
        spans.push(Span::styled(display, style_for_line_kind(line.kind, theme)));

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

pub(crate) fn style_for_line_kind(kind: DiffLineKind, theme: &ThemePalette) -> Style {
    match kind {
        DiffLineKind::Addition => theme.get(SemanticRole::GitAdded),
        DiffLineKind::Deletion => theme.get(SemanticRole::GitDeleted),
        DiffLineKind::Context => theme.get(SemanticRole::Muted),
        DiffLineKind::Header => theme.get(SemanticRole::Muted).add_modifier(Modifier::BOLD),
    }
}

fn gutter_width(lines: &[DiffLine]) -> usize {
    let old_width = max_lineno_digits(lines.iter().filter_map(|line| line.old_lineno));
    let new_width = max_lineno_digits(lines.iter().filter_map(|line| line.new_lineno));
    if old_width == 0 && new_width == 0 {
        return 0;
    }
    old_width + 1 + new_width + 2
}

fn max_lineno_digits(values: impl Iterator<Item = u32>) -> usize {
    values
        .map(|value| value.ilog10() as usize + 1)
        .max()
        .unwrap_or(0)
}

fn format_gutter(
    old_lineno: Option<u32>,
    new_lineno: Option<u32>,
    old_width: usize,
    new_width: usize,
) -> String {
    let old = old_lineno
        .map(|value| format!("{value:>old_width$}"))
        .unwrap_or_else(|| " ".repeat(old_width));
    let new = new_lineno
        .map(|value| format!("{value:>new_width$}"))
        .unwrap_or_else(|| " ".repeat(new_width));
    format!("{old} {new} ")
}

fn format_line_content(
    line: &DiffLine,
    text_width: usize,
    horizontal_offset: usize,
    word_wrap: bool,
) -> String {
    if word_wrap {
        return truncate_line(&line.content, text_width);
    }

    let sliced = slice_line(&line.content, horizontal_offset, text_width);
    truncate_line(&sliced, text_width)
}

fn slice_line(text: &str, offset: usize, width: usize) -> String {
    if offset == 0 {
        return text.to_string();
    }

    let chars: Vec<char> = text.chars().skip(offset).collect();
    if chars.len() <= width {
        return chars.into_iter().collect();
    }

    chars[..width].iter().collect()
}

fn format_diff_status(state: &AppState) -> String {
    let diff = &state.diff;
    if diff.loading && diff.lines.is_empty() {
        return "Loading…".to_string();
    }

    let path = diff.selected_path.as_deref().unwrap_or("No file selected");
    let source = match diff.source {
        DiffSource::Unstaged => "view: unstaged",
        DiffSource::Staged => "view: staged",
    };

    if diff.is_binary {
        return format!("{path} | {source} | binary");
    }

    if diff.error.is_some() {
        return path.to_string();
    }

    if diff.loading {
        return format!("{path} | {source} | reloading…");
    }

    format!("{path} | {source} | {} lines", diff.lines.len())
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
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::config::ResolvedConfig;
    use crate::diff::{DiffLine, DiffLineKind, DiffSource};
    use crate::layout::compute_layout;
    use crate::state::{AppState, DiffState};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state_with_diff(lines: Vec<DiffLine>, scroll_offset: usize) -> AppState {
        let mut state = AppState::from_startup(
            std::path::PathBuf::from("."),
            false,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        );
        state.diff = DiffState {
            selected_path: Some("src/main.rs".to_string()),
            source: DiffSource::Unstaged,
            lines,
            scroll_offset,
            ..DiffState::default()
        };
        state
    }

    #[test]
    fn style_for_line_kind_uses_git_semantic_roles() {
        let theme = load_theme_with_capabilities(
            &ResolvedConfig::default().theme,
            TerminalCapabilities::TrueColor,
        )
        .expect("theme");

        assert_eq!(
            style_for_line_kind(DiffLineKind::Addition, &theme).fg,
            theme.get(SemanticRole::GitAdded).fg
        );
        assert_eq!(
            style_for_line_kind(DiffLineKind::Deletion, &theme).fg,
            theme.get(SemanticRole::GitDeleted).fg
        );
        assert_eq!(
            style_for_line_kind(DiffLineKind::Context, &theme).fg,
            theme.get(SemanticRole::Muted).fg
        );
    }

    #[test]
    fn virtualized_render_shows_viewport_lines_only() {
        let lines = (0..1000)
            .map(|index| DiffLine {
                kind: DiffLineKind::Context,
                content: format!(" context {index}"),
                old_lineno: Some(index as u32 + 1),
                new_lineno: Some(index as u32 + 1),
            })
            .collect();
        let state = test_state_with_diff(lines, 500);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let area = Rect::new(0, 0, 80, 20);

        terminal
            .draw(|frame| {
                render_diff_pane(frame, area, true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(content.contains("context 500"));
        assert!(content.contains("context 509"));
        assert!(!content.contains("context 0"));
        assert!(!content.contains("context 999"));
    }

    #[test]
    fn addition_lines_render_with_git_added_color() {
        let state = test_state_with_diff(
            vec![DiffLine {
                kind: DiffLineKind::Addition,
                content: "+added line".to_string(),
                old_lineno: None,
                new_lineno: Some(1),
            }],
            0,
        );
        let theme = &state.theme;
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| {
                render_diff_pane(frame, Rect::new(0, 0, 80, 8), true, theme, &state);
            })
            .expect("draw");

        let expected = theme.get(SemanticRole::GitAdded).fg;
        let colored = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .any(|cell| cell.symbol() == "+" && Some(cell.fg) == expected);
        assert!(colored, "expected git_added color on + line");
    }

    #[test]
    fn binary_diff_shows_unsupported_message() {
        let mut state = test_state_with_diff(Vec::new(), 0);
        state.diff.is_binary = true;
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| {
                render_diff_pane(frame, Rect::new(0, 0, 80, 8), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("Binary diff not supported"));
    }

    #[test]
    fn empty_staged_diff_shows_view_hint() {
        let mut state = test_state_with_diff(Vec::new(), 0);
        state.diff.source = DiffSource::Staged;
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| {
                render_diff_pane(frame, Rect::new(0, 0, 80, 8), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("No staged changes"));
    }
}
