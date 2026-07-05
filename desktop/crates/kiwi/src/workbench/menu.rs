//! Title bar menu (File → Open Folder / Open Recent).

use std::path::PathBuf;

use egui::{RichText, Ui};

use crate::project::RecentProjects;

/// Pending actions from the title bar menu.
#[derive(Debug, Default)]
pub struct MenuState {
    /// User chose **Open Folder** — spawn a native picker on the next poll.
    pub open_folder_requested: bool,
    /// User chose a path from **Open Recent**.
    pub open_recent_path: Option<PathBuf>,
}

/// Renders the **File** menu at the start of the title bar.
pub fn file_menu(ui: &mut Ui, recent: &RecentProjects, menu: &mut MenuState) {
    ui.menu_button(RichText::new("File").size(13.0), |ui| {
        if ui.button("Open Folder…").clicked() {
            menu.open_folder_requested = true;
            ui.close_menu();
        }

        ui.menu_button("Open Recent", |ui| {
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
