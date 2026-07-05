//! Maps Cursor Dark tokens to egui visuals.

use egui::{Stroke, Visuals};

use super::cursor_dark::CursorDarkPalette;

/// Applies Cursor Dark palette to egui visuals.
pub fn adapt_visuals(visuals: &mut Visuals, palette: &CursorDarkPalette) {
    *visuals = Visuals::dark();

    visuals.override_text_color = Some(palette.text_primary);
    visuals.window_fill = palette.background_default;
    visuals.panel_fill = palette.background_panel;
    visuals.faint_bg_color = palette.background_elevated;
    visuals.extreme_bg_color = palette.background_sidebar;
    visuals.window_stroke = Stroke::new(1.0, palette.border_default);
    visuals.popup_shadow = egui::epaint::Shadow::NONE;

    visuals.hyperlink_color = palette.accent_primary;
    visuals.warn_fg_color = palette.warning;
    visuals.error_fg_color = palette.error;

    visuals.widgets.noninteractive.bg_fill = palette.background_panel;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.border_default);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text_secondary);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);

    visuals.widgets.inactive.bg_fill = palette.background_default;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.border_subtle);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text_disabled);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);

    visuals.widgets.hovered.bg_fill = palette.background_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.border_subtle);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.text_primary);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);

    visuals.widgets.active.bg_fill = palette.background_active;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent_active);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.text_primary);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);

    visuals.widgets.open.bg_fill = palette.background_active;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, palette.border_default);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, palette.text_primary);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(4);

    visuals.selection.bg_fill = palette.accent_primary.gamma_multiply(0.28);
    visuals.selection.stroke = Stroke::new(1.0, palette.accent_primary);
}
