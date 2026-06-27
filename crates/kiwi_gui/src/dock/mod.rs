//! egui_dock shell adapter (SPEC-022 / ADR-022).

mod actions;
mod context;
mod layout;
mod panels;
mod persistence;
mod region;
mod tab;
mod viewer;

pub use context::PanelContext;
pub use layout::initial_dock_state;
pub use panels::explorer_keyboard_action;
pub use persistence::{restore_dock, snapshot_from_dock};
pub use tab::KiwiTab;

use std::collections::HashMap;

use egui::Ui;
use egui_dock::{DockArea, DockState, Node, Style};

use actions::TabActions;
use region::DockRegion;
use viewer::KiwiTabViewer;

/// Owns the dock tree and renders it inside a parent [`Ui`].
pub struct DockShell {
    dock_state: DockState<KiwiTab>,
    last_region: HashMap<KiwiTab, DockRegion>,
}

impl DockShell {
    #[must_use]
    pub fn new() -> Self {
        Self {
            dock_state: initial_dock_state(),
            last_region: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_state(dock_state: DockState<KiwiTab>) -> Self {
        Self {
            dock_state,
            last_region: HashMap::new(),
        }
    }

    pub fn render(&mut self, ui: &mut Ui, ctx: PanelContext<'_>) {
        let mut viewer = KiwiTabViewer { ctx };
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut viewer);
    }

    /// Active tab in the focused dock leaf, if any.
    #[must_use]
    pub fn focused_tab(&self) -> Option<KiwiTab> {
        let (surface, node_index) = self.dock_state.focused_leaf()?;
        match &self.dock_state[surface][node_index] {
            Node::Leaf { tabs, active, .. } => tabs.get(active.0).copied(),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_tab_open(&self, tab: KiwiTab) -> bool {
        self.dock_state.find_main_surface_tab(&tab).is_some()
    }

    pub fn show_tab(&mut self, tab: KiwiTab) {
        self.actions_mut().show_tab(tab);
    }

    pub fn close_tab(&mut self, tab: KiwiTab) {
        self.actions_mut().close_tab(tab);
    }

    pub fn reset_layout(&mut self) {
        self.dock_state = initial_dock_state();
        self.last_region.clear();
    }

    #[allow(dead_code)] // tests
    pub fn dock_state(&self) -> &DockState<KiwiTab> {
        &self.dock_state
    }

    fn actions_mut(&mut self) -> TabActions<'_> {
        TabActions {
            dock_state: &mut self.dock_state,
            last_region: &mut self.last_region,
        }
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

    #[test]
    fn reset_layout_restores_factory_tabs() {
        let mut shell = DockShell::new();
        shell.close_tab(KiwiTab::Explorer);
        shell.close_tab(KiwiTab::Agent);
        assert!(!shell.is_tab_open(KiwiTab::Explorer));

        shell.reset_layout();
        assert!(shell.is_tab_open(KiwiTab::Explorer));
        assert!(shell.is_tab_open(KiwiTab::Agent));
        let tabs = tabs_in_dock(shell.dock_state());
        assert_eq!(tabs.len(), KiwiTab::factory_tabs().len());
    }
}
