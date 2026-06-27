//! Top menu bar — View menu for dock tabs (SPEC-022 / #185).
//!
//! File, Git, Help, and command palette arrive in #187.

use egui::Context;

use crate::dock::{DockShell, KiwiTab};

const MENU_BAR_HEIGHT: f32 = 28.0;

/// User action from the menu bar this frame.
#[derive(Debug, Default)]
pub struct MenuBarAction {
    pub reset_layout_requested: bool,
}

pub fn render_menu_bar(ctx: &Context, dock: &mut DockShell) -> MenuBarAction {
    let mut action = MenuBarAction::default();

    egui::TopBottomPanel::top("menu_bar")
        .min_height(MENU_BAR_HEIGHT)
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {
                    for tab in KiwiTab::all_variants() {
                        let mut open = dock.is_tab_open(*tab);
                        if ui.checkbox(&mut open, tab.title()).changed() {
                            if open {
                                dock.show_tab(*tab);
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
