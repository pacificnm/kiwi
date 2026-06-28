#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LeftNavTab {
    #[default]
    Files,
    Git,
    Gh,
    Search,
}

impl LeftNavTab {
    #[cfg_attr(not(test), allow(dead_code))]
    pub const ALL: [Self; 4] = [Self::Files, Self::Git, Self::Gh, Self::Search];

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Files => 0,
            Self::Git => 1,
            Self::Gh => 2,
            Self::Search => 3,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Git => "Git",
            Self::Gh => "GH",
            Self::Search => "Search",
        }
    }

    #[must_use]
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Files),
            1 => Some(Self::Git),
            2 => Some(Self::Gh),
            3 => Some(Self::Search),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MainTab {
    #[default]
    Agent,
    Issues,
    Branches,
    Prs,
    Diff,
    Preview,
    Logs,
    Settings,
    Plugins,
}

impl MainTab {
    #[cfg_attr(not(test), allow(dead_code))]
    pub const ALL: [Self; 9] = [
        Self::Agent,
        Self::Issues,
        Self::Branches,
        Self::Prs,
        Self::Diff,
        Self::Preview,
        Self::Logs,
        Self::Settings,
        Self::Plugins,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Agent => 0,
            Self::Issues => 1,
            Self::Branches => 2,
            Self::Prs => 3,
            Self::Diff => 4,
            Self::Preview => 5,
            Self::Logs => 6,
            Self::Settings => 7,
            Self::Plugins => 8,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::Issues => "Issues",
            Self::Branches => "Branches",
            Self::Prs => "PRs",
            Self::Diff => "Diff",
            Self::Preview => "Preview",
            Self::Logs => "Logs",
            Self::Settings => "Settings",
            Self::Plugins => "Plugins",
        }
    }

    #[must_use]
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Agent),
            1 => Some(Self::Issues),
            2 => Some(Self::Branches),
            3 => Some(Self::Prs),
            4 => Some(Self::Diff),
            5 => Some(Self::Preview),
            6 => Some(Self::Logs),
            7 => Some(Self::Settings),
            8 => Some(Self::Plugins),
            _ => None,
        }
    }

    /// Left nav tab to activate when this main tab is selected (SPEC-004 pairing).
    #[must_use]
    pub const fn paired_left_tab(self) -> Option<LeftNavTab> {
        match self {
            Self::Issues | Self::Prs | Self::Branches => Some(LeftNavTab::Gh),
            Self::Preview => Some(LeftNavTab::Files),
            Self::Diff => Some(LeftNavTab::Git),
            Self::Agent | Self::Logs | Self::Settings | Self::Plugins => None,
        }
    }
}

pub const LEFT_TAB_LABELS: [&str; 4] = [
    LeftNavTab::Files.label(),
    LeftNavTab::Git.label(),
    LeftNavTab::Gh.label(),
    LeftNavTab::Search.label(),
];

pub const MAIN_TAB_LABELS: [&str; 9] = [
    MainTab::Agent.label(),
    MainTab::Issues.label(),
    MainTab::Branches.label(),
    MainTab::Prs.label(),
    MainTab::Diff.label(),
    MainTab::Preview.label(),
    MainTab::Logs.label(),
    MainTab::Settings.label(),
    MainTab::Plugins.label(),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_tab_paired_left_tab_matches_spec() {
        assert_eq!(MainTab::Issues.paired_left_tab(), Some(LeftNavTab::Gh));
        assert_eq!(MainTab::Prs.paired_left_tab(), Some(LeftNavTab::Gh));
        assert_eq!(MainTab::Branches.paired_left_tab(), Some(LeftNavTab::Gh));
        assert_eq!(MainTab::Preview.paired_left_tab(), Some(LeftNavTab::Files));
        assert_eq!(MainTab::Diff.paired_left_tab(), Some(LeftNavTab::Git));
        assert_eq!(MainTab::Agent.paired_left_tab(), None);
        assert_eq!(MainTab::Logs.paired_left_tab(), None);
        assert_eq!(MainTab::Settings.paired_left_tab(), None);
        assert_eq!(MainTab::Plugins.paired_left_tab(), None);
    }

    #[test]
    fn plugins_tab_has_correct_index_and_label() {
        assert_eq!(MainTab::Plugins.index(), 8);
        assert_eq!(MainTab::Plugins.label(), "Plugins");
        assert_eq!(MainTab::from_index(8), Some(MainTab::Plugins));
    }

    #[test]
    fn main_tab_all_and_labels_are_in_sync() {
        assert_eq!(MainTab::ALL.len(), MAIN_TAB_LABELS.len());
        for (tab, label) in MainTab::ALL.iter().zip(MAIN_TAB_LABELS.iter()) {
            assert_eq!(tab.label(), *label);
        }
    }

    #[test]
    fn left_tabs_match_spec_order() {
        assert_eq!(LeftNavTab::Files.label(), "Files");
        assert_eq!(LeftNavTab::Search.index(), 3);
    }

    #[test]
    fn left_tab_constants_match_all_variants() {
        assert_eq!(LeftNavTab::ALL.len(), LEFT_TAB_LABELS.len());
        for (tab, label) in LeftNavTab::ALL.iter().zip(LEFT_TAB_LABELS.iter()) {
            assert_eq!(tab.label(), *label);
        }
    }
}
