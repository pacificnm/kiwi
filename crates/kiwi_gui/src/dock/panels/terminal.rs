//! Terminal (shell PTY) dock panel (SPEC-011 / SPEC-022).

use egui::{RichText, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::navigation::{FocusTarget, NavCommand};
use kiwi_core::theme::SemanticRole;

use super::pty_input::capture_pty_keyboard_focus;
use super::scrollback::render_shell_panel;
use crate::dock::context::PanelContext;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let (focused, clicked) = capture_pty_keyboard_focus(ui, "terminal_pty_surface");
    ctx.pty_surface.shell_keyboard_focus = focused;
    render_terminal_chrome(ui, ctx);
    render_shell_panel(ui, ctx);
    if clicked {
        let _ = (ctx.dispatch)(AppCommand::Navigation(NavCommand::SetFocus(
            FocusTarget::Shell,
        )));
    }
}

fn render_terminal_chrome(ui: &mut Ui, ctx: &PanelContext<'_>) {
    let shell = &ctx.state.shell;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        let (status, role) = if shell.running {
            ("running", SemanticRole::AgentSuccess)
        } else if shell.spawn_error.is_some() {
            ("error", SemanticRole::AgentError)
        } else {
            ("stopped", SemanticRole::Muted)
        };
        ui.colored_label(
            ctx.theme.role(role),
            RichText::new(format!("Shell · {status}")).strong(),
        );
        ui.colored_label(
            ctx.theme.role(SemanticRole::Muted),
            format!(
                "{} · {}×{}",
                shell.shell_name, shell.cols, shell.rows
            ),
        );
        if shell.scrollback.line_count() > 0 {
            ui.colored_label(
                ctx.theme.role(SemanticRole::Muted),
                format!("{} lines", shell.scrollback.line_count()),
            );
        }
        if !ctx.pty_surface.shell_keyboard_focus {
            ui.colored_label(
                ctx.theme.role(SemanticRole::Accent),
                "Click here to type",
            );
        }
    });
    ui.separator();
}
