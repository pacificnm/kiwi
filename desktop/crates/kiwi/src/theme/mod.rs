//! Cursor Dark theme for Kiwi.

mod adapter;
mod cursor_dark;
pub mod module;

use egui::Context;

pub use cursor_dark::PALETTE;
pub use module::KiwiThemeModule;

/// Applies Cursor Dark egui visuals.
pub fn apply(ctx: &Context) {
    ctx.style_mut(|style| adapter::adapt_visuals(&mut style.visuals, &PALETTE));
}
