//! egui_dock shell adapter (SPEC-022 / ADR-022).

mod context;
mod layout;
mod panels;
mod tab;
mod viewer;

pub use context::PanelContext;
pub use layout::initial_dock_state;
pub use tab::KiwiTab;

use egui::Ui;
use egui_dock::{DockArea, DockState, Style};

use viewer::KiwiTabViewer;

/// Owns the dock tree and renders it inside a parent [`Ui`].
pub struct DockShell {
    dock_state: DockState<KiwiTab>,
}

impl DockShell {
    #[must_use]
    pub fn new() -> Self {
        Self {
            dock_state: initial_dock_state(),
        }
    }

    #[must_use]
    #[allow(dead_code)] // workspace restore (#186)
    pub fn with_state(dock_state: DockState<KiwiTab>) -> Self {
        Self { dock_state }
    }

    pub fn render(&mut self, ui: &mut Ui, ctx: PanelContext<'_>) {
        let mut viewer = KiwiTabViewer { ctx };
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut viewer);
    }

    #[allow(dead_code)] // tests and persistence (#186)
    pub fn dock_state(&self) -> &DockState<KiwiTab> {
        &self.dock_state
    }
}

impl Default for DockShell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::layout::tabs_in_dock;

    #[test]
    fn default_shell_opens_factory_tabs() {
        let shell = DockShell::new();
        let tabs = tabs_in_dock(shell.dock_state());
        assert_eq!(tabs.len(), KiwiTab::factory_tabs().len());
    }
}
