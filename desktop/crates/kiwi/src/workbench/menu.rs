//! Title bar menus (File, Git).

use std::path::PathBuf;

use egui::{RichText, Ui};

use crate::project::RecentProjects;
use crate::theme::menu;

/// Pending actions from the title bar menu.
#[derive(Debug, Default)]
pub struct MenuState {
    /// User chose **Open Folder** — spawn a native picker on the next poll.
    pub open_folder_requested: bool,
    /// User chose a path from **Open Recent**.
    pub open_recent_path: Option<PathBuf>,
    /// User chose **Git → New Issue** — open the compose tab in the editor.
    pub new_issue_requested: bool,
    /// User chose **Git → New Comment** — open the comment modal.
    pub new_comment_requested: bool,
    /// User chose **Git → Manage Labels** — open the labels modal.
    pub manage_labels_requested: bool,
    /// User chose **Git → Manage Milestones** — open the milestones modal.
    pub manage_milestones_requested: bool,
}

/// Renders the **File** menu in the title bar.
pub fn file_menu(ui: &mut Ui, recent: &RecentProjects, menu: &mut MenuState) {
    ui.menu_button(RichText::new("File").size(13.0), |ui| {
        menu::prepare_menu_ui(ui);
        if ui.button("Open Folder…").clicked() {
            menu.open_folder_requested = true;
            ui.close_menu();
        }

        ui.menu_button("Open Recent", |ui| {
            menu::prepare_menu_ui(ui);
            if recent.is_empty() {
                ui.label(RichText::new("No recent folders").weak().size(12.0));
            } else {
                for path in recent.entries() {
                    let label = RecentProjects::menu_label(path);
                    if ui
                        .button(label)
                        .on_hover_text(path.display().to_string())
                        .clicked()
                    {
                        menu.open_recent_path = Some(path.clone());
                        ui.close_menu();
                    }
                }
            }
        });
    });
}

/// Renders the **Git** menu in the title bar.
pub fn git_menu(ui: &mut Ui, menu: &mut MenuState) {
    ui.menu_button(RichText::new("Git").size(13.0), |ui| {
        menu::prepare_menu_ui(ui);
        if ui.button("New Comment…").clicked() {
            menu.new_comment_requested = true;
            ui.close_menu();
        }
        if ui.button("New Issue…").clicked() {
            menu.new_issue_requested = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Manage Labels…").clicked() {
            menu.manage_labels_requested = true;
            ui.close_menu();
        }
        if ui.button("Manage Milestones…").clicked() {
            menu.manage_milestones_requested = true;
            ui.close_menu();
        }
    });
}
