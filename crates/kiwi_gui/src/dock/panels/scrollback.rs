//! Shared PTY scrollback rendering for Terminal and Agent panels.

use egui::{RichText, Ui};
use kiwi_core::shell::ScrollbackBuffer;
use kiwi_core::theme::SemanticRole;

use super::ansi::{ansi_layout_job, max_cols_for_ui, monospace_font_id};
use super::layout::{pty_dimensions_from_ui, render_virtual_rows, row_count_for_height, PTY_ROW_HEIGHT};
use crate::dock::context::PanelContext;
use crate::theme::GuiTheme;

pub struct PtyScrollbackView<'a> {
    pub scrollback: &'a ScrollbackBuffer,
    pub follow_tail: bool,
    pub viewport_offset: usize,
    pub spawn_error: Option<&'a str>,
    pub idle_hint: Option<&'a str>,
}

pub fn render_pty_scrollback(
    ui: &mut Ui,
    theme: &GuiTheme,
    pane: PtyScrollbackView<'_>,
    follow_tail: &mut bool,
    viewport_offset: &mut usize,
    viewport_rows_out: &mut usize,
) {
    if let Some(error) = pane.spawn_error {
        ui.colored_label(theme.role(SemanticRole::AgentError), error);
        *viewport_rows_out = 1;
        return;
    }

    let line_count = pane.scrollback.line_count();
    if line_count == 0 {
        if let Some(hint) = pane.idle_hint {
            ui.colored_label(theme.role(SemanticRole::Muted), hint);
        }
        *viewport_rows_out = 1;
        return;
    }

    let font_id = monospace_font_id(ui, theme.font_size);
    let max_cols = max_cols_for_ui(ui, &font_id);
    let viewport_estimate = row_count_for_height(ui.clip_rect().height(), PTY_ROW_HEIGHT);
    let mut scroll_row = pane.scrollback.viewport_start(
        viewport_estimate,
        pane.follow_tail,
        pane.viewport_offset,
    );

    let include_pending = *follow_tail;
    let viewport_rows = render_virtual_rows(
        ui,
        PTY_ROW_HEIGHT,
        line_count,
        &mut scroll_row,
        |ui, row_index| {
            let lines = pane.scrollback.viewport_lines(
                row_index,
                1,
                max_cols,
                include_pending && row_index + 1 >= line_count,
            );
            let text = lines.first().map(String::as_str).unwrap_or("");
            let job = ansi_layout_job(text, max_cols, font_id.clone());
            ui.label(job);
        },
    );

    sync_viewport_scroll(
        scroll_row,
        line_count,
        viewport_rows,
        follow_tail,
        viewport_offset,
    );
    *viewport_rows_out = viewport_rows;
}

pub fn render_pty_footer(ui: &mut Ui, theme: &GuiTheme, footer: Option<&str>) {
    if let Some(text) = footer {
        ui.add_space(4.0);
        ui.separator();
        ui.colored_label(theme.role(SemanticRole::Muted), text);
    }
}

fn sync_viewport_scroll(
    scroll_row: usize,
    line_count: usize,
    viewport_rows: usize,
    follow_tail: &mut bool,
    viewport_offset: &mut usize,
) {
    let max_start = line_count.saturating_sub(viewport_rows.max(1));
    if scroll_row >= max_start && line_count > 0 {
        *follow_tail = true;
        *viewport_offset = 0;
    } else {
        *follow_tail = false;
        *viewport_offset = scroll_row;
    }
}

pub fn render_shell_panel(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let idle_hint = if ctx.state.shell.scrollback.line_count() > 0
        || ctx.state.shell.scrollback.has_pending_line()
    {
        None
    } else if ctx.state.shell.running {
        Some("Starting shell…")
    } else {
        Some("Shell is not running")
    };

    let mut follow_tail = ctx.state.shell.follow_tail;
    let mut viewport_offset = ctx.state.shell.viewport_offset;
    let mut _viewport_rows = 1usize;

    render_pty_scrollback(
        ui,
        ctx.theme,
        PtyScrollbackView {
            scrollback: &ctx.state.shell.scrollback,
            follow_tail,
            viewport_offset,
            spawn_error: ctx.state.shell.spawn_error.as_deref(),
            idle_hint,
        },
        &mut follow_tail,
        &mut viewport_offset,
        &mut _viewport_rows,
    );

    ctx.state.shell.follow_tail = follow_tail;
    ctx.state.shell.viewport_offset = viewport_offset;
    let (cols, rows) = pty_dimensions_from_ui(ui, ctx.theme.font_size);
    ctx.state.viewport.shell_cols = cols;
    ctx.state.viewport.shell_rows = rows;
}

