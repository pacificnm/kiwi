//! Explorer sidebar — project file tree.

use egui::{Button, RichText, Ui};
use nest_core::AppContext;
use nest_file::FileService;

use crate::project::ProjectConfig;
use crate::theme::PALETTE;
use crate::workbench::editor::EditorState;
use crate::workbench::editor_files::{abs_path_for_rel, open_file_tab, spawn_read_file, validate_rel_path};
use crate::workbench::explorer::{ExplorerState, TreeNode};
use crate::workbench::FileLoadPending;

enum ExplorerAction {
    Toggle(String),
    Open(String),
}

/// Renders the project file explorer.
pub fn show(
    ui: &mut Ui,
    explorer: &mut ExplorerState,
    project: &ProjectConfig,
    editor: &mut EditorState,
    file_pending: &mut Option<FileLoadPending>,
    app_ctx: &AppContext,
) {
    ui.label(
        RichText::new(format!("{} — {}", explorer.root_label, project.root.display()))
            .weak()
            .size(11.0),
    );
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        if ui.small_button("Refresh").clicked() {
            if let Ok(files) = app_ctx.service::<FileService>() {
                if let Err(error) = explorer.refresh(&files) {
                    explorer.error = Some(error.to_string());
                }
            }
        }
        if ui.small_button("Collapse all").clicked() {
            explorer.collapse_all();
        }
    });

    if let Some(error) = &explorer.error {
        ui.label(
            RichText::new(error)
                .color(ui.visuals().error_fg_color)
                .size(11.0),
        );
    }

    ui.add_space(4.0);

    let Ok(files) = app_ctx.service::<FileService>() else {
        ui.label(RichText::new("File service unavailable").weak().size(12.0));
        return;
    };

    if let Err(error) = explorer.ensure_root_loaded(&files) {
        explorer.error = Some(error.to_string());
    }

    if let Some(error) = &explorer.tree.error {
        ui.label(
            RichText::new(error)
                .color(ui.visuals().error_fg_color)
                .size(11.0),
        );
    }

    let mut actions = Vec::new();
    render_node(
        ui,
        &explorer.tree,
        0,
        explorer.selected.as_deref(),
        &mut actions,
    );

    for action in actions {
        match action {
            ExplorerAction::Toggle(rel_path) => {
                if let Err(error) = explorer.toggle_dir(&rel_path, &files) {
                    explorer.error = Some(error.to_string());
                }
            }
            ExplorerAction::Open(rel_path) => {
                if validate_rel_path(&rel_path).is_ok() {
                    explorer.select_file(rel_path.clone());
                    let abs_path = abs_path_for_rel(&project.root, &rel_path);
                    if let Some(tab_index) = open_file_tab(editor, rel_path.clone(), abs_path) {
                        *file_pending =
                            Some(spawn_read_file(files.clone(), rel_path, tab_index));
                    }
                }
            }
        }
    }
}

fn render_node(
    ui: &mut Ui,
    node: &TreeNode,
    depth: usize,
    selected: Option<&str>,
    actions: &mut Vec<ExplorerAction>,
) {
    if node.rel_path != "." {
        let indent = depth as f32 * 14.0;
        let is_selected = selected == Some(node.rel_path.as_str());
        let label = if node.is_dir {
            let chevron = if node.expanded { "▾" } else { "▸" };
            format!("{chevron} {}", node.name)
        } else {
            format!("  {}", node.name)
        };

        ui.horizontal(|ui| {
            ui.add_space(indent);
            let response = ui.add(
                Button::new(RichText::new(label).size(13.0).monospace())
                    .frame(false)
                    .fill(if is_selected {
                        PALETTE.background_panel
                    } else {
                        egui::Color32::TRANSPARENT
                    }),
            );

            if response.clicked() {
                if node.is_dir {
                    actions.push(ExplorerAction::Toggle(node.rel_path.clone()));
                } else {
                    actions.push(ExplorerAction::Open(node.rel_path.clone()));
                }
            }
        });

        if let Some(error) = &node.error {
            ui.label(
                RichText::new(error)
                    .color(ui.visuals().error_fg_color)
                    .size(11.0),
            );
        }
    }

    if node.is_dir && (node.rel_path == "." || node.expanded) {
        let child_depth = if node.rel_path == "." { 0 } else { depth + 1 };
        for child in &node.children {
            render_node(ui, child, child_depth, selected, actions);
        }
    }
}
