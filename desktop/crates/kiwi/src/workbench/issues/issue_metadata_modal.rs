//! Modal for assigning labels and a milestone to a GitHub issue.

use egui::{ComboBox, RichText, ScrollArea, Ui};

use super::github_metadata::{GitHubLabel, GitHubMilestone};
use super::modal_frame::{self, MODAL_WIDTH};

/// State for the issue labels/milestone assign modal.
#[derive(Debug, Default)]
pub struct IssueMetadataModalState {
    pub open: bool,
    pub loading: bool,
    pub submitting: bool,
    pub error: Option<String>,
    pub issue_number: u64,
    pub labels: Vec<GitHubLabel>,
    pub milestones: Vec<GitHubMilestone>,
    pub selected_labels: Vec<String>,
    pub selected_milestone: Option<u64>,
}

impl IssueMetadataModalState {
    pub fn open_for_issue(&mut self, issue_number: u64) {
        self.open = true;
        self.loading = true;
        self.submitting = false;
        self.error = None;
        self.issue_number = issue_number;
        self.labels.clear();
        self.milestones.clear();
        self.selected_labels.clear();
        self.selected_milestone = None;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.loading = false;
        self.submitting = false;
        self.error = None;
    }

    pub fn apply_loaded(
        &mut self,
        labels: Vec<GitHubLabel>,
        milestones: Vec<GitHubMilestone>,
        selected_labels: Vec<String>,
        milestone: Option<u64>,
    ) {
        self.labels = labels;
        self.milestones = milestones;
        self.selected_labels = selected_labels;
        self.selected_milestone = milestone;
        self.loading = false;
        self.error = None;
    }

    pub fn fail(&mut self, error: String) {
        self.loading = false;
        self.submitting = false;
        self.error = Some(error);
    }
}

impl Clone for IssueMetadataModalState {
    fn clone(&self) -> Self {
        Self {
            open: false,
            loading: false,
            submitting: false,
            error: None,
            issue_number: self.issue_number,
            labels: self.labels.clone(),
            milestones: self.milestones.clone(),
            selected_labels: self.selected_labels.clone(),
            selected_milestone: self.selected_milestone,
        }
    }
}

/// User action from the issue metadata modal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueMetadataModalAction {
    RequestLoad,
    Save {
        issue_number: u64,
        labels: Vec<String>,
        milestone: Option<u64>,
    },
}

pub fn show_window(
    ctx: &egui::Context,
    state: &mut IssueMetadataModalState,
) -> Option<IssueMetadataModalAction> {
    if !state.open {
        return None;
    }

    let mut open = state.open;
    let mut action = None;
    let title = format!("Issue #{} Labels & Milestone", state.issue_number);
    modal_frame::centered_window(ctx, &title, &mut open, |ui| {
        action = show(state, ui);
    });
    if !open {
        state.close();
    }
    action
}

fn show(state: &mut IssueMetadataModalState, ui: &mut Ui) -> Option<IssueMetadataModalAction> {
    let mut action = None;

    if state.loading && state.labels.is_empty() {
        ui.label(RichText::new("Loading issue metadata…").weak().size(12.0));
        return Some(IssueMetadataModalAction::RequestLoad);
    }

    ui.label(
        RichText::new("Select labels and a milestone for this issue.")
            .weak()
            .size(12.0),
    );
    ui.add_space(8.0);

    ui.label(RichText::new("Labels").strong().size(12.0));
    ScrollArea::vertical()
        .id_salt(("kiwi-issue-labels", state.issue_number))
        .max_height(180.0)
        .show(ui, |ui| {
            if state.labels.is_empty() {
                ui.label(
                    RichText::new("No repository labels. Use Git → Manage Labels…")
                        .weak()
                        .size(11.0),
                );
            }
            for label in state.labels.clone() {
                let mut selected = state.selected_labels.iter().any(|name| name == &label.name);
                if ui.checkbox(&mut selected, &label.name).changed() {
                    if selected {
                        if !state.selected_labels.iter().any(|name| name == &label.name) {
                            state.selected_labels.push(label.name.clone());
                        }
                    } else {
                        state.selected_labels.retain(|name| name != &label.name);
                    }
                }
            }
        });

    ui.add_space(8.0);
    ui.label(RichText::new("Milestone").strong().size(12.0));
    let selected_text = state
        .selected_milestone
        .and_then(|number| {
            state
                .milestones
                .iter()
                .find(|item| item.number == number)
                .map(|item| format!("#{} {}", item.number, item.title))
        })
        .unwrap_or_else(|| "None".to_string());

    ComboBox::from_id_salt(("kiwi-issue-milestone", state.issue_number))
        .selected_text(selected_text)
        .width((ui.available_width().max(MODAL_WIDTH - 48.0)).min(MODAL_WIDTH - 24.0))
        .show_ui(ui, |ui| {
            if ui.selectable_label(state.selected_milestone.is_none(), "None").clicked() {
                state.selected_milestone = None;
            }
            for milestone in state.milestones.clone() {
                let selected = state.selected_milestone == Some(milestone.number);
                if ui
                    .selectable_label(
                        selected,
                        format!("#{} {} ({})", milestone.number, milestone.title, milestone.state),
                    )
                    .clicked()
                {
                    state.selected_milestone = Some(milestone.number);
                }
            }
        });

    ui.add_space(8.0);
    modal_frame::error_line(ui, &state.error);
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    !state.submitting && !state.loading,
                    egui::Button::new(RichText::new("Save").strong()),
                )
                .clicked()
            {
                action = Some(IssueMetadataModalAction::Save {
                    issue_number: state.issue_number,
                    labels: state.selected_labels.clone(),
                    milestone: state.selected_milestone,
                });
            }
            if ui.button("Cancel").clicked() {
                state.close();
            }
            if state.submitting {
                ui.label(RichText::new("Saving…").weak().italics().size(12.0));
            }
        });
    });

    action
}
