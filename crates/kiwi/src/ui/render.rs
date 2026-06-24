use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::bootstrap::StartupContext;
use crate::layout::{LayoutState, Region};
use crate::navigation::{NavigationState, LEFT_TAB_LABELS, MAIN_TAB_LABELS};
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::tabs::tab_bar_line;

pub fn draw_frame(frame: &mut Frame<'_>, context: &StartupContext, nav: &NavigationState) {
    let chrome = chrome_style(&context.theme);
    frame.render_widget(Clear, frame.area());
    frame.render_widget(Paragraph::new("").style(chrome), frame.area());

    render_tab_bar(
        frame,
        context.layout.rects.left_tabs,
        &LEFT_TAB_LABELS,
        nav.left_tab.index(),
        &context.theme,
        chrome,
    );
    render_tab_bar(
        frame,
        context.layout.rects.main_tabs,
        &MAIN_TAB_LABELS,
        nav.main_tab.index(),
        &context.theme,
        chrome,
    );

    render_pane(
        frame,
        context.layout.rects.left_content,
        nav.left_tab.label(),
        nav.focus.is_focused(Region::LeftContent),
        &context.theme,
        chrome,
        Some(left_pane_line(nav)),
    );
    render_pane(
        frame,
        context.layout.rects.main_content,
        nav.main_tab.label(),
        nav.focus.is_focused(Region::MainContent),
        &context.theme,
        chrome,
        Some(main_pane_line(nav)),
    );
    render_pane(
        frame,
        context.layout.rects.palette,
        "Commands",
        nav.focus.is_focused(Region::Palette),
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
        nav.focus.is_focused(Region::Shell),
        &context.theme,
        chrome,
        None,
    );

    render_status_placeholder(frame, &context.layout, &context.theme);
}

fn left_pane_line(nav: &NavigationState) -> Line<'_> {
    let slot = nav.left_slot();
    Line::from(format!(
        "{} view (selection: {})",
        nav.left_tab.label(),
        slot.selected_index
    ))
}

fn main_pane_line(nav: &NavigationState) -> Line<'_> {
    let slot = nav.main_slot();
    Line::from(format!(
        "{} view (selection: {})",
        nav.main_tab.label(),
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

    use crate::bootstrap::StartupContext;
    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::navigation::{
        LeftNavTab, MainTab, NavCommand, NavigationState, LEFT_TAB_LABELS, MAIN_TAB_LABELS,
    };
    use crate::terminal::TerminalGuard;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

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
        let nav = NavigationState::default();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| draw_frame(frame, &context, &nav))
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
        let context = test_context();
        let mut nav = NavigationState::default();
        nav.apply(NavCommand::SelectLeftTab(LeftNavTab::Git));
        nav.apply(NavCommand::SelectMainTab(MainTab::Issues));

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &context, &nav))
            .expect("draw");

        let content = buffer_content(terminal.backend().buffer());
        assert!(content.contains("Git view"));
        assert!(content.contains("Issues view"));
    }

    fn buffer_content(buffer: &ratatui::buffer::Buffer) -> String {
        buffer.content.iter().map(|cell| cell.symbol()).collect()
    }
}
