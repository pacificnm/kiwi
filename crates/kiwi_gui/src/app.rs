//! Main eframe application shell (SPEC-021 / SPEC-022).

use crate::chrome::render_status_bar;
use crate::dock::{DockShell, PanelContext};
use crate::runtime::GuiRuntime;
use crate::theme::GuiTheme;

/// GUI shell with egui_dock tab panels.
pub struct KiwiApp {
    runtime: GuiRuntime,
    gui_theme: GuiTheme,
    dock: DockShell,
}

impl KiwiApp {
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext<'_>, runtime: GuiRuntime) -> Self {
        let gui_theme = GuiTheme::from_palette(&runtime.state.theme, &runtime.state.config.gui);
        Self {
            runtime,
            gui_theme,
            dock: DockShell::new(),
        }
    }
}

impl eframe::App for KiwiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (should_quit, event_count) = self.runtime.process_pending_events();
        if should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.gui_theme.apply_to_context(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.dock.render(
                ui,
                PanelContext {
                    state: &self.runtime.state,
                    theme: &self.gui_theme,
                },
            );
        });

        render_status_bar(ctx, &self.gui_theme, &self.runtime.state);

        if ctx.input(|input| input.key_pressed(egui::Key::Q) && input.modifiers.command) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if event_count > 0 || self.runtime.state.dirty {
            ctx.request_repaint();
        }
        self.runtime.state.mark_clean();
    }
}
