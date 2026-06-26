//! Main eframe application shell (SPEC-021 G0: blank window).

/// Empty GUI shell; panels and bootstrap land in G1+.
pub struct KiwiApp;

impl KiwiApp {
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self
    }
}

impl eframe::App for KiwiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {});

        if ctx.input(|input| input.key_pressed(egui::Key::Q) && input.modifiers.command) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
