#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LeftNavTab {
    #[default]
    Files,
    Git,
    Diff,
    Gh,
    Search,
}

impl LeftNavTab {
    #[cfg_attr(not(test), allow(dead_code))]
    pub const ALL: [Self; 5] = [Self::Files, Self::Git, Self::Diff, Self::Gh, Self::Search];

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Files => 0,
            Self::Git => 1,
            Self::Diff => 2,
            Self::Gh => 3,
            Self::Search => 4,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Git => "Git",
            Self::Diff => "Diff",
            Self::Gh => "GH",
            Self::Search => "Search",
        }
    }

    #[must_use]
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Files),
            1 => Some(Self::Git),
            2 => Some(Self::Diff),
            3 => Some(Self::Gh),
            4 => Some(Self::Search),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MainTab {
    #[default]
    Agent,
    Issues,
    Prs,
    Diff,
    Preview,
    Logs,
}

impl MainTab {
    #[cfg_attr(not(test), allow(dead_code))]
    pub const ALL: [Self; 6] = [
        Self::Agent,
        Self::Issues,
        Self::Prs,
        Self::Diff,
        Self::Preview,
        Self::Logs,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Agent => 0,
            Self::Issues => 1,
            Self::Prs => 2,
            Self::Diff => 3,
            Self::Preview => 4,
            Self::Logs => 5,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::Issues => "Issues",
            Self::Prs => "PRs",
            Self::Diff => "Diff",
            Self::Preview => "Preview",
            Self::Logs => "Logs",
        }
    }

    #[must_use]
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Agent),
            1 => Some(Self::Issues),
            2 => Some(Self::Prs),
            3 => Some(Self::Diff),
            4 => Some(Self::Preview),
            5 => Some(Self::Logs),
            _ => None,
        }
    }
}

pub const LEFT_TAB_LABELS: [&str; 5] = [
    LeftNavTab::Files.label(),
    LeftNavTab::Git.label(),
    LeftNavTab::Diff.label(),
    LeftNavTab::Gh.label(),
    LeftNavTab::Search.label(),
];

pub const MAIN_TAB_LABELS: [&str; 6] = [
    MainTab::Agent.label(),
    MainTab::Issues.label(),
    MainTab::Prs.label(),
    MainTab::Diff.label(),
    MainTab::Preview.label(),
    MainTab::Logs.label(),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_tabs_match_spec_order() {
        assert_eq!(LeftNavTab::Files.label(), "Files");
        assert_eq!(LeftNavTab::Search.index(), 4);
    }

    #[test]
    fn left_tab_constants_match_all_variants() {
        assert_eq!(LeftNavTab::ALL.len(), LEFT_TAB_LABELS.len());
        for (tab, label) in LeftNavTab::ALL.iter().zip(LEFT_TAB_LABELS.iter()) {
            assert_eq!(tab.label(), *label);
        }
    }
}
