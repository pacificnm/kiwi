//! Modal for creating a new git branch.

use egui::{Align2, ComboBox, RichText, TextEdit, Ui, Window};

const MODAL_WIDTH: f32 = 440.0;

/// State for the create-branch modal.
#[derive(Debug, Default)]
pub struct BranchCreateModalState {
    pub open: bool,
    pub submitting: bool,
    pub error: Option<String>,
    pub name: String,
    pub start_branch: String,
    pub issue_number: Option<u64>,
}

impl BranchCreateModalState {
    pub fn open_with_branches(&mut self, branches: &[String], current_branch: &str) {
        self.open = true;
        self.submitting = false;
        self.error = None;
        self.issue_number = None;
        self.name.clear();
        self.start_branch = default_start_branch(branches, current_branch);
    }

    pub fn open_for_issue(
        &mut self,
        branches: &[String],
        current_branch: &str,
        issue_number: u64,
        issue_title: &str,
    ) {
        self.open = true;
        self.submitting = false;
        self.error = None;
        self.issue_number = Some(issue_number);
        self.name = branch_name_from_issue(issue_number, issue_title);
        self.start_branch = default_start_branch(branches, current_branch);
    }

    pub fn close(&mut self) {
        self.open = false;
        self.submitting = false;
        self.error = None;
        self.issue_number = None;
    }

    pub fn fail(&mut self, error: String) {
        self.submitting = false;
        self.error = Some(error);
    }
}

impl Clone for BranchCreateModalState {
    fn clone(&self) -> Self {
        Self {
            open: false,
            submitting: false,
            error: None,
            name: String::new(),
            start_branch: self.start_branch.clone(),
            issue_number: None,
        }
    }
}

/// User action from the create-branch modal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchCreateModalAction {
    Create {
        name: String,
        start_branch: Option<String>,
    },
}

pub fn show_window(
    ctx: &egui::Context,
    state: &mut BranchCreateModalState,
    branches: &[String],
) -> Option<BranchCreateModalAction> {
    if !state.open {
        return None;
    }

    let mut open = state.open;
    let mut action = None;
    let title = match state.issue_number {
        Some(number) => format!("Create Branch from Issue #{number}"),
        None => "Create Branch".into(),
    };

    Window::new(title)
        .collapsible(false)
        .resizable(false)
        .default_width(MODAL_WIDTH)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_width(MODAL_WIDTH - 24.0);
            action = modal_body(ui, state, branches);
        });

    if !open {
        state.close();
    }

    action
}

fn modal_body(
    ui: &mut Ui,
    state: &mut BranchCreateModalState,
    branches: &[String],
) -> Option<BranchCreateModalAction> {
    ui.label(
        RichText::new(if state.issue_number.is_some() {
            "Suggested branch name from the issue title — edit before creating."
        } else {
            "Create a new branch from the selected starting point."
        })
        .weak()
        .size(12.0),
    );
    ui.add_space(8.0);

    ui.label(RichText::new("Branch name").strong().size(12.0));
    ui.add(
        TextEdit::singleline(&mut state.name)
            .desired_width(ui.available_width().max(200.0))
            .hint_text("feature/my-branch"),
    );

    ui.add_space(6.0);
    ui.label(RichText::new("Start from").strong().size(12.0));
    let start_label = if state.start_branch.is_empty() {
        "Current HEAD".to_string()
    } else {
        state.start_branch.clone()
    };
    ComboBox::from_id_salt("kiwi-create-branch-start")
        .selected_text(start_label)
        .width(ui.available_width().max(200.0).min(MODAL_WIDTH - 48.0))
        .show_ui(ui, |ui| {
            if ui.selectable_label(state.start_branch.is_empty(), "Current HEAD").clicked() {
                state.start_branch.clear();
            }
            for branch in branches {
                if ui
                    .selectable_label(state.start_branch == *branch, branch)
                    .clicked()
                {
                    state.start_branch = branch.clone();
                }
            }
        });

    if let Some(error) = &state.error {
        ui.add_space(6.0);
        ui.label(
            RichText::new(error)
                .color(ui.visuals().error_fg_color)
                .size(12.0),
        );
    }

    let can_create = !state.submitting && !state.name.trim().is_empty();
    ui.add_space(12.0);
    let mut action = None;
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    can_create,
                    egui::Button::new(RichText::new("Create").strong()),
                )
                .clicked()
            {
                match parse_create(state) {
                    Ok(create) => action = Some(create),
                    Err(message) => state.error = Some(message),
                }
            }
            if ui.button("Cancel").clicked() {
                state.close();
            }
            if state.submitting {
                ui.label(RichText::new("Creating…").weak().italics().size(12.0));
            }
        });
    });

    action
}

fn parse_create(state: &BranchCreateModalState) -> Result<BranchCreateModalAction, String> {
    let name = state.name.trim().to_string();
    if name.is_empty() {
        return Err("Branch name is required".into());
    }
    if name.contains(' ') {
        return Err("Branch name cannot contain spaces".into());
    }
    let start_branch = if state.start_branch.trim().is_empty() {
        None
    } else {
        Some(state.start_branch.trim().to_string())
    };
    Ok(BranchCreateModalAction::Create { name, start_branch })
}

fn default_start_branch(branches: &[String], current_branch: &str) -> String {
    if current_branch == "HEAD" {
        branches.first().cloned().unwrap_or_default()
    } else {
        current_branch.to_string()
    }
}

/// Builds a git-safe branch name from a GitHub issue number and title.
pub fn branch_name_from_issue(number: u64, title: &str) -> String {
    let slug = slugify_branch_segment(title);
    if slug.is_empty() {
        format!("issue-{number}")
    } else {
        format!("{number}-{slug}")
    }
}

fn slugify_branch_segment(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in title.chars().take(80) {
        let segment = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else if ch.is_whitespace() || matches!(ch, '-' | '_' | '/' | '.') {
            '-'
        } else {
            continue;
        };
        if segment == '-' {
            if !last_dash && !slug.is_empty() {
                slug.push('-');
                last_dash = true;
            }
        } else {
            last_dash = false;
            slug.push(segment);
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.len() > 48 {
        slug.truncate(48);
        while slug.ends_with('-') {
            slug.pop();
        }
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::branch_name_from_issue;

    #[test]
    fn branch_name_from_issue_includes_number_and_slug() {
        assert_eq!(
            branch_name_from_issue(42, "Fix NaN in ComboBox width"),
            "42-fix-nan-in-combobox-width"
        );
    }

    #[test]
    fn branch_name_from_issue_falls_back_without_title() {
        assert_eq!(branch_name_from_issue(7, "!!!"), "issue-7");
    }
}
