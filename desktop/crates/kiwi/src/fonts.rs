//! UI and monospace fonts for Kiwi.

use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily, TextStyle};

/// Installs Inter (UI) and JetBrains Mono (code) into the egui context.
///
/// Merges with existing definitions (e.g. Font Awesome from `nest-icon`) rather
/// than replacing them.
pub fn install(ctx: &Context) {
    let mut fonts = ctx.fonts(|locked| locked.lock().fonts.definitions().clone());
    register_ui_fonts(&mut fonts);
    ctx.set_fonts(fonts);
    tune_text_styles(ctx);
    ctx.request_repaint();
}

fn register_ui_fonts(fonts: &mut FontDefinitions) {
    fonts.font_data.insert(
        "inter".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "inter-medium".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Medium.ttf"
        ))),
    );
    fonts.font_data.insert(
        "jetbrains-mono".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/JetBrainsMono-Regular.ttf"
        ))),
    );

    fonts.families.insert(
        FontFamily::Proportional,
        proportional_with_icon_fallback(fonts),
    );

    fonts.families.insert(
        FontFamily::Monospace,
        vec!["jetbrains-mono".into()],
    );
}

fn proportional_with_icon_fallback(fonts: &FontDefinitions) -> Vec<String> {
    let mut family = vec!["inter".into(), "inter-medium".into()];
    if let Some(existing) = fonts.families.get(&FontFamily::Proportional) {
        for key in existing {
            if key.starts_with("fa-") && !family.contains(key) {
                family.push(key.clone());
            }
        }
    }
    family
}

fn tune_text_styles(ctx: &Context) {
    ctx.style_mut(|style| {
        style.text_styles.insert(TextStyle::Body, egui::FontId::new(13.0, FontFamily::Proportional));
        style.text_styles.insert(TextStyle::Button, egui::FontId::new(13.0, FontFamily::Proportional));
        style.text_styles.insert(TextStyle::Heading, egui::FontId::new(18.0, FontFamily::Proportional));
        style.text_styles.insert(TextStyle::Monospace, egui::FontId::new(13.0, FontFamily::Monospace));
        style.text_styles.insert(TextStyle::Small, egui::FontId::new(11.0, FontFamily::Proportional));
    });
}
