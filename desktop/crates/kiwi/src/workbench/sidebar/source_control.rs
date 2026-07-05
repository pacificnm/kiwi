//! Source control sidebar — git status, stage, and commit.

use egui::{Align, Label, Layout, RichText, ScrollArea, Sense, TextEdit, Ui};

use nest_gui::{ActionButton, ButtonSize};
use nest_icon::{Icon, icons};

use crate::project::ProjectConfig;
use crate::theme::PALETTE;
use crate::workbench::sidebar::panel_width;
use crate::workbench::editor::EditorState;
use crate::workbench::editor_files::{
    abs_path_for_rel, open_diff_tab, spawn_git_diff_load, validate_rel_path,
};
use crate::workbench::source_control::{ChangeKind, DiffSide, GitChange, GitStatus, SourceControlState};
use crate::workbench::FileLoadPending;

const CONTROL_HEIGHT: f32 = 28.0;
const STATUS_COLUMN_WIDTH: f32 = 18.0;
const STAGE_BUTTON_WIDTH: f32 = 28.0;
const ROW_ITEM_GAP: f32 = 4.0;

/// Renders the git / source control panel.
pub fn show(
    ui: &mut Ui,
    source: &mut SourceControlState,
    project: &ProjectConfig,
    editor: &mut EditorState,
    file_pending: &mut Option<FileLoadPending>,
) {
    toolbar(ui, source, project);
    ui.add_space(6.0);

    if source.not_repo {
        ui.label(
            RichText::new("This folder is not a Git repository.")
                .weak()
                .size(12.0),
        );
        return;
    }

    if source.loading && source.status.is_none() {
        ui.label(RichText::new("Loading changes…").weak().size(12.0));
        return;
    }

    if let Some(error) = &source.error {
        ui.label(
            RichText::new(error)
                .color(ui.visuals().error_fg_color)
                .size(11.0),
        );
        ui.add_space(4.0);
    }

    commit_section(ui, source, project);

    let scroll_height = ui.available_height().max(0.0);
    ScrollArea::vertical()
        .id_salt((
            "kiwi-source-control-changes",
            project.root.display().to_string(),
        ))
        .auto_shrink([false; 2])
        .max_height(scroll_height)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            change_lists(ui, source, project, editor, file_pending);
        });
}

fn toolbar(ui: &mut Ui, source: &mut SourceControlState, project: &ProjectConfig) {
    full_width_row(ui, CONTROL_HEIGHT, |ui, _width| {
        if let Some(status) = &source.status {
            ui.label(
                RichText::new(branch_label(status))
                    .strong()
                    .size(12.0),
            );
        } else if source.loading {
            ui.label(RichText::new("Refreshing…").weak().size(12.0));
        } else {
            ui.label(RichText::new("Git").weak().size(12.0));
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.set_width(ui.available_width());
            if ui
                .add(
                    ActionButton::new(Icon::ARROW_ROTATE_RIGHT, "Refresh")
                        .size(ButtonSize::Small)
                        .enabled(!source.busy())
                        .tooltip("Refresh git status"),
                )
                .clicked()
            {
                source.request_refresh(&project.root);
            }
        });
    });
}

