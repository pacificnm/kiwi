//! Top menu bar — File, View, Git, Help (SPEC-022 / #185 / #187).

use egui::Context;

use crate::dock::{DockShell, KiwiTab};

const MENU_BAR_HEIGHT: f32 = 28.0;

/// User action from the menu bar this frame.
#[derive(Debug, Default)]
pub struct MenuBarAction {
    pub reset_layout_requested: bool,
    pub quit_requested: bool,
    pub git_refresh_requested: bool,
    pub command_palette_requested: bool,
    pub shortcuts_help_requested: bool,
    pub about_requested: bool,
    /// Dock tabs opened from the View menu this frame (need nav sync).
    pub tabs_opened: Vec<KiwiTab>,
}

pub fn render_menu_bar(ctx: &Context, dock: &mut DockShell) -> MenuBarAction {
    let mut action = MenuBarAction::default();

    egui::TopBottomPanel::top("menu_bar")
        .min_height(MENU_BAR_HEIGHT)
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .button("Command Palette…")
                        .on_hover_text("Ctrl+Shift+P")
                        .clicked()
                    {
                        action.command_palette_requested = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").on_hover_text("Ctrl+Q").clicked() {
                        action.quit_requested = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    for tab in KiwiTab::all_variants() {
                        let mut open = dock.is_tab_open(*tab);
                        if ui.checkbox(&mut open, tab.title()).changed() {
                            if open {
                                dock.show_tab(*tab);
                                action.tabs_opened.push(*tab);
                            } else {
                                dock.close_tab(*tab);
                            }
                        }
                    }
                    ui.separator();
                    if ui.button("Reset Layout…").clicked() {
                        action.reset_layout_requested = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Git", |ui| {
                    if ui.button("Refresh Status").on_hover_text("F5").clicked() {
                        action.git_refresh_requested = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Keyboard Shortcuts").clicked() {
                        action.shortcuts_help_requested = true;
                        ui.close_menu();
                    }
                    if ui.button("About Kiwi").clicked() {
                        action.about_requested = true;
                        ui.close_menu();
                    }
                });
            });
        });

    action
}

pub fn render_reset_layout_modal(ctx: &Context, open: &mut bool, dock: &mut DockShell) {
    if !*open {
        return;
    }

    let mut close = false;
    egui::Window::new("Reset Layout")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Restore the default dock layout? Unsaved panel positions will be lost.");
            ui.horizontal(|ui| {
                if ui.button("Reset").clicked() {
                    dock.reset_layout();
                    close = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        });

    if close {
        *open = false;
    }
}
