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

/// Inner width for sidebar scroll content (panel default minus frame padding).
pub(crate) const SIDEBAR_INNER_WIDTH: f32 = 236.0;

/// Renders the sidebar for the current activity.
pub fn sidebar(ui: &mut Ui, state: &mut WorkbenchState, app_ctx: &AppContext) {
    section_heading(ui, state.activity.tooltip());

    ScrollArea::vertical()
        .id_salt(("kiwi-sidebar", format!("{:?}", state.activity)))
        .auto_shrink([true; 2])
        .show(ui, |ui| {
            ui.set_width(SIDEBAR_INNER_WIDTH);
            match state.activity {
                Activity::Explorer => explorer::show(ui),
                Activity::Search => search::show(ui, &mut state.search_query),
                Activity::SourceControl => source_control::show(ui),
                Activity::Issues => issues::show(ui),
                Activity::Tasks => tasks::show(ui),
                Activity::Agent => agent::show(ui, &mut state.agent, app_ctx),
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
