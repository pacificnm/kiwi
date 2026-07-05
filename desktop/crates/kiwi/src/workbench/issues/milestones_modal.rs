//! Modal for viewing, creating, editing, and deleting repository milestones.

use egui::{RichText, ScrollArea, TextEdit, Ui};

use super::github_metadata::GitHubMilestone;
use super::modal_frame;

/// Milestones modal mode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum MilestonesMode {
    #[default]
    List,
    Create,
    Edit {
        number: u64,
    },
    DeleteConfirm {
        number: u64,
        title: String,
    },
}

/// State for the repository milestones manager modal.
#[derive(Debug, Default)]
pub struct MilestonesModalState {
    pub open: bool,
    pub loading: bool,
    pub submitting: bool,
    pub error: Option<String>,
    pub milestones: Vec<GitHubMilestone>,
    mode: MilestonesMode,
    title: String,
    description: String,
    due_on: String,
    state: String,
}

impl MilestonesModalState {
    pub fn open_list(&mut self) {
        self.open = true;
        self.loading = true;
        self.submitting = false;
        self.error = None;
        self.mode = MilestonesMode::List;
        self.milestones.clear();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.loading = false;
        self.submitting = false;
        self.error = None;
        self.mode = MilestonesMode::List;
    }

    pub fn apply_milestones(&mut self, milestones: Vec<GitHubMilestone>) {
        self.milestones = milestones;
        self.loading = false;
        self.error = None;
        self.mode = MilestonesMode::List;
    }

    pub fn apply_saved(&mut self, milestone: GitHubMilestone) {
        if let Some(existing) = self
            .milestones
            .iter_mut()
            .find(|item| item.number == milestone.number)
        {
            *existing = milestone;
        } else {
            self.milestones.push(milestone);
        }
        self.milestones
            .sort_by(|a, b| a.number.cmp(&b.number));
        self.submitting = false;
        self.error = None;
        self.mode = MilestonesMode::List;
    }

    pub fn apply_deleted(&mut self, number: u64) {
        self.milestones.retain(|item| item.number != number);
        self.submitting = false;
        self.error = None;
        self.mode = MilestonesMode::List;
    }

    pub fn fail(&mut self, error: String) {
        self.loading = false;
        self.submitting = false;
        self.error = Some(error);
    }
}

impl Clone for MilestonesModalState {
    fn clone(&self) -> Self {
        Self {
            open: false,
            loading: false,
            submitting: false,
            error: None,
            milestones: self.milestones.clone(),
            mode: MilestonesMode::List,
            title: String::new(),
            description: String::new(),
            due_on: String::new(),
            state: "open".into(),
        }
    }
}

/// User action from the milestones modal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MilestonesModalAction {
    RequestList,
    Create {
        title: String,
        description: String,
        due_on: Option<String>,
    },
    Update {
        number: u64,
        title: String,
        description: String,
        due_on: Option<String>,
        state: String,
    },
    Delete {
        number: u64,
    },
}

pub fn show_window(
    ctx: &egui::Context,
    state: &mut MilestonesModalState,
) -> Option<MilestonesModalAction> {
    if !state.open {
        return None;
    }

    let mut open = state.open;
    let mut action = None;
    modal_frame::centered_window(ctx, "Repository Milestones", &mut open, |ui| {
        action = show(state, ui);
    });
    if !open {
        state.close();
    }
    action
}

fn show(state: &mut MilestonesModalState, ui: &mut Ui) -> Option<MilestonesModalAction> {
    let mut action = None;
    match state.mode.clone() {
        MilestonesMode::List => {
            if state.loading && state.milestones.is_empty() {
                action = Some(MilestonesModalAction::RequestList);
            }
            list_body(ui, state, &mut action);
        }
        MilestonesMode::Create => form_body(ui, state, "New Milestone", &mut action, None),
        MilestonesMode::Edit { number } => {
            form_body(ui, state, "Edit Milestone", &mut action, Some(number));
        }
        MilestonesMode::DeleteConfirm { number, title } => {
            delete_body(ui, state, number, &title, &mut action);
        }
    }
    modal_frame::error_line(ui, &state.error);
    action
}

