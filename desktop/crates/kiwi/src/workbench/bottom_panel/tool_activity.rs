//! Tool activity bottom panel.

use egui::{RichText, Ui};

use crate::workbench::state::{ToolActivityEntry, WorkbenchState};

/// Renders agent tool activity from the current session.
pub fn show(ui: &mut Ui, state: &WorkbenchState) {
    if state.tool_activity.is_empty() {
        ui.label(
            RichText::new("No tool activity.")
                .monospace()
                .size(12.0),
        );
        return;
    }

    for entry in &state.tool_activity {
        render_entry(ui, entry);
        ui.add_space(6.0);
    }
}

fn render_entry(ui: &mut Ui, entry: &ToolActivityEntry) {
    let prefix = if entry.running { "🔧" } else { "✓" };
    ui.label(
        RichText::new(format!("{prefix} {}", entry.tool))
            .strong()
            .monospace()
            .size(12.0),
    );
    ui.label(
        RichText::new(&entry.detail)
            .weak()
            .monospace()
            .size(11.0),
    );
}
