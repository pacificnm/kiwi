//! Modal for merging a GitHub pull request.

use egui::{Align2, RichText, Ui, Window};

const MODAL_WIDTH: f32 = 480.0;

/// GitHub merge strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MergeMethod {
    #[default]
    Merge,
    Squash,
    Rebase,
}

impl MergeMethod {
    fn label(self) -> &'static str {
        match self {
            Self::Merge => "Create a merge commit",
            Self::Squash => "Squash and merge",
            Self::Rebase => "Rebase and merge",
        }
    }

    fn api_value(self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::Squash => "squash",
            Self::Rebase => "rebase",
        }
    }
}

/// State for the merge pull request modal.
#[derive(Debug, Default)]
pub struct PrMergeModalState {
    pub open: bool,
    pub submitting: bool,
    pub error: Option<String>,
    pub number: u64,
    pub title: String,
    pub head_branch: String,
    pub base_branch: String,
    pub mergeable: Option<bool>,
    pub mergeable_state: String,
    pub draft: bool,
    pub merged: bool,
    pub method: MergeMethod,
}

impl PrMergeModalState {
    pub fn open_for_pr(
        &mut self,
        number: u64,
        title: String,
        head_branch: String,
        base_branch: String,
        mergeable: Option<bool>,
        mergeable_state: String,
        draft: bool,
        merged: bool,
    ) {
        self.open = true;
        self.submitting = false;
        self.error = None;
        self.number = number;
        self.title = title;
        self.head_branch = head_branch;
        self.base_branch = base_branch;
        self.mergeable = mergeable;
        self.mergeable_state = mergeable_state;
        self.draft = draft;
        self.merged = merged;
        self.method = MergeMethod::Merge;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.submitting = false;
        self.error = None;
    }

    pub fn fail(&mut self, error: String) {
        self.submitting = false;
        self.error = Some(error);
    }

    pub fn can_merge(&self) -> bool {
        !self.submitting
            && !self.merged
            && !self.draft
            && self.mergeable != Some(false)
    }
}

impl Clone for PrMergeModalState {
    fn clone(&self) -> Self {
        Self {
            open: false,
            submitting: false,
            error: None,
            number: self.number,
            title: self.title.clone(),
            head_branch: self.head_branch.clone(),
            base_branch: self.base_branch.clone(),
            mergeable: self.mergeable,
            mergeable_state: self.mergeable_state.clone(),
            draft: self.draft,
            merged: self.merged,
            method: self.method,
        }
    }
}

/// User action from the merge PR modal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrMergeModalAction {
    Merge {
        number: u64,
        merge_method: String,
    },
}

pub fn show_window(
    ctx: &egui::Context,
    state: &mut PrMergeModalState,
) -> Option<PrMergeModalAction> {
    if !state.open {
        return None;
    }

    let mut open = state.open;
    let mut action = None;
    let title = format!("Merge Pull Request #{}", state.number);

    Window::new(title)
        .collapsible(false)
        .resizable(false)
        .default_width(MODAL_WIDTH)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_width(MODAL_WIDTH - 24.0);
            action = modal_body(ui, state);
        });

    if !open {
        state.close();
    }

    action
}

fn modal_body(ui: &mut Ui, state: &mut PrMergeModalState) -> Option<PrMergeModalAction> {
    ui.label(RichText::new(&state.title).strong().size(13.0));
    ui.add_space(8.0);
    ui.label(
        RichText::new(format!(
            "Merge `{}` into `{}`",
            state.head_branch, state.base_branch
        ))
        .size(12.0),
    );

    if state.merged {
        ui.add_space(6.0);
        ui.label(
            RichText::new("This pull request is already merged.")
                .color(ui.visuals().warn_fg_color)
                .size(12.0),
        );
    } else if state.draft {
        ui.add_space(6.0);
        ui.label(
            RichText::new("Draft pull requests must be marked ready before merging.")
                .color(ui.visuals().warn_fg_color)
                .size(12.0),
        );
    } else if state.mergeable == Some(false) {
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!(
                "GitHub reports this PR cannot be merged ({})",
                state.mergeable_state
            ))
            .color(ui.visuals().warn_fg_color)
            .size(12.0),
        );
    }

    ui.add_space(8.0);
    ui.label(RichText::new("Merge method").strong().size(12.0));
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.method, MergeMethod::Merge, MergeMethod::Merge.label());
    });
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.method, MergeMethod::Squash, MergeMethod::Squash.label());
    });
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.method, MergeMethod::Rebase, MergeMethod::Rebase.label());
    });

    if let Some(error) = &state.error {
        ui.add_space(6.0);
        ui.label(
            RichText::new(error)
                .color(ui.visuals().error_fg_color)
                .size(12.0),
        );
    }

    ui.add_space(12.0);
    let mut action = None;
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    state.can_merge(),
                    egui::Button::new(RichText::new("Merge").strong()),
                )
                .clicked()
            {
                action = Some(PrMergeModalAction::Merge {
                    number: state.number,
                    merge_method: state.method.api_value().to_string(),
                });
            }
            if ui.button("Cancel").clicked() {
                state.close();
            }
            if state.submitting {
                ui.label(RichText::new("Merging…").weak().italics().size(12.0));
            }
        });
    });

    action
}
