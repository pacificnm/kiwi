//! Left sidebar — dispatches to activity-specific panels.

mod agent;
mod explorer;
mod extensions;
mod issues;
mod search;
mod source_control;
mod tasks;
mod tools;

use egui::{RichText, ScrollArea, Ui};

use nest_core::AppContext;

use crate::workbench::activity::Activity;
use crate::workbench::state::WorkbenchState;
use crate::workbench::FileLoadPending;

/// Usable inner width (respects scroll bars and margins).
pub(crate) fn panel_width(ui: &Ui) -> f32 {
    ui.available_width().max(1.0)
}

/// Renders the sidebar for the current activity.
pub fn sidebar(
    ui: &mut Ui,
    state: &mut WorkbenchState,
    app_ctx: &AppContext,
    file_pending: &mut Option<FileLoadPending>,
) {
    ui.set_min_width(ui.available_width());
    ui.set_min_height(ui.available_height());

    section_heading(ui, state.activity.tooltip());

    if state.activity == Activity::SourceControl {
        source_control::show(
            ui,
            &mut state.source_control,
            &state.project,
            &mut state.editor,
            file_pending,
        );
        return;
    }

    if state.activity == Activity::Issues {
        issues::show(
            ui,
            &mut state.issues,
            &state.project,
            app_ctx,
            &mut state.editor,
            file_pending,
        );
        return;
    }

    let scroll_height = ui.available_height().max(0.0);
    ScrollArea::vertical()
        .id_salt((
            "kiwi-sidebar",
            format!("{:?}", state.activity),
            state.project.root.display().to_string(),
        ))
        .auto_shrink([false; 2])
        .max_height(scroll_height)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            match state.activity {
                Activity::Explorer => explorer::show(
                    ui,
                    &mut state.explorer,
                    &state.project,
                    &state.files,
                    &mut state.editor,
                    file_pending,
                ),
                Activity::Search => search::show(ui, &mut state.search_query),
                Activity::SourceControl => unreachable!(),
                Activity::Issues => unreachable!(),
                Activity::Tasks => tasks::show(ui),
                Activity::Agent => agent::show(ui, state, app_ctx),
                Activity::Tools => tools::show(ui),
                Activity::Extensions => extensions::show(ui),
            }
        });
}

/// Shared section title used by sidebar and AI panel.
pub fn section_heading(ui: &mut Ui, title: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(title).strong().size(11.0));
    ui.separator();
}
