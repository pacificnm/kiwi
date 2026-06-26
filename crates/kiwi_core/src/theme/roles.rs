use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticRole {
    Bg,
    Fg,
    Border,
    Accent,
    Muted,
    Selection,
    GitAdded,
    GitModified,
    GitDeleted,
    GitUntracked,
    IssueOpen,
    IssueInProgress,
    IssueClosed,
    PrOpen,
    PrDraft,
    PrMerged,
    PrClosed,
    AgentThinking,
    AgentExecuting,
    AgentSuccess,
    AgentError,
    AgentWarning,
    FileDir,
    FileSource,
    FileScript,
    FileMarkup,
    FileConfig,
    FileData,
    FileMedia,
    FileOther,
}

impl SemanticRole {
    pub const ALL: [Self; 30] = [
        Self::Bg,
        Self::Fg,
        Self::Border,
        Self::Accent,
        Self::Muted,
        Self::Selection,
        Self::GitAdded,
        Self::GitModified,
        Self::GitDeleted,
        Self::GitUntracked,
        Self::IssueOpen,
        Self::IssueInProgress,
        Self::IssueClosed,
        Self::PrOpen,
        Self::PrDraft,
        Self::PrMerged,
        Self::PrClosed,
        Self::AgentThinking,
        Self::AgentExecuting,
        Self::AgentSuccess,
        Self::AgentError,
        Self::AgentWarning,
        Self::FileDir,
        Self::FileSource,
        Self::FileScript,
        Self::FileMarkup,
        Self::FileConfig,
        Self::FileData,
        Self::FileMedia,
        Self::FileOther,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bg => "bg",
            Self::Fg => "fg",
            Self::Border => "border",
            Self::Accent => "accent",
            Self::Muted => "muted",
            Self::Selection => "selection",
            Self::GitAdded => "git_added",
            Self::GitModified => "git_modified",
            Self::GitDeleted => "git_deleted",
            Self::GitUntracked => "git_untracked",
            Self::IssueOpen => "issue_open",
            Self::IssueInProgress => "issue_in_progress",
            Self::IssueClosed => "issue_closed",
            Self::PrOpen => "pr_open",
            Self::PrDraft => "pr_draft",
            Self::PrMerged => "pr_merged",
            Self::PrClosed => "pr_closed",
            Self::AgentThinking => "agent_thinking",
            Self::AgentExecuting => "agent_executing",
            Self::AgentSuccess => "agent_success",
            Self::AgentError => "agent_error",
            Self::AgentWarning => "agent_warning",
            Self::FileDir => "file_dir",
            Self::FileSource => "file_source",
            Self::FileScript => "file_script",
            Self::FileMarkup => "file_markup",
            Self::FileConfig => "file_config",
            Self::FileData => "file_data",
            Self::FileMedia => "file_media",
            Self::FileOther => "file_other",
        }
    }
}

impl FromStr for SemanticRole {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        SemanticRole::ALL
            .into_iter()
            .find(|role| role.as_str() == value)
            .ok_or(())
    }
}

impl fmt::Display for SemanticRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
