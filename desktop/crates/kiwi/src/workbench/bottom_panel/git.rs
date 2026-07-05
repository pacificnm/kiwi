//! Git bottom panel — command output log and commit history.

use egui::{Align, Button, Frame, Layout, RichText, ScrollArea, Ui};

use crate::theme::PALETTE;
use crate::workbench::source_control::{GitCommitEntry, GitOutputEntry, GitPanelView};
use crate::workbench::state::WorkbenchState;

/// Renders git output or commit history for the bottom panel.
pub fn show(ui: &mut Ui, state: &mut WorkbenchState) {
    if state.source_control.git_panel_view == GitPanelView::History
        && !state.source_control.git_commits_loading
        && state.source_control.git_commits.is_empty()
        && state.source_control.git_commits_error.is_none()
        && !state.source_control.not_repo
    {
        state
            .source_control
            .request_commit_history(&state.project.root);
    }

    header(ui, state);
    ui.add_space(4.0);

    match state.source_control.git_panel_view {
        GitPanelView::Output => output_view(ui, state.source_control.git_output.entries()),
        GitPanelView::History => history_view(ui, state),
    }
}

fn header(ui: &mut Ui, state: &mut WorkbenchState) {
    let source = &mut state.source_control;

    ui.horizontal(|ui| {
        view_toggle(ui, &mut source.git_panel_view);

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            match source.git_panel_view {
                GitPanelView::Output => {
                    if ui.small_button("Clear").clicked() {
                        source.git_output.clear();
                    }
                }
                GitPanelView::History => {
                    let can_refresh = !source.git_commits_loading && !source.not_repo;
                    if ui
                        .add_enabled(can_refresh, Button::new(RichText::new("Refresh").size(11.0)))
                        .clicked()
                    {
                        source.request_commit_history(&state.project.root);
                    }
                }
            }
        });
    });
}

fn view_toggle(ui: &mut Ui, view: &mut GitPanelView) {
    ui.spacing_mut().item_spacing.x = 4.0;
    if ui
        .selectable_label(*view == GitPanelView::Output, RichText::new("Output").size(11.0))
        .clicked()
    {
        *view = GitPanelView::Output;
    }
    if ui
        .selectable_label(*view == GitPanelView::History, RichText::new("History").size(11.0))
        .clicked()
    {
        *view = GitPanelView::History;
    }
}

fn output_view(ui: &mut Ui, entries: &[GitOutputEntry]) {
    if entries.is_empty() {
        ui.label(
            RichText::new("Git command output appears here after stage, commit, or push.")
                .weak()
                .size(12.0),
        );
        return;
    }

    for entry in entries.iter().rev() {
        output_entry(ui, entry);
        ui.add_space(6.0);
    }
}

fn output_entry(ui: &mut Ui, entry: &GitOutputEntry) {
    let status_color = if entry.success {
        PALETTE.success
    } else {
        PALETTE.error
    };

    Frame::new()
        .fill(PALETTE.background_sidebar)
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(&entry.time)
                        .monospace()
                        .size(11.0)
                        .color(PALETTE.text_muted),
                );
                ui.label(
                    RichText::new(if entry.success { "OK" } else { "ERR" })
                        .monospace()
                        .size(11.0)
                        .color(status_color),
                );
                ui.label(
                    RichText::new(&entry.command)
                        .monospace()
                        .size(11.0)
                        .color(PALETTE.text_secondary),
                );
            });
            ui.add_space(2.0);
            ui.label(
                RichText::new(&entry.text)
                    .monospace()
                    .size(12.0)
                    .color(if entry.success {
                        PALETTE.text_primary
                    } else {
                        PALETTE.error
                    }),
            );
        });
}

fn history_view(ui: &mut Ui, state: &mut WorkbenchState) {
    let source = &mut state.source_control;

    if source.not_repo {
        ui.label(
            RichText::new("This folder is not a Git repository.")
                .weak()
                .size(12.0),
        );
        return;
    }

    if source.git_commits_loading && source.git_commits.is_empty() {
        ui.label(RichText::new("Loading commit history…").weak().size(12.0));
        return;
    }

    if let Some(error) = &source.git_commits_error {
        ui.label(
            RichText::new(error)
                .color(ui.visuals().error_fg_color)
                .size(12.0),
        );
        ui.add_space(4.0);
    }

    if source.git_commits.is_empty() {
        ui.label(
            RichText::new("No commits found.")
                .weak()
                .size(12.0),
        );
        return;
    }

    ScrollArea::vertical()
        .id_salt("kiwi-git-history")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let commits = source.git_commits.clone();
            for (index, commit) in commits.iter().enumerate() {
                history_row(ui, &mut source.git_selected_commit, index, commit);
            }
        });
}

fn history_row(
    ui: &mut Ui,
    selected: &mut Option<usize>,
    index: usize,
    commit: &GitCommitEntry,
) {
    let is_selected = *selected == Some(index);

    let response = ui
        .selectable_label(
            is_selected,
            RichText::new(format!(
                "{}  {}  {}",
                commit.short_hash, commit.date, commit.subject
            ))
            .monospace()
            .size(12.0),
        )
        .on_hover_text(format!("{} · {}", commit.author, commit.hash));

    if response.clicked() {
        *selected = if is_selected { None } else { Some(index) };
    }

    if is_selected {
        ui.indent("kiwi-git-commit-detail", |ui| {
            ui.label(
                RichText::new(format!(
                    "{}\n{}\n{}",
                    commit.hash, commit.author, commit.subject
                ))
                .weak()
                .size(11.0),
            );
        });
    }
}
