//! Window chrome around the dock area (SPEC-022 / SPEC-019).

mod command_palette;
mod help_modal;
mod menu_bar;
mod status_bar;

pub use command_palette::{
    palette_keyboard_action, palette_open_shortcut_action, render_command_palette,
};
pub use help_modal::{render_about_modal, render_shortcuts_modal};
pub use menu_bar::{render_menu_bar, render_reset_layout_modal};
pub use status_bar::render_status_bar;
