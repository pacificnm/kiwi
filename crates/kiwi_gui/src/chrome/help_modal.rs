//! Help dialogs for the menu bar (SPEC-022 / #187).

use egui::{Align2, Context, ScrollArea, Vec2};

const SHORTCUTS_TEXT: &str = "\
Global
  Ctrl+Shift+P / Ctrl+K   Command palette
  Ctrl+Q                    Quit
  Ctrl+Tab / Ctrl+Shift+Tab Next / previous dock tab

View
  Ctrl+Shift+E              Explorer
  Ctrl+`                      Terminal
  Ctrl+Shift+A              Agent
  Ctrl+Shift+F              Search

Git / GitHub
  F5                        Refresh status / issues

Terminal / Agent (when focused)
  Ctrl+Shift+C / Ctrl+Shift+V  Copy / paste
";

pub fn render_shortcuts_modal(ctx: &Context, open: &mut bool) {
    if !*open {
        return;
    }

    let mut close = false;
    egui::Window::new("Keyboard Shortcuts")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .default_width(480.0)
        .show(ctx, |ui| {
            ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                ui.monospace(SHORTCUTS_TEXT);
            });
            ui.add_space(8.0);
            if ui.button("Close").clicked() {
                close = true;
            }
        });

    if close {
        *open = false;
    }
}

pub fn render_about_modal(ctx: &Context, open: &mut bool) {
    if !*open {
        return;
    }

    let mut close = false;
    let version = env!("CARGO_PKG_VERSION");
    egui::Window::new("About Kiwi")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.heading("Kiwi");
            ui.label(format!("Version {version}"));
            ui.add_space(4.0);
            ui.label("Terminal-native AI development workspace.");
            ui.add_space(8.0);
            if ui.button("Close").clicked() {
                close = true;
            }
        });

    if close {
        *open = false;
    }
}
