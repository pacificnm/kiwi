//! Top menu bar — File, View, Git, Help (SPEC-022 / #185 / #187).

use egui::{Context, CursorIcon, Response, RichText, Ui};

use crate::dock::context_menu::{menu_action, menu_checkbox, render_menu_shell};
use crate::dock::{DockShell, KiwiTab};
use crate::theme::GuiTheme;

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
    pub settings_requested: bool,
    pub plugins_requested: bool,
    /// Dock tabs opened from the View menu this frame (need nav sync).
    pub tabs_opened: Vec<KiwiTab>,
}

pub fn render_menu_bar(ctx: &Context, theme: &GuiTheme, dock: &mut DockShell) -> MenuBarAction {
    let mut action = MenuBarAction::default();

    egui::TopBottomPanel::top("menu_bar")
        .min_height(MENU_BAR_HEIGHT)
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    render_menu_shell(ui, theme, "File", |ui| {
                        if menu_action_with_tip(ui, theme, "⌘", "Command Palette…", "Ctrl+Shift+P")
                        {
                            action.command_palette_requested = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_action(ui, theme, "◈", "Settings") {
                            action.settings_requested = true;
                            ui.close_menu();
                        }
                        if menu_action(ui, theme, "⧉", "Plugins") {
                            action.plugins_requested = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_action_with_tip(ui, theme, "✕", "Quit", "Ctrl+Q") {
                            action.quit_requested = true;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("View", |ui| {
                    render_menu_shell(ui, theme, "View", |ui| {
                        for tab in KiwiTab::all_variants().iter().filter(|t| !t.is_placeholder())
                        {
                            let mut open = dock.is_tab_open(*tab);
                            if tab.is_closeable() {
                                let response = menu_checkbox(
                                    ui,
                                    theme,
                                    tab_menu_icon(*tab),
                                    tab.title(),
                                    &mut open,
                                    true,
                                );
                                if response.changed() {
                                    if open {
                                        dock.show_tab(*tab);
                                        action.tabs_opened.push(*tab);
                                    } else {
                                        dock.close_tab(*tab);
                                    }
                                }
                            } else {
                                let mut pinned = true;
                                menu_checkbox(
                                    ui,
                                    theme,
                                    tab_menu_icon(*tab),
                                    tab.title(),
                                    &mut pinned,
                                    false,
                                )
                                .on_hover_text("This tab cannot be closed");
                            }
                        }
                        ui.separator();
                        if menu_action(ui, theme, "↺", "Reset Layout…") {
                            action.reset_layout_requested = true;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Git", |ui| {
                    render_menu_shell(ui, theme, "Git", |ui| {
                        if menu_action_with_tip(ui, theme, "↻", "Refresh Status", "F5") {
                            action.git_refresh_requested = true;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Help", |ui| {
                    render_menu_shell(ui, theme, "Help", |ui| {
                        if menu_action(ui, theme, "?", "Keyboard Shortcuts") {
                            action.shortcuts_help_requested = true;
                            ui.close_menu();
                        }
                        if menu_action(ui, theme, "◎", "About Kiwi") {
                            action.about_requested = true;
                            ui.close_menu();
                        }
                    });
                });
            });
        });

    action
}

fn menu_action_with_tip(
    ui: &mut Ui,
    theme: &GuiTheme,
    icon: &str,
    label: &str,
    tip: &str,
) -> bool {
    let row = menu_action_row(ui, theme, icon, label);
    row.on_hover_text(tip).clicked()
}

fn menu_action_row(ui: &mut Ui, theme: &GuiTheme, icon: &str, label: &str) -> Response {
    use kiwi_core::theme::SemanticRole;

    const MENU_ROW_HEIGHT: f32 = 22.0;
    const MENU_MIN_WIDTH: f32 = 280.0;

    ui.horizontal(|ui| {
        ui.set_min_height(MENU_ROW_HEIGHT);
        ui.set_min_width((MENU_MIN_WIDTH - 20.0).max(0.0));
        ui.label(
            RichText::new(icon)
                .monospace()
                .color(theme.role(SemanticRole::Accent)),
        );
        ui.add(
            egui::Label::new(RichText::new(label).color(theme.role(SemanticRole::Fg)))
                .truncate()
                .sense(egui::Sense::click()),
        )
        .on_hover_cursor(CursorIcon::PointingHand)
    })
    .inner
}

#[must_use]
const fn tab_menu_icon(tab: KiwiTab) -> &'static str {
    match tab {
        KiwiTab::Explorer => "▤",
        KiwiTab::GitStatus => "⑂",
        KiwiTab::GitDiff => "±",
        KiwiTab::GitLog => "⎇",
        KiwiTab::GitHubIssues => "◆",
        KiwiTab::Issues => "◎",
        KiwiTab::GitHubPrs => "⑃",
        KiwiTab::Preview => "▣",
        KiwiTab::Search => "⌕",
        KiwiTab::Terminal => "▭",
        KiwiTab::Agent => "▷",
        KiwiTab::Config => "◈",
        KiwiTab::Logs => "▤",
        KiwiTab::Plugins => "⧉",
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_menu_icons_are_unique_for_non_placeholder_tabs() {
        let icons: Vec<_> = KiwiTab::all_variants()
            .iter()
            .filter(|tab| !tab.is_placeholder())
            .map(|tab| tab_menu_icon(*tab))
            .collect();
        let mut deduped = icons.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(deduped.len(), icons.len());
    }
}
