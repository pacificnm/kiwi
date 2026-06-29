//! GitHub label and milestone picker overlays for the GUI.

use egui::{Align2, Context, Key, RichText, ScrollArea};
use kiwi_core::events::AppCommand;
use kiwi_core::state::AppState;
use kiwi_core::theme::SemanticRole;

use crate::theme::GuiTheme;

/// Keyboard handling while a GitHub picker overlay or create modal is open.
#[must_use]
pub fn github_picker_keyboard_action(ctx: &Context, state: &AppState) -> Option<AppCommand> {
    if state.github.issue_create_modal.open {
        return ctx.input_mut(|input| {
            if input.key_pressed(Key::Escape) {
                return Some(AppCommand::GitHubIssueCreateCancel);
            }
            if input.key_pressed(Key::Enter) && input.modifiers.command {
                return Some(AppCommand::GitHubIssueCreateSubmit);
            }
            None
        });
    }

    if state.github.label_picker.is_some() {
        return ctx.input_mut(|input| {
            if input.key_pressed(Key::Escape) {
                return Some(AppCommand::GitHubLabelPickerCancel);
            }
            if input.key_pressed(Key::Enter) {
                return Some(AppCommand::GitHubLabelPickerApply);
            }
            if input.key_pressed(Key::Space) {
                return Some(AppCommand::GitHubLabelPickerToggle);
            }
            if input.key_pressed(Key::ArrowUp) {
                return Some(AppCommand::GitHubLabelPickerMove(-1));
            }
            if input.key_pressed(Key::ArrowDown) {
                return Some(AppCommand::GitHubLabelPickerMove(1));
            }
            None
        });
    }

    if state.github.milestone_picker.is_some() {
        return ctx.input_mut(|input| {
            if input.key_pressed(Key::Escape) {
                return Some(AppCommand::GitHubMilestonePickerCancel);
            }
            if input.key_pressed(Key::Enter) {
                return Some(AppCommand::GitHubMilestonePickerApply);
            }
            if input.key_pressed(Key::ArrowUp) {
                return Some(AppCommand::GitHubMilestonePickerMove(-1));
            }
            if input.key_pressed(Key::ArrowDown) {
                return Some(AppCommand::GitHubMilestonePickerMove(1));
            }
            None
        });
    }

    None
}

/// Render label/milestone picker modals and issue create modal.
pub fn render_github_picker_overlays(
    ctx: &Context,
    theme: &GuiTheme,
    state: &mut AppState,
) -> Option<AppCommand> {
    if let Some(command) = render_issue_create_modal(ctx, theme, state) {
        return Some(command);
    }
    if let Some(command) = render_label_picker_overlay(ctx, theme, state) {
        return Some(command);
    }
    render_milestone_picker_overlay(ctx, theme, state)
}

fn render_issue_create_modal(
    ctx: &Context,
    theme: &GuiTheme,
    state: &mut AppState,
) -> Option<AppCommand> {
    if !state.github.issue_create_modal.open {
        return None;
    }

    let mut clicked_create = false;
    let mut clicked_cancel = false;

    egui::Window::new("New Issue")
        .collapsible(false)
        .resizable(true)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(480.0)
        .show(ctx, |ui| {
            let modal = &mut state.github.issue_create_modal;

            if modal.submitting {
                ui.label(
                    RichText::new("Creating issue…")
                        .color(theme.role(SemanticRole::Muted)),
                );
                return;
            }

            ui.label(
                RichText::new("Title")
                    .strong()
                    .color(theme.role(SemanticRole::Fg)),
            );
            ui.add(
                egui::TextEdit::singleline(&mut modal.title)
                    .desired_width(f32::INFINITY)
                    .hint_text("Issue title"),
            );

            ui.add_space(6.0);
            ui.label(
                RichText::new("Body")
                    .strong()
                    .color(theme.role(SemanticRole::Fg)),
            );
            ui.add(
                egui::TextEdit::multiline(&mut modal.body)
                    .desired_width(f32::INFINITY)
                    .desired_rows(8)
                    .hint_text("Optional description"),
            );

            if let Some(error) = &modal.error {
                ui.add_space(6.0);
                ui.label(RichText::new(error).color(theme.role(SemanticRole::AgentError)));
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!modal.title.trim().is_empty(), egui::Button::new("Create"))
                    .clicked()
                {
                    clicked_create = true;
                }
                if ui.button("Cancel").clicked() {
                    clicked_cancel = true;
                }
            });
            ui.label(
                RichText::new("Ctrl+Enter create · Esc cancel")
                    .small()
                    .color(theme.role(SemanticRole::Muted)),
            );
        });

    if clicked_cancel {
        return Some(AppCommand::GitHubIssueCreateCancel);
    }
    if clicked_create {
        return Some(AppCommand::GitHubIssueCreateSubmit);
    }

    None
}

