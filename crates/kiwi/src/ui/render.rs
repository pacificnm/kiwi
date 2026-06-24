use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::layout::{LayoutState, Region};
use crate::navigation::{LEFT_TAB_LABELS, MAIN_TAB_LABELS};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::tabs::tab_bar_line;

pub fn draw_frame(frame: &mut Frame<'_>, state: &AppState) {
    let chrome = chrome_style(&state.theme);
    frame.render_widget(Clear, frame.area());
    frame.render_widget(Paragraph::new("").style(chrome), frame.area());

    render_tab_bar(
        frame,
        state.layout.rects.left_tabs,
        &LEFT_TAB_LABELS,
        state.navigation.left_tab.index(),
        &state.theme,
        chrome,
    );
    render_tab_bar(
        frame,
        state.layout.rects.main_tabs,
        &MAIN_TAB_LABELS,
        state.navigation.main_tab.index(),
        &state.theme,
        chrome,
    );

    render_pane(
        frame,
        state.layout.rects.left_content,
        state.navigation.left_tab.label(),
        state.navigation.focus.is_focused(Region::LeftContent),
        &state.theme,
        chrome,
        Some(left_pane_line(state)),
    );
    render_pane(
        frame,
        state.layout.rects.main_content,
        state.navigation.main_tab.label(),
        state.navigation.focus.is_focused(Region::MainContent),
        &state.theme,
        chrome,
        Some(main_pane_line(state)),
    );
    render_pane(
        frame,
        state.layout.rects.palette,
        "Commands",
        state.navigation.focus.is_focused(Region::Palette),
        &state.theme,
        chrome,
        Some(Line::from(Span::styled(
            "Ctrl+P for commands",
            state.theme.get(SemanticRole::Muted),
        ))),
    );

    let shell_title = format!("Shell: {}", state.config.shell.command);
    render_pane(
        frame,
        state.layout.rects.shell,
        &shell_title,
        state.navigation.focus.is_focused(Region::Shell),
        &state.theme,
        chrome,
        None,
    );

    render_status_placeholder(frame, &state.layout, &state.theme);
}

fn left_pane_line(state: &AppState) -> Line<'_> {
    let slot = state.navigation.left_slot();
    Line::from(format!(
        "{} view (selection: {})",
        state.navigation.left_tab.label(),
        slot.selected_index
    ))
}

fn main_pane_line(state: &AppState) -> Line<'_> {
    let slot = state.navigation.main_slot();
    Line::from(format!(
        "{} view (selection: {})",
        state.navigation.main_tab.label(),
        slot.selected_index
    ))
}

fn render_tab_bar(
    frame: &mut Frame<'_>,
    area: Rect,
    tabs: &[&'static str],
    selected: usize,
    theme: &ThemePalette,
    chrome: Style,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let line = tab_bar_line(tabs, selected, theme);
    frame.render_widget(Paragraph::new(line).style(chrome), area);
}

fn render_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    focused: bool,
    theme: &ThemePalette,
    chrome: Style,
    content: Option<Line<'_>>,
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
        .border_style(border_style)
        .style(chrome);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(line) = content {
        frame.render_widget(Paragraph::new(line).style(chrome), inner);
    }
}

fn render_status_placeholder(frame: &mut Frame<'_>, layout: &LayoutState, theme: &ThemePalette) {
    let area = layout.rects.status_bar;
    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut style = theme.get(SemanticRole::Selection);
    if style.fg.is_none() {
        if let Some(fg) = theme.get(SemanticRole::Fg).fg {
            style = style.fg(fg);
        }
    }

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(" Kiwi ", style))),
        area,
    );
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
    use crate::navigation::{LeftNavTab, MainTab, NavCommand};
    use crate::state::AppState;
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
    fn draw_frame_renders_tab_labels_and_pane_titles() {
        let state = test_state();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let content = buffer_content(buffer);

        for label in LEFT_TAB_LABELS {
            assert!(content.contains(label), "missing left tab {label}");
        }
        for label in MAIN_TAB_LABELS {
            assert!(content.contains(label), "missing main tab {label}");
        }
        assert!(content.contains("Shell: bash"));
        assert!(content.contains("Ctrl+P for commands"));
        assert!(content.contains("Files view"));
        assert!(content.contains("Agent view"));
    }

    #[test]
    fn draw_frame_reflects_orthogonal_tab_selection() {
        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Git));
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let content = buffer_content(terminal.backend().buffer());
        assert!(content.contains("Git view"));
        assert!(content.contains("Issues view"));
    }

    fn buffer_content(buffer: &ratatui::buffer::Buffer) -> String {
        buffer.content.iter().map(|cell| cell.symbol()).collect()
    }
}
