//! GUI dock tab identifiers (SPEC-022 / ADR-022).
//!
//! GUI-only; not used by TUI navigation (SPEC-004). ADR-022 mentions `AiChat` as an
//! evolution of Agent — v1 uses a single [`KiwiTab::Agent`] PTY tab.

/// Dock panel tab key for [`egui_dock::DockState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum KiwiTab {
    Explorer,
    GitStatus,
    GitDiff,
    GitLog,
    GitHubIssues,
    Issues,
    GitHubPrs,
    Preview,
    Search,
    Terminal,
    Agent,
    Config,
    Logs,
    Plugins,
}

impl KiwiTab {
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Explorer => "Files",
            Self::GitStatus => "Git",
            Self::GitDiff => "Diff",
            Self::GitLog => "Git Log",
            Self::GitHubIssues => "GH",
            Self::Issues => "Issues",
            Self::GitHubPrs => "PRs",
            Self::Preview => "Preview",
            Self::Search => "Search",
            Self::Terminal => "Terminal",
            Self::Agent => "Agent",
            Self::Config => "Settings",
            Self::Logs => "Logs",
            Self::Plugins => "Plugins",
        }
    }

    /// All tab variants (SPEC-022 tab metadata table).
    #[must_use]
    pub const fn all_variants() -> &'static [Self] {
        &[
            Self::Explorer,
            Self::GitStatus,
            Self::GitDiff,
            Self::GitLog,
            Self::GitHubIssues,
            Self::Issues,
            Self::GitHubPrs,
            Self::Preview,
            Self::Search,
            Self::Terminal,
            Self::Agent,
            Self::Config,
            Self::Logs,
            Self::Plugins,
        ]
    }

    /// Returns `true` for tabs whose panels are not yet implemented (show placeholder text).
    /// Used to hide stub tabs from the View menu (#278).
    #[must_use]
    pub const fn is_placeholder(self) -> bool {
        matches!(self, Self::Logs | Self::GitLog)
    }

    /// Factory-default open tabs (ADR-022); used by layout and tests.
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
        assert_eq!(variants.len(), 14);
        let mut titles = variants.iter().map(|tab| tab.title()).collect::<Vec<_>>();
        titles.sort_unstable();
        titles.dedup();
        assert_eq!(titles.len(), variants.len());
    }

    #[test]
    fn placeholder_tabs_are_logs_and_gitlog() {
        assert!(KiwiTab::Logs.is_placeholder());
        assert!(KiwiTab::GitLog.is_placeholder());
        assert!(!KiwiTab::Config.is_placeholder());
        assert!(!KiwiTab::Explorer.is_placeholder());
        assert!(!KiwiTab::Terminal.is_placeholder());
    }

    #[test]
    fn factory_tabs_are_subset_of_all_variants() {
        for tab in KiwiTab::factory_tabs() {
            assert!(KiwiTab::all_variants().contains(tab));
        }
    }

    #[test]
    fn factory_tabs_include_explorer_git_and_agent() {
        let tabs = KiwiTab::factory_tabs();
        assert!(tabs.contains(&KiwiTab::Explorer));
        assert!(tabs.contains(&KiwiTab::GitStatus));
        assert!(tabs.contains(&KiwiTab::Agent));
    }
}
