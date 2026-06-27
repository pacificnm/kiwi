//! Initial dock tree before workspace persistence (#186).

use egui_dock::{DockState, NodeIndex};

use super::tab::KiwiTab;

/// Left column width on first run (~22%, gui-layout.md).
pub(crate) const DEFAULT_LEFT_FRACTION: f32 = 0.22;
/// Top area height before bottom terminal strip (~72% → ~28% bottom).
pub(crate) const DEFAULT_TOP_FRACTION: f32 = 0.72;

/// ADR-022 three-region factory layout: left stack, center Agent, bottom Terminal.
#[must_use]
pub fn initial_dock_state() -> DockState<KiwiTab> {
    let mut dock = DockState::new(vec![KiwiTab::Agent]);
    let surface = dock.main_surface_mut();

    let [_center, _left] = surface.split_left(
        NodeIndex::root(),
        DEFAULT_LEFT_FRACTION,
        vec![KiwiTab::Explorer, KiwiTab::GitStatus, KiwiTab::GitHubIssues],
    );

    let [_top, _bottom] = surface.split_below(
        NodeIndex::root(),
        DEFAULT_TOP_FRACTION,
        vec![KiwiTab::Terminal],
    );

    debug_assert_eq!(dock.iter_all_tabs().count(), KiwiTab::factory_tabs().len());

    dock
}

#[cfg(test)]
pub(crate) fn tabs_in_dock(dock: &DockState<KiwiTab>) -> Vec<KiwiTab> {
    dock.iter_all_tabs().map(|(_, tab)| *tab).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::region::find_leaf_for_region;
    use crate::dock::region::DockRegion;

    #[test]
    fn initial_dock_contains_factory_tabs() {
        let dock = initial_dock_state();
        let tabs = tabs_in_dock(&dock);
        for expected in KiwiTab::factory_tabs() {
            assert!(tabs.contains(expected), "missing factory tab {expected:?}");
        }
        assert_eq!(tabs.len(), KiwiTab::factory_tabs().len());
    }

    #[test]
    fn initial_dock_splits_into_three_regions() {
        let dock = initial_dock_state();

        let left = find_leaf_for_region(&dock, DockRegion::Left).expect("left leaf");
        let center = find_leaf_for_region(&dock, DockRegion::Center).expect("center leaf");
        let bottom = find_leaf_for_region(&dock, DockRegion::Bottom).expect("bottom leaf");

        assert_ne!(left, center);
        assert_ne!(center, bottom);
        assert_ne!(left, bottom);
    }

    #[test]
    fn left_stack_holds_nav_tabs() {
        let dock = initial_dock_state();
        let left = find_leaf_for_region(&dock, DockRegion::Left).expect("left leaf");
        let surface = dock.main_surface();

        let left_tabs = surface[left].tabs().expect("left node is leaf").to_vec();

        assert!(left_tabs.contains(&KiwiTab::Explorer));
        assert!(left_tabs.contains(&KiwiTab::GitStatus));
        assert!(left_tabs.contains(&KiwiTab::GitHubIssues));
    }

    #[test]
    fn center_leaf_holds_agent() {
        let dock = initial_dock_state();
        let center = find_leaf_for_region(&dock, DockRegion::Center).expect("center leaf");
        let tabs = dock.main_surface()[center]
            .tabs()
            .expect("center node is leaf");
        assert!(tabs.contains(&KiwiTab::Agent));
    }

    #[test]
    fn bottom_leaf_holds_terminal() {
        let dock = initial_dock_state();
        let bottom = find_leaf_for_region(&dock, DockRegion::Bottom).expect("bottom leaf");
        let tabs = dock.main_surface()[bottom]
            .tabs()
            .expect("bottom node is leaf");
        assert_eq!(tabs, &[KiwiTab::Terminal]);
    }
}
