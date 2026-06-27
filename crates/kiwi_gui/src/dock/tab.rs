//! GUI dock tab identifiers (SPEC-022 / ADR-022).
//!
//! GUI-only; not used by TUI navigation (SPEC-004). ADR-022 mentions `AiChat` as an
//! evolution of Agent — v1 uses a single [`KiwiTab::Agent`] PTY tab.

/// Dock panel tab key for [`egui_dock::DockState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)] // non-factory variants open via View menu (#185) and drag-add
pub enum KiwiTab {
    Explorer,
    GitStatus,
    GitDiff,
    GitLog,
    GitHubIssues,
    GitHubPrs,
    Preview,
    Search,
    Terminal,
    Agent,
    Config,
    Logs,
}

impl KiwiTab {
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Explorer => "Explorer",
            Self::GitStatus => "Git Status",
            Self::GitDiff => "Diff",
            Self::GitLog => "Git Log",
            Self::GitHubIssues => "Issues",
            Self::GitHubPrs => "Pull Requests",
            Self::Preview => "Preview",
            Self::Search => "Search",
            Self::Terminal => "Terminal",
            Self::Agent => "Agent",
            Self::Config => "Settings",
            Self::Logs => "Logs",
        }
    }

    #[must_use]
    pub const fn closable(self) -> bool {
        true
    }

    /// All tab variants (SPEC-022 tab metadata table).
    #[must_use]
    #[allow(dead_code)] // View menu tab picker (#185)
    pub const fn all_variants() -> &'static [Self] {
        &[
            Self::Explorer,
            Self::GitStatus,
            Self::GitDiff,
            Self::GitLog,
            Self::GitHubIssues,
            Self::GitHubPrs,
            Self::Preview,
            Self::Search,
            Self::Terminal,
            Self::Agent,
            Self::Config,
            Self::Logs,
        ]
    }

    /// Tabs opened on first run before persistence (#185 adds split layout).
    #[must_use]
    pub const fn factory_tabs() -> &'static [Self] {
        &[
            Self::Explorer,
            Self::GitStatus,
            Self::GitHubIssues,
            Self::Agent,
            Self::Terminal,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_have_unique_titles() {
        let variants = KiwiTab::all_variants();
        assert_eq!(variants.len(), 12);
        let mut titles = variants.iter().map(|tab| tab.title()).collect::<Vec<_>>();
        titles.sort_unstable();
        titles.dedup();
        assert_eq!(titles.len(), variants.len());
    }

    #[test]
    fn factory_tabs_are_subset_of_all_variants() {
        for tab in KiwiTab::factory_tabs() {
            assert!(KiwiTab::all_variants().contains(tab));
        }
    }

    #[test]
    fn every_tab_is_closable_in_v1() {
        for tab in KiwiTab::all_variants() {
            assert!(tab.closable());
        }
    }
}