fn render_label_picker_overlay(
    ctx: &Context,
    theme: &GuiTheme,
    state: &mut AppState,
) -> Option<AppCommand> {
    let Some(picker) = state.github.label_picker.as_ref() else {
        return None;
    };

    let title = if picker.applying {
        format!("Adding labels to #{}", picker.issue_number)
    } else if picker.loading {
        format!("Loading labels for #{}", picker.issue_number)
    } else {
        format!("Add labels to #{}", picker.issue_number)
    };

    let mut clicked_apply = false;
    let mut clicked_cancel = false;
    let mut clicked_toggle: Option<usize> = None;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(420.0)
        .show(ctx, |ui| {
            let picker = state.github.label_picker.as_ref().expect("picker");

            if picker.loading {
                ui.label(
                    RichText::new("Loading repository labels…")
                        .color(theme.role(SemanticRole::Muted)),
                );
                return;
            }

            if let Some(error) = &picker.error {
                ui.label(RichText::new(error).color(theme.role(SemanticRole::AgentError)));
                if ui.button("Close").clicked() {
                    clicked_cancel = true;
                }
                return;
            }

            if picker.labels.is_empty() {
                ui.label(
                    RichText::new("No repository labels found.")
                        .color(theme.role(SemanticRole::Muted)),
                );
                if ui.button("Close").clicked() {
                    clicked_cancel = true;
                }
                return;
            }

            ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                for (index, label) in picker.labels.iter().enumerate() {
                    let selected = picker.selected.get(index).copied().unwrap_or(false);
                    let marker = if selected { "[x]" } else { "[ ]" };
                    let mut text = format!("{marker} {}", label.name);
                    if !label.description.is_empty() {
                        text.push_str(" — ");
                        text.push_str(&label.description);
                    }
                    let response = ui.selectable_label(index == picker.cursor, text);
                    if response.clicked() {
                        clicked_toggle = Some(index);
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    clicked_apply = true;
                }
                if ui.button("Cancel").clicked() {
                    clicked_cancel = true;
                }
            });
            ui.label(
                RichText::new("Space toggle · Enter apply · Esc cancel")
                    .small()
                    .color(theme.role(SemanticRole::Muted)),
            );
        });

    if clicked_cancel {
        return Some(AppCommand::GitHubLabelPickerCancel);
    }
    if let Some(index) = clicked_toggle {
        if let Some(picker) = state.github.label_picker.as_mut() {
            picker.cursor = index;
            picker.toggle_cursor();
            state.dirty = true;
        }
        return None;
    }
    if clicked_apply {
        return Some(AppCommand::GitHubLabelPickerApply);
    }

    None
}

fn render_milestone_picker_overlay(
    ctx: &Context,
    theme: &GuiTheme,
    state: &mut AppState,
) -> Option<AppCommand> {
    let Some(picker) = state.github.milestone_picker.as_ref() else {
        return None;
    };

    let title = if picker.applying {
        format!("Assigning milestone to #{}", picker.issue_number)
    } else if picker.loading {
        format!("Loading milestones for #{}", picker.issue_number)
    } else {
        format!("Assign milestone to #{}", picker.issue_number)
    };

    let mut clicked_apply = false;
    let mut clicked_cancel = false;
    let mut clicked_row: Option<usize> = None;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(420.0)
        .show(ctx, |ui| {
            let picker = state.github.milestone_picker.as_ref().expect("picker");

            if picker.loading {
                ui.label(
                    RichText::new("Loading repository milestones…")
                        .color(theme.role(SemanticRole::Muted)),
                );
                return;
            }

            if let Some(error) = &picker.error {
                ui.label(RichText::new(error).color(theme.role(SemanticRole::AgentError)));
                if ui.button("Close").clicked() {
                    clicked_cancel = true;
                }
                return;
            }

            if picker.milestones.is_empty() {
                ui.label(
                    RichText::new("No open milestones found.")
                        .color(theme.role(SemanticRole::Muted)),
                );
                if ui.button("Close").clicked() {
                    clicked_cancel = true;
                }
                return;
            }

            ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                for (index, milestone) in picker.milestones.iter().enumerate() {
                    let mut text = format!("#{} · {}", milestone.number, milestone.title);
                    if !milestone.description.is_empty() {
                        text.push_str(" — ");
                        text.push_str(&milestone.description);
                    }
                    let response = ui.selectable_label(index == picker.cursor, text);
                    if response.clicked() {
                        clicked_row = Some(index);
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Assign").clicked() {
                    clicked_apply = true;
                }
                if ui.button("Cancel").clicked() {
                    clicked_cancel = true;
                }
            });
            ui.label(
                RichText::new("Enter assign · Esc cancel")
                    .small()
                    .color(theme.role(SemanticRole::Muted)),
            );
        });

    if clicked_cancel {
        return Some(AppCommand::GitHubMilestonePickerCancel);
    }
    if let Some(index) = clicked_row {
        if let Some(picker) = state.github.milestone_picker.as_mut() {
            picker.cursor = index;
            state.dirty = true;
        }
        return Some(AppCommand::GitHubMilestonePickerApply);
    }
    if clicked_apply {
        return Some(AppCommand::GitHubMilestonePickerApply);
    }

    None
}
