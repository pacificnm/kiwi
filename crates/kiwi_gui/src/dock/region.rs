//! Default dock regions for tab placement (SPEC-022 / ADR-022).

use egui_dock::{DockState, NodeIndex, SurfaceIndex, TabIndex, Tree};

use super::tab::KiwiTab;

/// Factory layout region for a tab strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DockRegion {
    Left,
    Center,
    Bottom,
}

impl KiwiTab {
    /// Preferred region when (re)opening a tab (SPEC-022).
    #[must_use]
    pub const fn default_region(self) -> DockRegion {
        match self {
            Self::Explorer | Self::GitStatus | Self::GitHubIssues => DockRegion::Left,
            Self::Terminal => DockRegion::Bottom,
            Self::Agent
            | Self::Issues
            | Self::GitDiff
            | Self::GitLog
            | Self::GitHubPrs
            | Self::Preview
            | Self::Search
            | Self::Config
            | Self::Logs => DockRegion::Center,
        }
    }
}

impl DockRegion {
    const fn anchor_tab(self) -> KiwiTab {
        match self {
            Self::Left => KiwiTab::Explorer,
            Self::Center => KiwiTab::Agent,
            Self::Bottom => KiwiTab::Terminal,
        }
    }
}

/// Classify which region a leaf belongs to based on its open tabs.
#[must_use]
pub(crate) fn region_of_leaf(tree: &Tree<KiwiTab>, node: NodeIndex) -> DockRegion {
    if leaf_contains(tree, node, KiwiTab::Terminal) {
        return DockRegion::Bottom;
    }
    if leaf_contains(tree, node, KiwiTab::Explorer)
        || leaf_contains(tree, node, KiwiTab::GitStatus)
        || leaf_contains(tree, node, KiwiTab::GitHubIssues)
    {
        return DockRegion::Left;
    }
    DockRegion::Center
}

#[must_use]
pub(crate) fn find_leaf_for_region(
    dock: &DockState<KiwiTab>,
    region: DockRegion,
) -> Option<NodeIndex> {
    let anchor = region.anchor_tab();
    if let Some((node, _)) = dock.find_main_surface_tab(&anchor) {
        return Some(node);
    }

    for ((surface_index, node), tab) in dock.iter_all_tabs() {
        if surface_index.is_main() && tab.default_region() == region {
            return Some(node);
        }
    }
    None
}

pub(crate) fn push_tab_to_leaf(tree: &mut Tree<KiwiTab>, node: NodeIndex, tab: KiwiTab) {
    use egui_dock::Node;

    match &mut tree[node] {
        Node::Leaf { tabs, active, .. } => {
            *active = TabIndex(tabs.len());
            tabs.push(tab);
            tree.set_focused_node(node);
        }
        _ => tree.push_to_first_leaf(tab),
    }
}

pub(crate) fn focus_tab(dock: &mut DockState<KiwiTab>, (node, tab_index): (NodeIndex, TabIndex)) {
    dock.set_active_tab((SurfaceIndex::main(), node, tab_index));
    dock.set_focused_node_and_surface((SurfaceIndex::main(), node));
}

fn leaf_contains(tree: &Tree<KiwiTab>, node: NodeIndex, needle: KiwiTab) -> bool {
    tree[node].tabs().is_some_and(|tabs| tabs.contains(&needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::layout::initial_dock_state;

    #[test]
    fn default_regions_match_spec() {
        assert_eq!(KiwiTab::Explorer.default_region(), DockRegion::Left);
        assert_eq!(KiwiTab::Agent.default_region(), DockRegion::Center);
        assert_eq!(KiwiTab::Terminal.default_region(), DockRegion::Bottom);
        assert_eq!(KiwiTab::GitDiff.default_region(), DockRegion::Center);
    }

    #[test]
    fn factory_layout_regions_are_discoverable() {
        let dock = initial_dock_state();
        assert!(find_leaf_for_region(&dock, DockRegion::Left).is_some());
        assert!(find_leaf_for_region(&dock, DockRegion::Center).is_some());
        assert!(find_leaf_for_region(&dock, DockRegion::Bottom).is_some());
    }
}