fn commit_section(ui: &mut Ui, source: &mut SourceControlState, project: &ProjectConfig) {
    let width = panel_width(ui);

    ui.add_space(8.0);
    ui.label(RichText::new("Message").weak().size(11.0));
    ui.add_space(2.0);
    ui.add(
        TextEdit::multiline(&mut source.commit_message)
            .hint_text("Commit message")
            .desired_width(width)
            .desired_rows(2)
            .frame(true),
    );
    ui.add_space(6.0);

    let has_staged = source
        .status
        .as_ref()
        .is_some_and(|status| status.staged().next().is_some());
    let can_commit = has_staged
        && !source.commit_message.trim().is_empty()
        && !source.busy();
    let can_push = source
        .status
        .as_ref()
        .is_some_and(|status| status.branch != "HEAD" && status.ahead > 0 && !source.busy());

    full_width_row(ui, CONTROL_HEIGHT, |ui, _width| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if ui
            .add(
                ActionButton::new(Icon::CHECK, "Stage All")
                    .size(ButtonSize::Small)
                    .enabled(!source.busy())
                    .tooltip("Stage all changes"),
            )
            .clicked()
        {
            source.stage_all(&project.root);
        }

        if ui
            .add(
                ActionButton::new(Icon::solid(icons::solid::FLOPPY_DISK), "Commit")
                    .size(ButtonSize::Small)
                    .enabled(can_commit)
                    .fill(PALETTE.accent_primary)
                    .text_color(egui::Color32::WHITE)
                    .tooltip("Commit staged changes"),
            )
            .clicked()
        {
            source.commit(&project.root);
        }

        if ui
            .add(
                ActionButton::new(Icon::solid(icons::solid::UPLOAD), "Push")
                    .size(ButtonSize::Small)
                    .enabled(can_push)
                    .tooltip(push_tooltip(source.status.as_ref()))
                    .fill(if can_push {
                        PALETTE.accent_primary
                    } else {
                        egui::Color32::TRANSPARENT
                    })
                    .text_color(if can_push {
                        egui::Color32::WHITE
                    } else {
                        ui.visuals().text_color()
                    }),
            )
            .clicked()
        {
            source.push(&project.root);
        }
    });
}

fn change_lists(
    ui: &mut Ui,
    source: &mut SourceControlState,
    project: &ProjectConfig,
    editor: &mut EditorState,
    file_pending: &mut Option<FileLoadPending>,
) {
    let staged: Vec<GitChange> = source
        .status
        .as_ref()
        .map(|status| status.staged().cloned().collect())
        .unwrap_or_default();
    let unstaged: Vec<GitChange> = source
        .status
        .as_ref()
        .map(|status| status.unstaged().cloned().collect())
        .unwrap_or_default();

    if source.status.is_none() && !source.loading {
        ui.label(RichText::new("No changes loaded.").weak().size(12.0));
        return;
    }

    if staged.is_empty() && unstaged.is_empty() && source.status.is_some() {
        ui.label(
            RichText::new("No changes. Working tree clean.")
                .weak()
                .size(12.0),
        );
        return;
    }

    let mut open_actions = Vec::new();

    if !staged.is_empty() {
        section_heading(ui, &format!("Staged Changes ({})", staged.len()));
        for change in &staged {
            change_row(ui, source, project, change, true, &mut open_actions);
        }
        ui.add_space(6.0);
    }

    if !unstaged.is_empty() {
        section_heading(ui, &format!("Changes ({})", unstaged.len()));
        for change in &unstaged {
            change_row(ui, source, project, change, false, &mut open_actions);
        }
    }

    for action in open_actions {
        if file_pending.is_some() {
            break;
        }
        if validate_rel_path(&action.rel_path).is_err() {
            continue;
        }
        let abs_path = abs_path_for_rel(&project.root, &action.rel_path);
        if let Some(tab_index) = open_diff_tab(
            editor,
            action.rel_path.clone(),
            abs_path,
            action.staged,
        ) {
            *file_pending = Some(spawn_git_diff_load(
                project.root.clone(),
                action.rel_path,
                action.side,
                tab_index,
            ));
        }
    }
}

fn section_heading(ui: &mut Ui, title: &str) {
    ui.label(RichText::new(title).strong().size(11.0));
    ui.separator();
    ui.add_space(2.0);
}

struct OpenDiffAction {
    rel_path: String,
    side: DiffSide,
    staged: bool,
}

