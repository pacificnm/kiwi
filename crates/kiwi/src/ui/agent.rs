use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::state::AppState;
use crate::theme::ThemePalette;

use super::scrollback::{render_scrollback_pane, ScrollbackPane};

pub fn render_agent_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    focused: bool,
    theme: &ThemePalette,
    chrome: Style,
    state: &AppState,
) {
    render_scrollback_pane(
        frame,
        area,
        title,
        focused,
        theme,
        chrome,
        ScrollbackPane {
            scrollback: &state.agent.scrollback,
            follow_tail: state.agent.follow_tail,
            viewport_offset: state.agent.viewport_offset,
            spawn_error: state.agent.spawn_error.as_deref(),
            idle_hint: None,
        },
    );
}