pub fn render_agent_panel(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    if ctx.state.agent_manager.session_count() > 1 {
        if let Some(id) = render_agent_session_tabs(ui, ctx) {
            let _ = (ctx.dispatch)(kiwi_core::events::AppCommand::AgentSetActive(id));
        }
        ui.add_space(4.0);
    }

    let footer = ctx.state.active_agent().restart_hint.clone();
    let idle_hint = agent_idle_hint(ctx.state);
    let spawn_error = ctx.state.active_agent().spawn_error.clone();
    let mut follow_tail = ctx.state.active_agent().follow_tail;
    let mut viewport_offset = ctx.state.active_agent().viewport_offset;
    let mut _viewport_rows = 1usize;

    {
        let scrollback = &ctx.state.active_agent().scrollback;
        render_pty_scrollback(
            ui,
            ctx.theme,
            PtyScrollbackView {
                scrollback,
                follow_tail,
                viewport_offset,
                spawn_error: spawn_error.as_deref(),
                idle_hint: idle_hint.as_deref(),
            },
            &mut follow_tail,
            &mut viewport_offset,
            &mut _viewport_rows,
        );
    }

    {
        let agent = ctx.state.active_agent_mut();
        agent.follow_tail = follow_tail;
        agent.viewport_offset = viewport_offset;
    }

    let (cols, rows) = pty_dimensions_from_ui(ui, ctx.theme.font_size);
    ctx.state.viewport.agent_cols = cols;
    ctx.state.viewport.agent_rows = rows;

    render_pty_footer(ui, ctx.theme, footer.as_deref());
}

fn agent_idle_hint(state: &kiwi_core::state::AppState) -> Option<String> {
    let agent = state.active_agent();
    if agent.scrollback.line_count() > 0 || agent.scrollback.has_pending_line() {
        None
    } else if !agent.spawned {
        Some("Agent spawns when you open this tab (View → Agent or Ctrl+2).".to_string())
    } else if agent.running {
        Some("Starting agent…".to_string())
    } else {
        None
    }
}

fn render_agent_session_tabs(
    ui: &mut Ui,
    ctx: &PanelContext<'_>,
) -> Option<kiwi_core::agent::AgentId> {
    let mut selected = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        for session in ctx.state.agent_manager.sessions() {
            let active = session.id == ctx.state.agent_manager.active_id();
            let label = if session.pty.running {
                format!("{} •", session.label)
            } else {
                session.label.clone()
            };
            let color = if active {
                ctx.theme.role(SemanticRole::Accent)
            } else {
                ctx.theme.role(SemanticRole::Muted)
            };
            if ui
                .add(
                    egui::Label::new(RichText::new(label).color(color).strong())
                        .sense(egui::Sense::click()),
                )
                .clicked()
            {
                selected = Some(session.id);
            }
        }
    });
    selected
}

#[cfg(test)]
mod tests {
    use kiwi_core::shell::ScrollbackBuffer;

    use super::*;

    #[test]
    fn sync_viewport_scroll_at_bottom_sets_follow_tail() {
        let mut follow = false;
        let mut offset = 5;
        sync_viewport_scroll(10, 12, 3, &mut follow, &mut offset);
        assert!(follow);
        assert_eq!(offset, 0);
    }

    #[test]
    fn sync_viewport_scroll_mid_buffer_clears_follow_tail() {
        let mut follow = true;
        let mut offset = 0;
        sync_viewport_scroll(2, 20, 5, &mut follow, &mut offset);
        assert!(!follow);
        assert_eq!(offset, 2);
    }

    #[test]
    fn scrollback_line_count_includes_appended_output() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"line one\nline two\n");
        assert_eq!(buffer.line_count(), 2);
    }
}
