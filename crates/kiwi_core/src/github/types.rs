pub const ISSUE_LIST_CACHE_SECS: u64 = 60;
pub const PR_LIST_CACHE_SECS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueState {
    Open,
    Closed,
}

impl IssueState {
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw {
            "CLOSED" => Self::Closed,
            _ => Self::Open,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub number: u32,
    pub title: String,
    pub state: IssueState,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueListLoadResult {
    pub issues: Vec<Issue>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrState {
    Open,
    Draft,
    Merged,
    Closed,
}

impl PrState {
    #[must_use]
    pub fn parse(raw: &str, is_draft: bool) -> Self {
        if is_draft {
            return Self::Draft;
        }

        match raw {
            "MERGED" => Self::Merged,
            "CLOSED" => Self::Closed,
            _ => Self::Open,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Draft => "draft",
            Self::Merged => "merged",
            Self::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequest {
    pub number: u32,
    pub title: String,
    pub state: PrState,
    pub author: String,
    pub is_draft: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrListLoadResult {
    pub prs: Vec<PullRequest>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitHubAuthErrorKind {
    NotInstalled,
    NotAuthenticated,
    CommandFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubAuthCheckResult {
    pub auth_ok: bool,
    pub error_kind: Option<GitHubAuthErrorKind>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct IssueComment {
    pub author: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueDetail {
    pub number: u32,
    pub title: String,
    pub state: IssueState,
    pub author: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub body: Option<String>,
    pub display_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueDetailLoadResult {
    pub detail: Option<IssueDetail>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrDetail {
    pub number: u32,
    pub title: String,
    pub state: PrState,
    pub author: String,
    pub display_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrDetailLoadResult {
    pub detail: Option<PrDetail>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueActionResult {
    pub success: bool,
    pub error: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueCreateRequest {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueCreateResult {
    pub result: IssueActionResult,
    pub number: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitHubBrowserTarget {
    Issue(u32),
    PullRequest(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoLabel {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoLabelsLoadResult {
    pub labels: Vec<RepoLabel>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoMilestone {
    pub number: u32,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoMilestonesLoadResult {
    pub milestones: Vec<RepoMilestone>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestonePickerState {
    pub issue_number: u32,
    pub milestones: Vec<RepoMilestone>,
    pub cursor: usize,
    pub loading: bool,
    pub applying: bool,
    pub error: Option<String>,
}

impl MilestonePickerState {
    pub fn new(issue_number: u32) -> Self {
        Self {
            issue_number,
            milestones: Vec::new(),
            cursor: 0,
            loading: true,
            applying: false,
            error: None,
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        if self.milestones.is_empty() {
            self.cursor = 0;
            return;
        }

        let len = self.milestones.len() as i32;
        let next = (self.cursor as i32 + delta).rem_euclid(len);
        self.cursor = usize::try_from(next).unwrap_or(0);
    }

    #[must_use]
    pub fn selected_milestone(&self) -> Option<&RepoMilestone> {
        self.milestones.get(self.cursor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelPickerState {
    pub issue_number: u32,
    pub existing_labels: Vec<String>,
    pub labels: Vec<RepoLabel>,
    pub cursor: usize,
    pub selected: Vec<bool>,
    pub loading: bool,
    pub applying: bool,
    pub error: Option<String>,
}

impl LabelPickerState {
    pub fn new(issue_number: u32, existing_labels: Vec<String>) -> Self {
        Self {
            issue_number,
            existing_labels,
            labels: Vec::new(),
            cursor: 0,
            selected: Vec::new(),
            loading: true,
            applying: false,
            error: None,
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        if self.labels.is_empty() {
            self.cursor = 0;
            return;
        }

        let len = self.labels.len() as i32;
        let next = (self.cursor as i32 + delta).rem_euclid(len);
        self.cursor = usize::try_from(next).unwrap_or(0);
    }

    pub fn toggle_cursor(&mut self) {
        if let Some(selected) = self.selected.get_mut(self.cursor) {
            *selected = !*selected;
        }
    }

    pub fn labels_to_add(&self) -> Vec<String> {
        self.labels
            .iter()
            .zip(self.selected.iter())
            .filter(|(label, selected)| **selected && !self.existing_labels.contains(&label.name))
            .map(|(label, _)| label.name.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrCreateRequest {
    pub title: String,
    pub body: String,
    pub base: Option<String>,
}

impl GitHubBrowserTarget {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Issue(_) => "issue",
            Self::PullRequest(_) => "pull request",
        }
    }

    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::Issue(number) | Self::PullRequest(number) => number,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitHubLeftPane {
    #[default]
    Issues,
    Prs,
    Branches,
}

impl GitHubLeftPane {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Issues => 0,
            Self::Prs => 1,
            Self::Branches => 2,
        }
    }
}
