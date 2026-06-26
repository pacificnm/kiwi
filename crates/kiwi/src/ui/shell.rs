use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::selection::SelectionPane;
use crate::state::AppState;
use crate::theme::ThemePalette;

use super::scrollback::{render_scrollback_pane, ScrollbackPane};

pub fn render_shell_pane(
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
            scrollback: &state.shell.scrollback,
            follow_tail: state.shell.follow_tail,
            viewport_offset: state.shell.viewport_offset,
            spawn_error: state.shell.spawn_error.as_deref(),
            idle_hint: if state.shell.scrollback.line_count() > 0
                || state.shell.scrollback.has_pending_line()
            {
                None
            } else if focused {
                Some("Starting shell…")
            } else {
                Some("Click or Tab to focus shell")
            },
            footer: None,
            selection_pane: Some(SelectionPane::Shell),
            show_pty_cursor: focused && state.shell.running && state.pty_cursor_blink_on,
        },
        &state.text_selection,
    );
}
