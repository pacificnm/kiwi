//! Initial dock tree before workspace persistence (#186).

use egui_dock::DockState;

use super::tab::KiwiTab;

/// First-run dock: factory tabs in the root leaf (#185 adds ADR-022 split regions).
#[must_use]
pub fn initial_dock_state() -> DockState<KiwiTab> {
    DockState::new(KiwiTab::factory_tabs().to_vec())
}

#[cfg(test)]
pub(crate) fn tabs_in_dock(dock: &DockState<KiwiTab>) -> Vec<KiwiTab> {
    dock.iter_all_tabs().map(|(_, tab)| *tab).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_dock_contains_factory_tabs() {
        let dock = initial_dock_state();
        let tabs = tabs_in_dock(&dock);
        for expected in KiwiTab::factory_tabs() {
            assert!(tabs.contains(expected), "missing factory tab {expected:?}");
        }
    }

    #[test]
    fn initial_dock_has_at_least_one_tab() {
        let dock = initial_dock_state();
        assert!(!tabs_in_dock(&dock).is_empty());
    }
}
