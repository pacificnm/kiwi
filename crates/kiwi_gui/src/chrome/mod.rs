//! Window chrome around the dock area (SPEC-022 / SPEC-019).

mod menu_bar;
mod status_bar;

pub use menu_bar::{render_menu_bar, render_reset_layout_modal};
pub use status_bar::render_status_bar;
