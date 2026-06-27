//! Tab show/hide/focus helpers for the View menu (SPEC-022 / #185).

use std::collections::HashMap;

use egui_dock::{DockState, SurfaceIndex};

use super::region::{
    find_leaf_for_region, focus_tab, push_tab_to_leaf, region_of_leaf, DockRegion,
};
use super::tab::KiwiTab;

pub(crate) struct TabActions<'a> {
    pub dock_state: &'a mut DockState<KiwiTab>,
    pub last_region: &'a mut HashMap<KiwiTab, DockRegion>,
}

impl TabActions<'_> {
    #[must_use]
    pub fn is_open(&self, tab: KiwiTab) -> bool {
        self.dock_state.find_main_surface_tab(&tab).is_some()
    }

    pub fn focus_tab(&mut self, tab: KiwiTab) {
        if let Some((node, tab_index)) = self.dock_state.find_main_surface_tab(&tab) {
            focus_tab(self.dock_state, (node, tab_index));
        }
    }

    pub fn close_tab(&mut self, tab: KiwiTab) {
        let Some((node, tab_index)) = self.dock_state.find_main_surface_tab(&tab) else {
            return;
        };

        let region = region_of_leaf(self.dock_state.main_surface(), node);
        self.last_region.insert(tab, region);

        self.dock_state
            .remove_tab((SurfaceIndex::main(), node, tab_index));
    }

    pub fn show_tab(&mut self, tab: KiwiTab) {
        if self.is_open(tab) {
            self.focus_tab(tab);
            return;
        }

        let region = self
            .last_region
            .get(&tab)
            .copied()
            .unwrap_or_else(|| tab.default_region());

        if let Some(node) = find_leaf_for_region(self.dock_state, region) {
            push_tab_to_leaf(self.dock_state.main_surface_mut(), node, tab);
            if let Some((node, tab_index)) = self.dock_state.find_main_surface_tab(&tab) {
                focus_tab(self.dock_state, (node, tab_index));
            }
            return;
        }

        self.dock_state.push_to_focused_leaf(tab);
    }

    #[allow(dead_code)] // unit tests
    pub fn toggle_tab(&mut self, tab: KiwiTab) {
        if self.is_open(tab) {
            self.close_tab(tab);
        } else {
            self.show_tab(tab);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::layout::initial_dock_state;

    #[test]
    fn close_and_reopen_git_status_in_left_region() {
        let mut dock = initial_dock_state();
        let mut last_region = HashMap::new();
        let mut actions = TabActions {
            dock_state: &mut dock,
            last_region: &mut last_region,
        };

        assert!(actions.is_open(KiwiTab::GitStatus));
        actions.close_tab(KiwiTab::GitStatus);
        assert!(!actions.is_open(KiwiTab::GitStatus));

        actions.show_tab(KiwiTab::GitStatus);
        assert!(actions.is_open(KiwiTab::GitStatus));

        let left = find_leaf_for_region(&dock, DockRegion::Left).expect("left leaf");
        let (node, _) = dock
            .find_main_surface_tab(&KiwiTab::GitStatus)
            .expect("git status open");
        assert_eq!(node, left);
    }

    #[test]
    fn show_diff_opens_in_center_when_closed() {
        let mut dock = initial_dock_state();
        let mut last_region = HashMap::new();
        let mut actions = TabActions {
            dock_state: &mut dock,
            last_region: &mut last_region,
        };

        assert!(!actions.is_open(KiwiTab::GitDiff));
        actions.show_tab(KiwiTab::GitDiff);
        assert!(actions.is_open(KiwiTab::GitDiff));

        let center = find_leaf_for_region(&dock, DockRegion::Center).expect("center leaf");
        let (node, _) = dock
            .find_main_surface_tab(&KiwiTab::GitDiff)
            .expect("diff open");
        assert_eq!(node, center);
    }

    #[test]
    fn toggle_tab_closes_then_opens() {
        let mut dock = initial_dock_state();
        let mut last_region = HashMap::new();
        let mut actions = TabActions {
            dock_state: &mut dock,
            last_region: &mut last_region,
        };

        actions.toggle_tab(KiwiTab::Agent);
        assert!(!actions.is_open(KiwiTab::Agent));
        actions.toggle_tab(KiwiTab::Agent);
        assert!(actions.is_open(KiwiTab::Agent));
    }
}
