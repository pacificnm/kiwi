use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::search::{SearchMode, SearchResult};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::scrollbar::{render_vertical_scrollbar, split_for_scrollbar};

const QUERY_ROWS: u16 = 1;
const STATUS_ROWS: u16 = 1;

pub fn search_interaction_at(state: &AppState, area: Rect, column: u16, row: u16) -> Option<usize> {
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

    let results_area = search_results_area(pane_inner(area)?);
    if column < results_area.x
        || column >= results_area.x.saturating_add(results_area.width)
        || row < results_area.y
        || row >= results_area.y.saturating_add(results_area.height)
    {
        return None;
    }

    let viewport_index = usize::from(row.saturating_sub(results_area.y));
    let index = state.search.scroll_offset.saturating_add(viewport_index);
    if index < state.search.results.len() {
        Some(index)
    } else {
        None
    }
}

fn pane_inner(area: Rect) -> Option<Rect> {
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    if inner.width == 0 || inner.height == 0 {
        None
    } else {
        Some(inner)
    }
}

fn search_results_area(inner: Rect) -> Rect {
    let results_top = inner.y.saturating_add(QUERY_ROWS);
    let results_height =
        inner
            .height
            .saturating_sub(QUERY_ROWS)
            .saturating_sub(if inner.height > QUERY_ROWS {
                STATUS_ROWS
            } else {
                0
            });
    Rect {
        x: inner.x,
        y: results_top,
        width: inner.width,
        height: results_height,
    }
}

pub fn render_search_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let title = format!("Search: {}", state.search.mode.label());
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

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let Some(inner) = pane_inner(area) else {
        return;
    };

    let query_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: QUERY_ROWS.min(inner.height),
    };
    render_query_line(frame, query_area, state, theme);

    let status_y = inner
        .y
        .saturating_add(inner.height.saturating_sub(STATUS_ROWS));
    let status_area = Rect {
        x: inner.x,
        y: status_y,
        width: inner.width,
        height: STATUS_ROWS.min(inner.height),
    };

    let results_area = search_results_area(inner);

    if results_area.height > 0 && results_area.width > 0 {
        render_results(frame, results_area, focused, theme, state);
    }

    if status_area.height > 0 {
        render_status_line(frame, status_area, state, theme);
    }
}

fn render_query_line(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &ThemePalette) {
    let query = if state.search.query.is_empty() {
        "/ type to search".to_string()
    } else {
        format!("/{}", state.search.query)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Query ", theme.get(SemanticRole::Muted)),
            Span::styled(
                truncate_line(&query, area.width as usize),
                theme.get(SemanticRole::Fg),
            ),
        ]))
        .style(chrome_style(theme)),
        area,
    );
}

fn render_status_line(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &ThemePalette) {
    let status = if state.search.running {
        "Searching…".to_string()
    } else if let Some(error) = &state.search.error {
        error.clone()
    } else if state.search.truncated {
        format!(
            "{} results (truncated) · Enter open · e editor · Ctrl+M mode",
            state.search.results.len()
        )
    } else if state.search.query.is_empty() {
        "Type to search · Ctrl+M mode".to_string()
    } else if state.search.results.is_empty() {
        "No results · Ctrl+M mode".to_string()
    } else {
        format!(
            "{} results · Enter open · e editor · Ctrl+M mode",
            state.search.results.len()
        )
    };

    frame.render_widget(
        Paragraph::new(truncate_line(&status, area.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn render_results(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let (content, scrollbar) = split_for_scrollbar(area);
    let viewport_rows = content.height as usize;
    let max_width = content.width as usize;
    let total_rows = state.search.results.len();
    let mut lines = Vec::new();

    for viewport_index in 0..viewport_rows {
        let Some(line) = render_result_line(state, theme, viewport_index, max_width, focused)
        else {
            break;
        };
        lines.push(line);
    }

    if lines.is_empty() {
        let hint = if state.search.query.is_empty() {
            "Type to search files or content"
        } else if state.search.running {
            "Searching…"
        } else {
            "No matches"
        };
        lines.push(Line::from(Span::styled(
            truncate_line(hint, max_width),
            theme.get(SemanticRole::Muted),
        )));
    }

    frame.render_widget(Clear, content);
    frame.render_widget(Paragraph::new(lines).style(chrome_style(theme)), content);
    if let Some(scrollbar_area) = scrollbar {
        render_vertical_scrollbar(
            frame,
            scrollbar_area,
            state.search.scroll_offset,
            total_rows,
            viewport_rows,
            focused,
            theme,
        );
    }
}

fn render_result_line(
    state: &AppState,
    theme: &ThemePalette,
    viewport_index: usize,
    max_width: usize,
    focused: bool,
) -> Option<Line<'static>> {
    let result = state.search.result_at_viewport(viewport_index)?;
    let row_index = state.search.scroll_offset + viewport_index;
    let selected = row_index == state.search.selected;

    let mut style = theme.get(SemanticRole::Fg);
    if selected {
        style = if focused {
            theme.get(SemanticRole::Accent)
        } else {
            theme.get(SemanticRole::Selection)
        };
        style = style.add_modifier(Modifier::BOLD);
    }

    let label = result_label(result, state.search.mode);
    let prefix = if selected { "▸ " } else { "  " };
    Some(Line::from(Span::styled(
        truncate_line(&format!("{prefix}{label}"), max_width),
        style,
    )))
}

fn result_label(result: &SearchResult, mode: SearchMode) -> String {
    match mode {
        SearchMode::Files => result.id.clone(),
        SearchMode::Content => {
            if result.preview.is_empty() {
                result.id.clone()
            } else {
                format!("{}  {}", result.id, result.preview)
            }
        }
    }
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
    use crate::search::{SearchResult, SearchState};
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    #[test]
    fn render_search_pane_shows_query_and_status() {
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
        state.search = SearchState {
            query: "main".to_string(),
            running: true,
            ..SearchState::default()
        };

        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_search_pane(frame, Rect::new(0, 0, 80, 10), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("/main"));
        assert!(content.contains("Searching"));
    }

    #[test]
    fn render_search_pane_lists_result_paths() {
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
        state.search = SearchState {
            query: "main".to_string(),
            results: vec![
                SearchResult::file(PathBuf::from("src/main.rs"), "src/main.rs".to_string()),
                SearchResult::file(PathBuf::from("lib/main.rs"), "lib/main.rs".to_string()),
            ],
            ..SearchState::default()
        };

        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_search_pane(frame, Rect::new(0, 0, 80, 12), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("lib/main.rs"));
        assert!(content.contains("2 results"));
    }

    #[test]
    fn search_interaction_at_selects_result_row() {
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
        state.search = SearchState {
            query: "main".to_string(),
            results: vec![
                SearchResult::file(PathBuf::from("src/main.rs"), "src/main.rs".to_string()),
                SearchResult::file(PathBuf::from("lib/main.rs"), "lib/main.rs".to_string()),
            ],
            ..SearchState::default()
        };

        let area = Rect::new(0, 0, 80, 12);
        let inner = pane_inner(area).expect("inner");
        let results = search_results_area(inner);
        let index = search_interaction_at(&state, area, results.x, results.y.saturating_add(1))
            .expect("second row");
        assert_eq!(index, 1);
    }
}
