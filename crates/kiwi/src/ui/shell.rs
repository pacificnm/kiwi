use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn render_shell_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    focused: bool,
    theme: &ThemePalette,
    chrome: Style,
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

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let visible_height = inner.height as usize;
    let max_width = inner.width as usize;
    let start = state.shell.scrollback.viewport_start(
        visible_height,
        state.shell.follow_tail,
        state.shell.viewport_offset,
    );
    let lines = state
        .shell
        .scrollback
        .visible_lines(start, visible_height, max_width);

    if lines.is_empty() {
        if let Some(error) = &state.shell.spawn_error {
            frame.render_widget(
                Paragraph::new(Line::from(error.clone())).style(chrome),
                inner,
            );
        }
        return;
    }

    let text = Text::from(lines.into_iter().map(Line::from).collect::<Vec<_>>());
    frame.render_widget(Paragraph::new(text).style(chrome), inner);
}
