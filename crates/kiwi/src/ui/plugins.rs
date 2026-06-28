use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::state::{AppState, PluginStatus};
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn render_plugins_pane(
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

    let outer = Block::default()
        .title(" Plugin Manager ")
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    let inner = outer.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(outer, area);

    if inner.height == 0 || inner.width < 20 {
        return;
    }

    let entries = &state.plugins.entries;

    if entries.is_empty() {
        let msg = Paragraph::new(Line::from(vec![Span::styled(
            "No plugins installed. Use `kiwi plugin install <path>` to add one.",
            theme.get(SemanticRole::Muted),
        )]))
        .style(chrome_style(theme));
        frame.render_widget(msg, inner);
        return;
    }

    // Split into list (left, ~40%) and detail (right, ~60%).
    let list_width = (inner.width * 2 / 5).max(18);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(list_width), Constraint::Min(0)])
        .split(inner);

    render_plugin_list(frame, chunks[0], focused, theme, state);
    render_plugin_detail(frame, chunks[1], focused, theme, state);
}

fn render_plugin_list(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let entries = &state.plugins.entries;
    let selected = state.plugins.selected_index.min(entries.len().saturating_sub(1));

    let items: Vec<ListItem<'_>> = entries
        .iter()
        .map(|entry| {
            let (badge, badge_style) = status_badge(&entry.status, theme);
            let name_style = if entry.enabled {
                chrome_style(theme)
            } else {
                theme.get(SemanticRole::Muted)
            };
            ListItem::new(Line::from(vec![
                Span::styled(badge, badge_style),
                Span::raw(" "),
                Span::styled(entry.display_name.as_str(), name_style),
            ]))
        })
        .collect();

    let highlight_style = theme
        .get(SemanticRole::Accent)
        .add_modifier(Modifier::REVERSED);

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title("Plugins")
                .borders(Borders::RIGHT)
                .border_style(border_style)
                .style(chrome_style(theme)),
        )
        .highlight_style(highlight_style);

    let mut list_state = ListState::default().with_selected(Some(selected));
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_plugin_detail(
    frame: &mut Frame<'_>,
    area: Rect,
    _focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let entries = &state.plugins.entries;
    if entries.is_empty() || area.width == 0 {
        return;
    }

    let selected = state.plugins.selected_index.min(entries.len().saturating_sub(1));
    let entry = &entries[selected];

    let chrome = chrome_style(theme);
    let muted = theme.get(SemanticRole::Muted);
    let accent = theme.get(SemanticRole::Accent);
    let (badge_text, badge_style) = status_badge(&entry.status, theme);

    let mut lines: Vec<Line<'_>> = Vec::new();

    // Name + version header
    lines.push(Line::from(vec![
        Span::styled(entry.display_name.clone(), accent.add_modifier(Modifier::BOLD)),
        Span::styled(format!("  v{}", entry.version), muted),
    ]));

    // Status row
    lines.push(Line::from(vec![
        Span::styled("Status:  ", muted),
        Span::styled(badge_text, badge_style),
        if !entry.enabled {
            Span::styled("  (disabled)", muted)
        } else {
            Span::raw("")
        },
    ]));

    // Author
    if !entry.author.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Author:  ", muted),
            Span::styled(entry.author.clone(), chrome),
        ]));
    }

    // Description
    if !entry.description.is_empty() {
        lines.push(Line::raw(""));
        for desc_line in entry.description.lines() {
            lines.push(Line::from(Span::styled(desc_line.to_owned(), chrome)));
        }
    }

    // Commands
    if !entry.command_ids.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled("Commands:", muted)));
        for id in &entry.command_ids {
            lines.push(Line::from(vec![
                Span::styled("  • ", muted),
                Span::styled(id.clone(), chrome),
            ]));
        }
    }

    // Failure / incompatibility reason
    match &entry.status {
        PluginStatus::Failed(reason) | PluginStatus::Incompatible(reason) => {
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::styled("Reason:  ", muted),
                Span::styled(reason.clone(), theme.get(SemanticRole::AgentError)),
            ]));
        }
        _ => {}
    }

    // Hint line at bottom
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "Use `kiwi plugin enable/disable <name>` to toggle, then restart.",
        muted,
    )));

    let scroll = state.plugins.detail_scroll as u16;
    let para = Paragraph::new(lines)
        .style(chrome)
        .scroll((scroll, 0))
        .block(Block::default().style(chrome));

    frame.render_widget(para, area);
}

fn status_badge<'a>(status: &'a PluginStatus, theme: &ThemePalette) -> (String, Style) {
    match status {
        PluginStatus::Loaded => ("●".to_string(), theme.get(SemanticRole::GitAdded)),
        PluginStatus::Disabled => ("○".to_string(), theme.get(SemanticRole::Muted)),
        PluginStatus::Failed(_) => ("✗".to_string(), theme.get(SemanticRole::AgentError)),
        PluginStatus::Incompatible(_) => ("⚠".to_string(), theme.get(SemanticRole::GitModified)),
        PluginStatus::Missing => ("?".to_string(), theme.get(SemanticRole::AgentError)),
    }
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
    use crate::layout::compute_layout;
    use crate::state::{AppState, PluginEntry, PluginStatus, PluginsState};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            std::path::PathBuf::from("."),
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
    fn renders_empty_state_without_panic() {
        let state = test_state();
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_plugins_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");
        let buf: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(buf.contains("Plugin Manager") || buf.contains("No plugins"));
    }

    #[test]
    fn renders_plugin_list_with_entries() {
        let mut state = test_state();
        state.plugins = PluginsState {
            entries: vec![
                PluginEntry {
                    name: "hello".to_string(),
                    display_name: "Hello Plugin".to_string(),
                    version: "0.1.0".to_string(),
                    description: "Says hello.".to_string(),
                    author: "Alice".to_string(),
                    enabled: true,
                    status: PluginStatus::Loaded,
                    command_ids: vec!["hello.greet".to_string()],
                },
                PluginEntry {
                    name: "ollama".to_string(),
                    display_name: "Ollama".to_string(),
                    version: "0.2.0".to_string(),
                    description: "".to_string(),
                    author: "".to_string(),
                    enabled: false,
                    status: PluginStatus::Disabled,
                    command_ids: vec![],
                },
            ],
            selected_index: 0,
            ..PluginsState::default()
        };

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_plugins_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let buf: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(buf.contains("Hello Plugin"));
        assert!(buf.contains("Ollama"));
    }
}
