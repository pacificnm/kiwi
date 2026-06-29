//! Transient toast overlay (mirrors TUI notification toasts).

use egui::{Align, Frame, RichText};
use kiwi_core::state::AppState;
use kiwi_core::theme::SemanticRole;

use crate::theme::GuiTheme;

/// Render the current toast message above the status bar, if any.
pub fn render_toast(ctx: &egui::Context, theme: &GuiTheme, state: &AppState) {
    let Some(message) = state.notifications.toast.message.as_deref() else {
        return;
    };

    egui::TopBottomPanel::bottom("toast_overlay")
        .exact_height(22.0)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(theme.role(SemanticRole::Bg))
                .stroke(egui::Stroke::new(1.0, theme.role(SemanticRole::Accent)))
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                        ui.label(
                            RichText::new(message)
                                .color(theme.role(SemanticRole::Fg))
                                .strong(),
                        );
                    });
                });
        });
}
