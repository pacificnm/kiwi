//! Cursor-style menu sizing for Kiwi dropdown and context menus.

use egui::{InnerResponse, Response, Style, Ui};

/// Default width for title-bar dropdowns and context menus.
pub const MENU_WIDTH: f32 = 220.0;

/// Minimum row height for menu items.
pub const MENU_ITEM_HEIGHT: f32 = 24.0;

/// Padding inside menu item buttons.
pub const MENU_BUTTON_PADDING: egui::Vec2 = egui::vec2(12.0, 6.0);

/// Applies global egui spacing used by all menus.
pub fn adapt_spacing(style: &mut Style) {
    style.spacing.menu_width = MENU_WIDTH;
    style.spacing.menu_margin = egui::Margin::symmetric(8, 6);
}

/// Prepares a menu panel with consistent Kiwi sizing.
pub fn prepare_menu_ui(ui: &mut Ui) {
    ui.set_min_width(MENU_WIDTH);
    ui.style_mut().spacing.button_padding = MENU_BUTTON_PADDING;
    ui.style_mut().spacing.interact_size.y = MENU_ITEM_HEIGHT;
}

/// Right-click context menu with global Kiwi menu sizing.
pub fn context_menu(
    response: &Response,
    add_contents: impl FnOnce(&mut Ui),
) -> Option<InnerResponse<()>> {
    response.context_menu(|ui| {
        prepare_menu_ui(ui);
        add_contents(ui);
    })
}
