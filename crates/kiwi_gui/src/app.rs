//! Main eframe application shell (SPEC-021).

use crate::bootstrap::GuiBootstrapContext;

/// GUI shell; panels and services land in later milestones.
pub struct KiwiApp {
    context: GuiBootstrapContext,
}

impl KiwiApp {
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext<'_>, context: GuiBootstrapContext) -> Self {
        Self { context }
    }
}

impl eframe::App for KiwiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Kiwi");
            ui.separator();
            ui.label(format!("Repository: {}", self.context.repo_root.display()));
            ui.label(format!("Theme: {}", self.context.theme.name));
            ui.label(format!(
                "Editor: {}",
                self.context
                    .config
                    .editor
                    .configured_command
                    .as_deref()
                    .unwrap_or("(auto)")
            ));
            if !self.context.is_git_repo {
                ui.colored_label(
                    egui::Color32::from_rgb(224, 175, 104),
                    "Not a git repository — git features disabled",
                );
            }
        });

        if ctx.input(|input| input.key_pressed(egui::Key::Q) && input.modifiers.command) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
