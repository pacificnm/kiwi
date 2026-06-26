use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::selection::SelectionPane;
use crate::state::AppState;
use crate::theme::ThemePalette;

use super::scrollback::{render_scrollback_pane, ScrollbackPane};

pub fn render_agent_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    focused: bool,
    theme: &ThemePalette,
    hint_style: Style,
    state: &AppState,
) {
    render_scrollback_pane(
        frame,
        area,
        title,
        focused,
        theme,
        hint_style,
        ScrollbackPane {
            scrollback: &state.agent.scrollback,
            follow_tail: state.agent.follow_tail,
            viewport_offset: state.agent.viewport_offset,
            spawn_error: None,
            idle_hint: None,
            footer: state.agent.restart_hint.as_deref(),
            selection_pane: Some(SelectionPane::Agent),
            show_pty_cursor: focused
                && state.agent.running
                && state.agent.follow_tail
                && state.pty_cursor_blink_on,
        },
        &state.text_selection,
    );
}
