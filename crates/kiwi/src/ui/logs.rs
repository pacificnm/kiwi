use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::state::{AppState, LogLevel};
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn render_logs_pane(
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
    let chrome = chrome_style(theme);

    let block = Block::default()
        .title("Logs")
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let items: Vec<ListItem<'_>> = state
        .logs
        .entries
        .iter()
        .rev()
        .take(inner.height as usize)
        .map(|entry| {
            let prefix = match entry.level {
                LogLevel::Info => "INFO",
                LogLevel::Error => "ERR ",
            };
            let level_style = match entry.level {
                LogLevel::Info => theme.get(SemanticRole::Muted),
                LogLevel::Error => theme.get(SemanticRole::AgentError),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{prefix} "), level_style),
                Span::styled(entry.message.as_str(), chrome),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
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
    use crate::state::{AppState, LogEntry, LogLevel, LogsState};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("."),
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
    fn render_logs_pane_shows_recent_entries() {
        let mut state = test_state();
        state.logs = LogsState {
            entries: vec![LogEntry {
                level: LogLevel::Info,
                message: "Launched editor `nvim` for /tmp/a.rs".to_string(),
            }],
        };

        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_logs_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(content.contains("Launched editor"));
    }
}
