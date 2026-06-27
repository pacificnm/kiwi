//! Main eframe application shell (SPEC-021).

use kiwi_core::theme::SemanticRole;

use crate::chrome::render_status_bar;
use crate::runtime::GuiRuntime;
use crate::theme::GuiTheme;

/// GUI shell; dock panels land in #184.
pub struct KiwiApp {
    runtime: GuiRuntime,
    gui_theme: GuiTheme,
}

impl KiwiApp {
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext<'_>, runtime: GuiRuntime) -> Self {
        let gui_theme = GuiTheme::from_palette(&runtime.state.theme, &runtime.state.config.gui);
        Self { runtime, gui_theme }
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
        render_status_bar(ctx, &self.gui_theme, &self.runtime.state);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Kiwi");
            ui.separator();
            ui.label(format!(
                "Repository: {}",
                self.runtime.state.repo_root.display()
            ));
            ui.label(format!("Theme: {}", self.runtime.state.theme.name));
            ui.label(format!(
                "Editor: {}",
                self.runtime
                    .state
                    .config
                    .editor
                    .configured_command
                    .as_deref()
                    .unwrap_or("(auto)")
            ));
            if !self.runtime.state.workspace_meta.is_git_repo {
                ui.colored_label(
                    self.gui_theme.role(SemanticRole::AgentWarning),
                    "Not a git repository — git features disabled",
                );
            }

            ui.separator();
            ui.label("Panels and dock layout arrive in a later milestone.");
        });

        if ctx.input(|input| input.key_pressed(egui::Key::Q) && input.modifiers.command) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if event_count > 0 || self.runtime.state.dirty {
            ctx.request_repaint();
        }
        self.runtime.state.mark_clean();
    }
}