fn change_row(
    ui: &mut Ui,
    source: &mut SourceControlState,
    project: &ProjectConfig,
    change: &GitChange,
    staged_section: bool,
    open_actions: &mut Vec<OpenDiffAction>,
) {
    let row_height = ui.spacing().interact_size.y;

    full_width_row(ui, row_height, |ui, width| {
        ui.spacing_mut().item_spacing.x = ROW_ITEM_GAP;

        left_cell(ui, egui::vec2(STATUS_COLUMN_WIDTH, row_height), |ui| {
            ui.add(
                Label::new(
                    RichText::new(change.kind.label())
                        .monospace()
                        .size(11.0)
                        .color(status_color(change.kind)),
                )
                .halign(Align::LEFT),
            );
        });

        let path_width =
            (width - STATUS_COLUMN_WIDTH - STAGE_BUTTON_WIDTH - ROW_ITEM_GAP * 2.0).max(40.0);
        left_cell(ui, egui::vec2(path_width, row_height), |ui| {
            let response = ui.add(
                Label::new(
                    RichText::new(&change.path)
                        .size(12.0)
                        .color(PALETTE.text_primary),
                )
                .truncate()
                .halign(Align::LEFT)
                .sense(Sense::click()),
            );
            if response.clicked() {
                open_actions.push(OpenDiffAction {
                    rel_path: change.path.clone(),
                    side: diff_side(change, staged_section),
                    staged: staged_section,
                });
            }
            response.on_hover_text("Open diff in editor");
        });

        left_cell(ui, egui::vec2(STAGE_BUTTON_WIDTH, row_height), |ui| {
            if source.busy() {
                return;
            }
            if staged_section {
                if ui.small_button("−").on_hover_text("Unstage").clicked() {
                    source.unstage_path(&project.root, &change.path);
                }
            } else if ui.small_button("+").on_hover_text("Stage").clicked() {
                source.stage_path(&project.root, &change.path);
            }
        });
    });
}

/// Left-aligned cell. Never use [`Ui::add_sized`] for list rows — it applies
/// [`Layout::centered_and_justified`] and centers the widget in the cell.
fn left_cell<R>(ui: &mut Ui, size: egui::Vec2, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        ui.set_width(size.x);
        add_contents(ui)
    })
    .inner
}

fn full_width_row<R>(ui: &mut Ui, height: f32, add_contents: impl FnOnce(&mut Ui, f32) -> R) -> R {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, height),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_width(width);
            add_contents(ui, width)
        },
    )
    .inner
}

fn diff_side(change: &GitChange, staged_section: bool) -> DiffSide {
    if change.kind == ChangeKind::Untracked {
        DiffSide::Untracked
    } else if staged_section {
        DiffSide::Staged
    } else {
        DiffSide::Unstaged
    }
}

fn status_color(kind: ChangeKind) -> egui::Color32 {
    match kind {
        ChangeKind::Modified => PALETTE.warning,
        ChangeKind::Added | ChangeKind::Copied => PALETTE.success,
        ChangeKind::Deleted => PALETTE.error,
        ChangeKind::Renamed | ChangeKind::Untracked | ChangeKind::Other => PALETTE.text_secondary,
    }
}

fn branch_glyph() -> &'static str {
    "⎇"
}

fn branch_label(status: &GitStatus) -> String {
    let mut label = format!("{} {}", branch_glyph(), status.branch);
    if status.ahead > 0 {
        label.push_str(&format!(" ↑{}", status.ahead));
    }
    if status.behind > 0 {
        label.push_str(&format!(" ↓{}", status.behind));
    }
    label
}

fn push_tooltip(status: Option<&GitStatus>) -> &'static str {
    match status {
        Some(status) if status.branch == "HEAD" => "Cannot push from detached HEAD",
        Some(status) if status.ahead == 0 => "No commits to push",
        Some(status) if !status.has_upstream => {
            "Push and set upstream on origin"
        }
        _ => "Push to remote",
    }
}

#[cfg(test)]
mod tests {
    use super::branch_glyph;

    #[test]
    fn branch_glyph_is_non_empty() {
        assert!(!branch_glyph().is_empty());
    }
}
