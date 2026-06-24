use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::bootstrap::StartupContext;
use crate::layout::{LayoutState, Region};
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::state::UiState;
use super::tabs::{tab_bar_line, LEFT_TAB_LABELS, MAIN_TAB_LABELS};

pub fn draw_frame(frame: &mut Frame<'_>, context: &StartupContext, ui: &UiState) {
    let chrome = chrome_style(&context.theme);
    frame.render_widget(Clear, frame.area());
    frame.render_widget(Paragraph::new("").style(chrome), frame.area());

    let left_selected = ui.left_tab.min(LEFT_TAB_LABELS.len().saturating_sub(1));
    let main_selected = ui.main_tab.min(MAIN_TAB_LABELS.len().saturating_sub(1));

    render_tab_bar(
        frame,
        context.layout.rects.left_tabs,
        LEFT_TAB_LABELS,
        left_selected,
        &context.theme,
        chrome,
    );
    render_tab_bar(
        frame,
        context.layout.rects.main_tabs,
        MAIN_TAB_LABELS,
        main_selected,
        &context.theme,
        chrome,
    );

    render_pane(
        frame,
        context.layout.rects.left_content,
        LEFT_TAB_LABELS[left_selected],
        ui.focus.is_focused(Region::LeftContent),
        &context.theme,
        chrome,
        None,
    );
    render_pane(
        frame,
        context.layout.rects.main_content,
        MAIN_TAB_LABELS[main_selected],
        ui.focus.is_focused(Region::MainContent),
        &context.theme,
        chrome,
        None,
    );
    render_pane(
        frame,
        context.layout.rects.palette,
        "Commands",
        ui.focus.is_focused(Region::Palette),
        &context.theme,
        chrome,
        Some(Line::from(Span::styled(
            "Ctrl+P for commands",
            context.theme.get(SemanticRole::Muted),
        ))),
    );

    let shell_title = format!("Shell: {}", context.config.shell.command);
    render_pane(
        frame,
        context.layout.rects.shell,
        &shell_title,
        ui.focus.is_focused(Region::Shell),
        &context.theme,
        chrome,
        None,
    );

    render_status_placeholder(frame, &context.layout, &context.theme, chrome);
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
    content: Option<Line<'static>>,
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

fn render_status_placeholder(
    frame: &mut Frame<'_>,
    layout: &LayoutState,
    theme: &ThemePalette,
    _chrome: Style,
) {
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

    use crate::bootstrap::StartupContext;
    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::terminal::TerminalGuard;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;
    use crate::ui::state::UiState;

    fn test_context() -> StartupContext {
        StartupContext {
            repo_root: PathBuf::from("."),
            is_git_repo: false,
            config: ResolvedConfig::default(),
            theme: load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            layout: compute_layout(120, 40, 30).expect("layout"),
            terminal: TerminalGuard::inactive(),
        }
    }

    #[test]
    fn draw_frame_renders_tab_labels_and_pane_titles() {
        let context = test_context();
        let ui = UiState::default();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| draw_frame(frame, &context, &ui))
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
    }

    fn buffer_content(buffer: &ratatui::buffer::Buffer) -> String {
        buffer.content.iter().map(|cell| cell.symbol()).collect()
    }
}
