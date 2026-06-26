//! Main eframe application shell (SPEC-021).

use kiwi_core::theme::SemanticRole;

use crate::bootstrap::GuiBootstrapContext;
use crate::theme::GuiTheme;

/// GUI shell; panels and services land in later milestones.
pub struct KiwiApp {
    context: GuiBootstrapContext,
    gui_theme: GuiTheme,
}

impl KiwiApp {
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext<'_>, context: GuiBootstrapContext) -> Self {
        let gui_theme = GuiTheme::from_palette(&context.theme, &context.config.gui);
        Self { context, gui_theme }
    }
}

impl eframe::App for KiwiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.gui_theme.apply_to_context(ctx);

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
                    self.gui_theme.role(SemanticRole::AgentWarning),
                    "Not a git repository — git features disabled",
                );
            }

            ui.separator();
            ui.label("Theme preview:");
            role_swatch(
                ui,
                &self.gui_theme,
                SemanticRole::GitModified,
                "git modified",
            );
            role_swatch(ui, &self.gui_theme, SemanticRole::IssueOpen, "issue open");
            role_swatch(ui, &self.gui_theme, SemanticRole::AgentError, "agent error");
        });

        if ctx.input(|input| input.key_pressed(egui::Key::Q) && input.modifiers.command) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn role_swatch(ui: &mut egui::Ui, theme: &GuiTheme, role: SemanticRole, label: &str) {
    let color = theme.role(role);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 2.0, color);
        ui.colored_label(color, label);
    });
}
