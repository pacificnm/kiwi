//! Modal for viewing, creating, editing, and deleting repository labels.

use egui::{Color32, RichText, ScrollArea, TextEdit, Ui};

use super::github_metadata::GitHubLabel;
use super::modal_frame::{self, MODAL_WIDTH};

/// Labels modal mode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum LabelsMode {
    #[default]
    List,
    Create,
    Edit {
        original_name: String,
    },
    DeleteConfirm {
        name: String,
    },
}

/// State for the repository labels manager modal.
#[derive(Debug, Default)]
pub struct LabelsModalState {
    pub open: bool,
    pub loading: bool,
    pub submitting: bool,
    pub error: Option<String>,
    pub labels: Vec<GitHubLabel>,
    mode: LabelsMode,
    name: String,
    color: String,
    description: String,
}

impl LabelsModalState {
    pub fn open_list(&mut self) {
        self.open = true;
        self.loading = true;
        self.submitting = false;
        self.error = None;
        self.mode = LabelsMode::List;
        self.labels.clear();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.loading = false;
        self.submitting = false;
        self.error = None;
        self.mode = LabelsMode::List;
    }

    pub fn apply_labels(&mut self, labels: Vec<GitHubLabel>) {
        self.labels = labels;
        self.loading = false;
        self.error = None;
        self.mode = LabelsMode::List;
    }

    pub fn apply_saved(&mut self, label: GitHubLabel) {
        if let Some(existing) = self.labels.iter_mut().find(|item| item.name == label.name) {
            *existing = label.clone();
        } else if let LabelsMode::Edit { original_name } = &self.mode {
            if let Some(existing) = self
                .labels
                .iter_mut()
                .find(|item| item.name == *original_name)
            {
                *existing = label.clone();
            } else {
                self.labels.push(label);
            }
        } else {
            self.labels.push(label);
        }
        self.labels.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.submitting = false;
        self.error = None;
        self.mode = LabelsMode::List;
    }

    pub fn apply_deleted(&mut self, name: &str) {
        self.labels.retain(|label| label.name != name);
        self.submitting = false;
        self.error = None;
        self.mode = LabelsMode::List;
    }

    pub fn fail(&mut self, error: String) {
        self.loading = false;
        self.submitting = false;
        self.error = Some(error);
    }
}

impl Clone for LabelsModalState {
    fn clone(&self) -> Self {
        Self {
            open: false,
            loading: false,
            submitting: false,
            error: None,
            labels: self.labels.clone(),
            mode: LabelsMode::List,
            name: String::new(),
            color: String::new(),
            description: String::new(),
        }
    }
}

/// User action from the labels modal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelsModalAction {
    RequestList,
    Create {
        name: String,
        color: String,
        description: String,
    },
    Update {
        original_name: String,
        name: String,
        color: String,
        description: String,
    },
    Delete {
        name: String,
    },
}

pub fn show(state: &mut LabelsModalState, ui: &mut Ui) -> Option<LabelsModalAction> {
    if !state.open {
        return None;
    }

    let mut action = None;
    match state.mode.clone() {
        LabelsMode::List => {
            if state.loading && state.labels.is_empty() {
                action = Some(LabelsModalAction::RequestList);
            }
            list_body(ui, state, &mut action);
        }
        LabelsMode::Create => form_body(ui, state, "New Label", &mut action, None),
        LabelsMode::Edit { original_name } => {
            form_body(ui, state, "Edit Label", &mut action, Some(&original_name));
        }
        LabelsMode::DeleteConfirm { name } => {
            delete_body(ui, state, &name, &mut action);
        }
    }
    modal_frame::error_line(ui, &state.error);
    action
}

pub fn show_window(ctx: &egui::Context, state: &mut LabelsModalState) -> Option<LabelsModalAction> {
    if !state.open {
        return None;
    }

    let mut open = state.open;
    let mut action = None;
    modal_frame::centered_window(ctx, "Repository Labels", &mut open, |ui| {
        action = show(state, ui);
    });
    if !open {
        state.close();
    }
    action
}

