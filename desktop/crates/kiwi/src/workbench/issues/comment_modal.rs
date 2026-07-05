//! Modal dialog for posting a GitHub issue comment.

use egui::{Align2, ComboBox, RichText, TextEdit, Ui, Window};

use crate::workbench::issues::IssueListItem;

const MODAL_WIDTH: f32 = 440.0;

/// User action from the new-comment modal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentModalAction {
    /// Post the comment to GitHub.
    Submit {
        /// Target issue number.
        issue_number: u64,
        /// Comment body markdown.
        body: String,
    },
}

/// State for the new GitHub comment modal.
#[derive(Debug, Default)]
pub struct CommentModalState {
    /// Whether the modal is visible.
    pub open: bool,
    /// Issue number input.
    pub issue_number: String,
    /// Comment body draft.
    pub body: String,
    /// True while the comment is posting.
    pub submitting: bool,
    /// Validation or API error message.
    pub error: Option<String>,
}

impl CommentModalState {
    /// Opens the modal, optionally pre-filling the issue number.
    pub fn open_with_issue(&mut self, issue_number: Option<u64>) {
        self.open = true;
        self.issue_number = issue_number
            .map(|number| number.to_string())
            .unwrap_or_default();
        self.body.clear();
        self.error = None;
        self.submitting = false;
    }

    /// Marks the modal as closed and clears transient state.
    pub fn close(&mut self) {
        self.open = false;
        self.submitting = false;
        self.error = None;
    }
}

impl Clone for CommentModalState {
    fn clone(&self) -> Self {
        Self {
            open: false,
            issue_number: self.issue_number.clone(),
            body: String::new(),
            submitting: false,
            error: None,
        }
    }
}

/// Renders the centered new-comment modal when [`CommentModalState::open`] is true.
pub fn show(
    ctx: &egui::Context,
    modal: &mut CommentModalState,
    issues: &[IssueListItem],
) -> Option<CommentModalAction> {
    if !modal.open {
        return None;
    }

    let mut action = None;
    let mut open = modal.open;

    Window::new("New GitHub Comment")
        .collapsible(false)
        .resizable(false)
        .fixed_size([MODAL_WIDTH, 0.0])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_width(MODAL_WIDTH - 24.0);
            action = modal_body(ui, modal, issues);
        });

    if !open {
        modal.close();
    }

    action
}

fn modal_body(
    ui: &mut Ui,
    modal: &mut CommentModalState,
    issues: &[IssueListItem],
) -> Option<CommentModalAction> {
    ui.label(
        RichText::new("Post a comment on a GitHub issue in the current repository.")
            .weak()
            .size(12.0),
    );
    ui.add_space(8.0);

    if !issues.is_empty() {
        ui.label(RichText::new("Open issue").strong().size(12.0));
        let selected_label = issues
            .iter()
            .find(|item| modal.issue_number == item.number.to_string())
            .map(|item| format!("#{} {}", item.number, item.title))
            .unwrap_or_else(|| "Select an issue…".to_string());

        ComboBox::from_id_salt("kiwi-comment-issue-picker")
            .selected_text(selected_label)
            .width(MODAL_WIDTH - 48.0)
            .show_ui(ui, |ui| {
                for item in issues {
                    if ui
                        .selectable_label(
                            modal.issue_number == item.number.to_string(),
                            format!("#{} {}", item.number, item.title),
                        )
                        .clicked()
                    {
                        modal.issue_number = item.number.to_string();
                        modal.error = None;
                    }
                }
            });
        ui.add_space(6.0);
    }

    ui.label(RichText::new("Issue number").strong().size(12.0));
    ui.add(
        TextEdit::singleline(&mut modal.issue_number)
            .desired_width(f32::INFINITY)
            .hint_text("e.g. 42"),
    );

    ui.add_space(8.0);
    ui.label(RichText::new("Comment").strong().size(12.0));
    ui.add(
        TextEdit::multiline(&mut modal.body)
            .desired_width(f32::INFINITY)
            .desired_rows(8)
            .hint_text("Write your comment…"),
    );

    if let Some(error) = &modal.error {
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
            let can_submit = !modal.submitting
                && !modal.issue_number.trim().is_empty()
                && !modal.body.trim().is_empty();

            if ui
                .add_enabled(
                    can_submit,
                    egui::Button::new(RichText::new("Post Comment").strong()),
                )
                .clicked()
            {
                match parse_submit(modal) {
                    Ok(submit) => action = Some(submit),
                    Err(message) => modal.error = Some(message),
                }
            }

            if ui.button("Cancel").clicked() {
                modal.close();
            }

            if modal.submitting {
                ui.label(RichText::new("Posting…").weak().italics().size(12.0));
            }
        });
    });

    action
}

fn parse_submit(modal: &CommentModalState) -> Result<CommentModalAction, String> {
    let issue_number = modal
        .issue_number
        .trim()
        .parse::<u64>()
        .map_err(|_| "Issue number must be a positive integer".to_string())?;
    let body = modal.body.trim().to_string();
    if body.is_empty() {
        return Err("Comment cannot be empty".into());
    }
    Ok(CommentModalAction::Submit {
        issue_number,
        body,
    })
}