fn list_body(
    ui: &mut Ui,
    state: &mut MilestonesModalState,
    action: &mut Option<MilestonesModalAction>,
) {
    ui.label(
        RichText::new("Create, edit, or delete milestones for this repository.")
            .weak()
            .size(12.0),
    );
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui
            .add_enabled(!state.loading && !state.submitting, egui::Button::new("New Milestone"))
            .clicked()
        {
            state.mode = MilestonesMode::Create;
            state.title.clear();
            state.description.clear();
            state.due_on.clear();
            state.state = "open".into();
            state.error = None;
        }
        if ui
            .add_enabled(!state.loading, egui::Button::new("Refresh"))
            .clicked()
        {
            state.loading = true;
            *action = Some(MilestonesModalAction::RequestList);
        }
        if state.loading {
            ui.label(RichText::new("Loading…").weak().italics().size(12.0));
        }
    });

    ui.add_space(8.0);
    ScrollArea::vertical()
        .id_salt("kiwi-milestones-modal-list")
        .max_height(280.0)
        .show(ui, |ui| {
            if state.milestones.is_empty() && !state.loading {
                ui.label(RichText::new("No milestones yet.").weak().size(12.0));
                return;
            }
            for milestone in state.milestones.clone() {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("#{} {}", milestone.number, milestone.title))
                            .strong()
                            .size(12.0),
                    );
                    ui.label(
                        RichText::new(&milestone.state)
                            .weak()
                            .size(11.0),
                    );
                    if let Some(due) = milestone.due_on.as_deref().and_then(|v| v.get(0..10)) {
                        ui.label(RichText::new(format!("due {due}")).weak().size(11.0));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(!state.submitting, egui::Button::new("Delete"))
                            .clicked()
                        {
                            state.mode = MilestonesMode::DeleteConfirm {
                                number: milestone.number,
                                title: milestone.title.clone(),
                            };
                            state.error = None;
                        }
                        if ui
                            .add_enabled(!state.submitting, egui::Button::new("Edit"))
                            .clicked()
                        {
                            state.mode = MilestonesMode::Edit {
                                number: milestone.number,
                            };
                            state.title = milestone.title;
                            state.description = milestone.description;
                            state.due_on = modal_frame::format_due_on(&milestone.due_on);
                            state.state = milestone.state;
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
    state: &mut MilestonesModalState,
    title: &str,
    action: &mut Option<MilestonesModalAction>,
    edit_number: Option<u64>,
) {
    ui.label(RichText::new(title).strong().size(13.0));
    ui.add_space(8.0);

    ui.label(RichText::new("Title").strong().size(12.0));
    ui.add(
        TextEdit::singleline(&mut state.title)
            .desired_width(f32::INFINITY)
            .hint_text("v1.0"),
    );

    ui.add_space(6.0);
    ui.label(RichText::new("Description").strong().size(12.0));
    ui.add(
        TextEdit::multiline(&mut state.description)
            .desired_width(f32::INFINITY)
            .desired_rows(3),
    );

    ui.add_space(6.0);
    ui.label(RichText::new("Due date").strong().size(12.0));
    ui.add(
        TextEdit::singleline(&mut state.due_on)
            .desired_width(f32::INFINITY)
            .hint_text("YYYY-MM-DD"),
    );

    if edit_number.is_some() {
        ui.add_space(6.0);
        ui.label(RichText::new("State").strong().size(12.0));
        ui.horizontal(|ui| {
            ui.selectable_value(&mut state.state, "open".into(), "open");
            ui.selectable_value(&mut state.state, "closed".into(), "closed");
        });
    }

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    !state.submitting && !state.title.trim().is_empty(),
                    egui::Button::new(RichText::new("Save").strong()),
                )
                .clicked()
            {
                match parse_form(state, edit_number) {
                    Ok(form_action) => *action = Some(form_action),
                    Err(message) => state.error = Some(message),
                }
            }
            if ui.button("Cancel").clicked() {
                state.mode = MilestonesMode::List;
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
    state: &mut MilestonesModalState,
    number: u64,
    title: &str,
    action: &mut Option<MilestonesModalAction>,
) {
    ui.label(
        RichText::new(format!("Delete milestone #{number} \"{title}\"?"))
            .strong()
            .size(13.0),
    );
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(!state.submitting, egui::Button::new(RichText::new("Delete").strong()))
                .clicked()
            {
                *action = Some(MilestonesModalAction::Delete { number });
            }
            if ui.button("Cancel").clicked() {
                state.mode = MilestonesMode::List;
                state.error = None;
            }
            if state.submitting {
                ui.label(RichText::new("Deleting…").weak().italics().size(12.0));
            }
        });
    });
}

fn parse_form(
    state: &MilestonesModalState,
    edit_number: Option<u64>,
) -> Result<MilestonesModalAction, String> {
    let title = state.title.trim().to_string();
    if title.is_empty() {
        return Err("Title is required".into());
    }
    let description = state.description.trim().to_string();
    let due_on = modal_frame::parse_due_on(&state.due_on)?;
    match edit_number {
        Some(number) => Ok(MilestonesModalAction::Update {
            number,
            title,
            description,
            due_on,
            state: state.state.clone(),
        }),
        None => Ok(MilestonesModalAction::Create {
            title,
            description,
            due_on,
        }),
    }
}