fn list_body(ui: &mut Ui, state: &mut LabelsModalState, action: &mut Option<LabelsModalAction>) {
    ui.label(
        RichText::new("Create, edit, or delete labels for this repository.")
            .weak()
            .size(12.0),
    );
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui
            .add_enabled(!state.loading && !state.submitting, egui::Button::new("New Label"))
            .clicked()
        {
            state.mode = LabelsMode::Create;
            state.name.clear();
            state.color = "0366d6".into();
            state.description.clear();
            state.error = None;
        }
        if ui
            .add_enabled(!state.loading, egui::Button::new("Refresh"))
            .clicked()
        {
            state.loading = true;
            *action = Some(LabelsModalAction::RequestList);
        }
        if state.loading {
            ui.label(RichText::new("Loading…").weak().italics().size(12.0));
        }
    });

    ui.add_space(8.0);
    ScrollArea::vertical()
        .id_salt("kiwi-labels-modal-list")
        .max_height(280.0)
        .show(ui, |ui| {
            if state.labels.is_empty() && !state.loading {
                ui.label(RichText::new("No labels yet.").weak().size(12.0));
                return;
            }
            for label in state.labels.clone() {
                ui.horizontal(|ui| {
                    label_swatch(ui, &label.color);
                    ui.label(RichText::new(&label.name).strong().size(12.0));
                    if !label.description.is_empty() {
                        ui.label(
                            RichText::new(&label.description)
                                .weak()
                                .size(11.0),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(!state.submitting, egui::Button::new("Delete"))
                            .clicked()
                        {
                            state.mode = LabelsMode::DeleteConfirm {
                                name: label.name.clone(),
                            };
                            state.error = None;
                        }
                        if ui
                            .add_enabled(!state.submitting, egui::Button::new("Edit"))
                            .clicked()
                        {
                            state.mode = LabelsMode::Edit {
                                original_name: label.name.clone(),
                            };
                            state.name = label.name;
                            state.color = label.color.clone();
                            state.description = label.description;
                            state.error = None;
                        }
                    });
                });
                ui.add_space(4.0);
            }
        });

    ui.add_space(8.0);
    if ui.button("Close").clicked() {
        state.close();
    }
}

fn form_body(
    ui: &mut Ui,
    state: &mut LabelsModalState,
    title: &str,
    action: &mut Option<LabelsModalAction>,
    original_name: Option<&str>,
) {
    ui.label(RichText::new(title).strong().size(13.0));
    ui.add_space(8.0);

    ui.label(RichText::new("Name").strong().size(12.0));
    ui.add(
        TextEdit::singleline(&mut state.name)
            .desired_width(f32::INFINITY)
            .hint_text("bug"),
    );

    ui.add_space(6.0);
    ui.label(RichText::new("Color").strong().size(12.0));
    ui.horizontal(|ui| {
        label_swatch(ui, state.color.trim().trim_start_matches('#'));
        ui.add(
            TextEdit::singleline(&mut state.color)
                .desired_width(MODAL_WIDTH - 120.0)
                .hint_text("d73a4a"),
        );
    });

    ui.add_space(6.0);
    ui.label(RichText::new("Description").strong().size(12.0));
    ui.add(
        TextEdit::multiline(&mut state.description)
            .desired_width(f32::INFINITY)
            .desired_rows(3)
            .hint_text("Optional description"),
    );

    let can_submit = !state.submitting && !state.name.trim().is_empty();
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    can_submit,
                    egui::Button::new(RichText::new("Save").strong()),
                )
                .clicked()
            {
                match parse_form(state, original_name) {
                    Ok(form_action) => *action = Some(form_action),
                    Err(message) => state.error = Some(message),
                }
            }
            if ui.button("Cancel").clicked() {
                state.mode = LabelsMode::List;
                state.error = None;
            }
            if state.submitting {
                ui.label(RichText::new("Saving…").weak().italics().size(12.0));
            }
        });
    });
}

fn delete_body(
    ui: &mut Ui,
    state: &mut LabelsModalState,
    name: &str,
    action: &mut Option<LabelsModalAction>,
) {
    ui.label(
        RichText::new(format!("Delete label \"{name}\"?"))
            .strong()
            .size(13.0),
    );
    ui.add_space(8.0);
    ui.label(
        RichText::new("This removes the label from the repository. Issues keep their history but lose this label.")
            .weak()
            .size(12.0),
    );

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(!state.submitting, egui::Button::new(RichText::new("Delete").strong()))
                .clicked()
            {
                *action = Some(LabelsModalAction::Delete {
                    name: name.to_string(),
                });
            }
            if ui.button("Cancel").clicked() {
                state.mode = LabelsMode::List;
                state.error = None;
            }
            if state.submitting {
                ui.label(RichText::new("Deleting…").weak().italics().size(12.0));
            }
        });
    });
}

fn parse_form(
    state: &LabelsModalState,
    original_name: Option<&str>,
) -> Result<LabelsModalAction, String> {
    let name = state.name.trim().to_string();
    if name.is_empty() {
        return Err("Name is required".into());
    }
    let color = modal_frame::parse_label_color(&state.color)?;
    let description = state.description.trim().to_string();
    match original_name {
        Some(original) => Ok(LabelsModalAction::Update {
            original_name: original.to_string(),
            name,
            color,
            description,
        }),
        None => Ok(LabelsModalAction::Create {
            name,
            color,
            description,
        }),
    }
}

fn label_swatch(ui: &mut Ui, color: &str) {
    let parsed = parse_hex_color(color).unwrap_or(Color32::from_rgb(3, 102, 214));
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(3), parsed);
}

fn parse_hex_color(color: &str) -> Option<Color32> {
    if color.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&color[0..2], 16).ok()?;
    let g = u8::from_str_radix(&color[2..4], 16).ok()?;
    let b = u8::from_str_radix(&color[4..6], 16).ok()?;
    Some(Color32::from_rgb(r, g, b))
}
