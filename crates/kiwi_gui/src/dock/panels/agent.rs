//! Agent PTY dock panel (SPEC-010 / SPEC-022).

use egui::{RichText, Ui};
use kiwi_core::theme::SemanticRole;

use super::pty_input::{capture_pty_keyboard_focus, navigation_sync_commands};
use super::scrollback::render_agent_panel;
use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let (focused, clicked) = capture_pty_keyboard_focus(ui, "agent_pty_surface");
    ctx.pty_surface.agent_keyboard_focus = focused;
    render_agent_chrome(ui, ctx);
    render_agent_panel(ui, ctx);
    if clicked {
        for command in navigation_sync_commands(ctx.state, KiwiTab::Agent) {
            let _ = (ctx.dispatch)(command);
        }
    }
}

fn render_agent_chrome(ui: &mut Ui, ctx: &PanelContext<'_>) {
    let agent = ctx.state.active_agent();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        let (status, role) = if agent.running {
            ("running", SemanticRole::AgentSuccess)
        } else if agent.spawn_error.is_some() {
            ("error", SemanticRole::AgentError)
        } else if !agent.spawned {
            ("not spawned", SemanticRole::Muted)
        } else {
            ("stopped", SemanticRole::Muted)
        };
        ui.colored_label(
            ctx.theme.role(role),
            RichText::new(format!("Agent · {status}")).strong(),
        );
        ui.colored_label(
            ctx.theme.role(SemanticRole::Muted),
            ctx.state.agent_manager.status_bar_label(),
        );
        if agent.scrollback.line_count() > 0 {
            ui.colored_label(
                ctx.theme.role(SemanticRole::Muted),
                format!("{} lines", agent.scrollback.line_count()),
            );
        }
        if !ctx.pty_surface.agent_keyboard_focus {
            ui.colored_label(
                ctx.theme.role(SemanticRole::Accent),
                "Click here to type",
            );
        }
    });
    ui.separator();
}
