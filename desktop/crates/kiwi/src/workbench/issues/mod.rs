//! GitHub issues sidebar and issue loading for editor tabs.

pub mod auth;
pub mod comment_modal;
mod github;
mod github_metadata;
mod github_pulls;
mod issue_metadata_modal;
mod labels_modal;
mod milestones_modal;
mod modal_frame;
mod pr_merge_modal;

use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::Receiver;

use nest_http_client::HttpClientService;

use crate::workbench::editor_files::FileLoadEvent;

pub use auth::load_token_from_config;
pub use comment_modal::{CommentModalAction, CommentModalState};
pub use github::{
    read_github_repo, GitHubVerifyEvent, IssueCommentCreateEvent, IssueCreateEvent,
    IssueSendToAgentEvent,
};
pub use github_metadata::MetadataEvent;
pub use issue_metadata_modal::{show_window as show_issue_metadata_modal, IssueMetadataModalAction, IssueMetadataModalState};
pub use labels_modal::{show_window as show_labels_modal, LabelsModalAction, LabelsModalState};
pub use milestones_modal::{
    show_window as show_milestones_modal, MilestonesModalAction, MilestonesModalState,
};
pub use github_pulls::{
    PullRequestDetail, PullRequestEvent, PullRequestListItem, spawn_pull_request_list,
    spawn_pull_request_load, spawn_pull_request_merge,
};
pub use pr_merge_modal::{show_window as show_pr_merge_modal, PrMergeModalAction, PrMergeModalState};

use github::{
    spawn_create_comment, spawn_create_issue, spawn_issue_for_agent, spawn_issue_list,
    spawn_issue_load, spawn_verify_token,
};
use github_metadata::{
    spawn_create_label, spawn_create_milestone, spawn_delete_label, spawn_delete_milestone,
    spawn_list_labels, spawn_list_milestones, spawn_load_issue_metadata, spawn_update_issue_metadata,
    spawn_update_label, spawn_update_milestone,
};

/// Issues sidebar sub-view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IssuesSidebarView {
    /// Open GitHub issues.
    #[default]
    Issues,
    /// Open pull requests.
    PullRequests,
}

/// One issue row in the sidebar list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueListItem {
    /// GitHub issue number.
    pub number: u64,
    /// Issue title.
    pub title: String,
    /// `open` or `closed`.
    pub state: String,
    /// GitHub username of the author.
    pub author: String,
    /// Link to the issue on GitHub.
    pub html_url: String,
}

/// Issue context loaded for the agent prompt.
#[derive(Debug, Clone)]
pub struct IssueAgentContext {
    /// GitHub issue number.
    pub number: u64,
    /// Issue title.
    pub title: String,
    /// Formatted issue body.
    pub content: String,
}

/// Result of loading the issue list.
#[derive(Debug)]
pub enum IssuesListEvent {
    /// Parsed repository and issues.
    Ready {
        /// `(owner, repo)` from origin.
        repo: (String, String),
        /// Open issues (pull requests excluded).
        issues: Vec<IssueListItem>,
    },
    /// Failed to load issues.
    Failed(String),
}

/// GitHub issues sidebar state.
#[derive(Debug)]
pub struct IssuesState {
    /// Repository parsed from `origin`, when known.
    pub repo: Option<(String, String)>,
    /// Loaded open issues.
    pub issues: Vec<IssueListItem>,
    /// Last error message.
    pub error: Option<String>,
    /// True while the issue list is loading.
    pub loading: bool,
    /// Token saved in Kiwi config (not env/`gh` fallbacks).
    pub stored_token: Option<String>,
    /// GitHub username from the last successful token verification.
    pub auth_login: Option<String>,
    /// Transient auth status shown in the sidebar and auth tab.
    pub auth_status: Option<String>,
    /// Last auth error message.
    pub auth_error: Option<String>,
    /// True while verifying a token.
    pub auth_verifying: bool,
    /// New comment modal state.
    pub comment_modal: CommentModalState,
    /// Repository labels manager modal.
    pub labels_modal: LabelsModalState,
    /// Repository milestones manager modal.
    pub milestones_modal: MilestonesModalState,
    /// Issue labels/milestone assign modal.
    pub issue_metadata_modal: IssueMetadataModalState,
    /// Set when a comment posts successfully; consumed to refresh the open issue tab.
    pub comment_posted_on: Option<u64>,
    /// Set when issue metadata updates; consumed to refresh the open issue tab.
    pub issue_updated_on: Option<u64>,
    /// Loaded issue waiting to be attached to the agent prompt.
    pub issue_sent_to_agent: Option<IssueAgentContext>,
    /// Active sidebar list (issues vs pull requests).
    pub sidebar_view: IssuesSidebarView,
    /// Loaded open pull requests.
    pub pull_requests: Vec<PullRequestListItem>,
    /// True while the pull request list is loading.
    pub pr_loading: bool,
    /// Cached pull request details keyed by number.
    pub pr_details: HashMap<u64, PullRequestDetail>,
    /// Merge pull request modal state.
    pub pr_merge_modal: PrMergeModalState,
    /// PR loaded into an editor tab; consumed by the workbench.
    pub pr_loaded: Option<(usize, PullRequestDetail)>,
    /// Set when a PR merges successfully; consumed to refresh tabs/lists.
    pub pr_merged_on: Option<u64>,
    list_pending: Option<Receiver<IssuesListEvent>>,
    verify_pending: Option<Receiver<GitHubVerifyEvent>>,
    comment_pending: Option<Receiver<IssueCommentCreateEvent>>,
    metadata_pending: Option<Receiver<MetadataEvent>>,
    agent_issue_pending: Option<Receiver<IssueSendToAgentEvent>>,
    pr_list_pending: Option<Receiver<PullRequestEvent>>,
    pr_pending: Option<Receiver<PullRequestEvent>>,
}

impl Default for IssuesState {
    fn default() -> Self {
        Self::new()
    }
}

impl IssuesState {
    /// Creates empty issues state.
    pub fn new() -> Self {
        Self {
            repo: None,
            issues: Vec::new(),
            error: None,
            loading: false,
            stored_token: None,
            auth_login: None,
            auth_status: None,
            auth_error: None,
            auth_verifying: false,
            comment_modal: CommentModalState::default(),
            labels_modal: LabelsModalState::default(),
            milestones_modal: MilestonesModalState::default(),
            issue_metadata_modal: IssueMetadataModalState::default(),
            comment_posted_on: None,
            issue_updated_on: None,
            issue_sent_to_agent: None,
            sidebar_view: IssuesSidebarView::Issues,
            pull_requests: Vec::new(),
            pr_loading: false,
            pr_details: HashMap::new(),
            pr_merge_modal: PrMergeModalState::default(),
            pr_loaded: None,
            pr_merged_on: None,
            list_pending: None,
            verify_pending: None,
            comment_pending: None,
            metadata_pending: None,
            agent_issue_pending: None,
            pr_list_pending: None,
            pr_pending: None,
        }
    }

    /// Returns the effective GitHub token (GitHub CLI, env, or saved config).
    pub fn token(&self) -> Option<String> {
        github::resolve_github_token(self.stored_token.as_deref())
    }

    /// Returns true when GitHub CLI or another token source is available.
    pub fn is_authenticated(&self) -> bool {
        github::gh_auth_available() || self.token().is_some()
    }

    /// Refreshes auth status from the GitHub CLI (`gh auth login` session).
    pub fn sync_gh_auth(&mut self) {
        if let Some(login) = github::gh_cli_login() {
            self.auth_login = Some(login.clone());
            self.auth_status = Some(format!("Signed in via GitHub CLI as @{login}"));
            self.auth_error = None;
            return;
        }

        if github::github_token_from_env().is_some() {
            self.auth_status = Some("Using GitHub token from environment".into());
            self.auth_error = None;
            return;
        }

        if self
            .stored_token
            .as_deref()
            .is_some_and(|token| !token.trim().is_empty())
        {
            self.auth_status = Some("Using GitHub token from Kiwi config".into());
            self.auth_error = None;
            return;
        }

        self.auth_login = None;
        self.auth_status = None;
        self.auth_error = Some("Run `gh auth login` to sign in to GitHub".into());
    }

    /// Updates the saved token and clears cached login until re-verified.
    pub fn set_stored_token(&mut self, token: Option<String>) {
        self.stored_token = token
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        self.auth_login = None;
        self.auth_error = None;
    }

    /// Returns true while a network request is in flight.
    pub fn busy(&self) -> bool {
        self.loading
            || self.list_pending.is_some()
            || self.auth_verifying
            || self.verify_pending.is_some()
            || self.comment_pending.is_some()
            || self.metadata_pending.is_some()
            || self.comment_modal.submitting
            || self.labels_modal.submitting
            || self.milestones_modal.submitting
            || self.issue_metadata_modal.submitting
            || self.agent_issue_pending.is_some()
            || self.pr_list_pending.is_some()
            || self.pr_pending.is_some()
            || self.pr_merge_modal.submitting
    }

    /// Opens the new-comment modal, optionally pre-filling the issue number.
    pub fn open_new_comment(&mut self, issue_number: Option<u64>) {
        self.comment_modal.open_with_issue(issue_number);
    }

    /// Opens the repository labels manager modal.
    pub fn open_manage_labels(&mut self) {
        self.labels_modal.open_list();
    }

    /// Opens the repository milestones manager modal.
    pub fn open_manage_milestones(&mut self) {
        self.milestones_modal.open_list();
    }

    /// Opens the issue labels/milestone assign modal.
    pub fn open_issue_metadata(&mut self, issue_number: u64) {
        self.issue_metadata_modal.open_for_issue(issue_number);
    }

    /// Loads a GitHub issue and queues it for the agent prompt.
    pub fn request_send_to_agent(
        &mut self,
        root: &Path,
        issue_number: u64,
        http: HttpClientService,
    ) {
        if self.agent_issue_pending.is_some() {
            return;
        }
        let Some((owner, repo)) = self
            .repo
            .clone()
            .or_else(|| read_github_repo(root).ok())
        else {
            self.error = Some("Could not resolve GitHub repository from origin".into());
            return;
        };
        self.error = None;
        self.auth_status = Some(format!("Loading issue #{issue_number} for agent…"));
        self.agent_issue_pending = Some(spawn_issue_for_agent(
            http,
            owner,
            repo,
            issue_number,
            self.token(),
        ));
    }

    /// Loads open pull requests for the workspace.
    pub fn request_pr_list(&mut self, root: &Path, http: HttpClientService) {
        if self.pr_list_pending.is_some() {
            return;
        }
        let Some((owner, repo)) = self
            .repo
            .clone()
            .or_else(|| read_github_repo(root).ok())
        else {
            return;
        };
        self.pr_loading = true;
        self.pr_list_pending = Some(spawn_pull_request_list(
            http,
            owner,
            repo,
            self.token(),
        ));
    }

    /// Loads one pull request into an editor tab.
    pub fn request_open_pr(
        &mut self,
        root: &Path,
        number: u64,
        tab_index: usize,
        http: HttpClientService,
    ) {
        if self.pr_pending.is_some() {
            return;
        }
        let Some((owner, repo)) = self
            .repo
            .clone()
            .or_else(|| read_github_repo(root).ok())
        else {
            self.error = Some("Could not resolve GitHub repository from origin".into());
            return;
        };
        self.pr_pending = Some(spawn_pull_request_load(
            http,
            owner,
            repo,
            number,
            tab_index,
            self.token(),
        ));
    }

    /// Opens the merge modal for a pull request.
    pub fn open_merge_pr(&mut self, number: u64) {
        if let Some(detail) = self.pr_details.get(&number) {
            self.pr_merge_modal.open_for_pr(
                detail.number,
                detail.title.clone(),
                detail.head_branch.clone(),
                detail.base_branch.clone(),
                detail.mergeable,
                detail.mergeable_state.clone(),
                detail.draft,
                detail.merged,
            );
            return;
        }
        if let Some(item) = self.pull_requests.iter().find(|pr| pr.number == number) {
            self.pr_merge_modal.open_for_pr(
                item.number,
                item.title.clone(),
                item.head_branch.clone(),
                item.base_branch.clone(),
                None,
                "unknown".into(),
                item.draft,
                false,
            );
        }
    }

    /// Merges a pull request via the GitHub API.
    pub fn request_merge_pr(
        &mut self,
        root: &Path,
        number: u64,
        merge_method: String,
        http: HttpClientService,
    ) {
        if self.pr_pending.is_some() {
            return;
        }
        let Some((owner, repo)) = self
            .repo
            .clone()
            .or_else(|| read_github_repo(root).ok())
        else {
            self.pr_merge_modal
                .fail("Could not resolve GitHub repository from origin".into());
            return;
        };
        self.pr_merge_modal.submitting = true;
        self.pr_merge_modal.error = None;
        self.pr_pending = Some(spawn_pull_request_merge(
            http,
            owner,
            repo,
            number,
            merge_method,
            self.token(),
        ));
    }

    /// Starts loading open GitHub issues for the workspace.
    pub fn request_list(&mut self, root: &Path, http: HttpClientService) {
        if self.list_pending.is_some() {
            return;
        }
        self.loading = true;
        self.error = None;
        self.list_pending = Some(spawn_issue_list(
            http,
            root.to_path_buf(),
            self.token(),
        ));
    }

    /// Verifies the current token against GitHub.
    pub fn request_verify(&mut self, http: HttpClientService) {
        let Some(token) = self.token() else {
            self.auth_error = Some("No GitHub token configured".into());
            return;
        };
        if self.verify_pending.is_some() {
            return;
        }
        self.auth_verifying = true;
        self.auth_error = None;
        self.auth_status = Some("Verifying token…".into());
        self.verify_pending = Some(spawn_verify_token(http, token));
    }

    /// Polls background channels; returns true when the UI should repaint.
    pub fn poll(&mut self) -> bool {
        let mut repaint = false;

        if let Some(rx) = self.metadata_pending.as_ref() {
            match rx.try_recv() {
                Ok(event) => {
                    self.handle_metadata_event(event);
                    self.metadata_pending = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.fail_open_modals("GitHub metadata request interrupted".into());
                    self.metadata_pending = None;
                    repaint = true;
                }
            }
        }

        if let Some(rx) = self.agent_issue_pending.as_ref() {
            match rx.try_recv() {
                Ok(IssueSendToAgentEvent::Ready {
                    number,
                    title,
                    content,
                }) => {
                    self.agent_issue_pending = None;
                    self.issue_sent_to_agent = Some(IssueAgentContext {
                        number,
                        title,
                        content,
                    });
                    self.auth_status =
                        Some(format!("Issue #{number} ready — attached to agent prompt"));
                    repaint = true;
                }
                Ok(IssueSendToAgentEvent::Failed { error }) => {
                    self.agent_issue_pending = None;
                    self.error = Some(error);
                    self.auth_status = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.agent_issue_pending = None;
                    self.error = Some("GitHub issue load for agent interrupted".into());
                    repaint = true;
                }
            }
        }

        if let Some(rx) = self.pr_list_pending.as_ref() {
            match rx.try_recv() {
                Ok(PullRequestEvent::ListReady(items)) => {
                    self.pull_requests = items;
                    self.pr_loading = false;
                    self.pr_list_pending = None;
                    repaint = true;
                }
                Ok(PullRequestEvent::Failed { context, error }) => {
                    self.pr_loading = false;
                    self.error = Some(format!("Failed to {context}: {error}"));
                    self.pr_list_pending = None;
                    repaint = true;
                }
                Ok(_) => {
                    self.pr_list_pending = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.pr_loading = false;
                    self.pr_list_pending = None;
                    repaint = true;
                }
            }
        }

        if let Some(rx) = self.pr_pending.as_ref() {
            match rx.try_recv() {
                Ok(PullRequestEvent::Loaded { tab_index, detail }) => {
                    self.pr_details.insert(detail.number, detail.clone());
                    self.pr_loaded = Some((tab_index, detail));
                    self.pr_pending = None;
                    repaint = true;
                }
                Ok(PullRequestEvent::Merged { number, base_branch }) => {
                    self.pr_merge_modal.submitting = false;
                    self.pr_merge_modal.close();
                    self.pr_merged_on = Some(number);
                    self.pull_requests.retain(|pr| pr.number != number);
                    self.auth_status =
                        Some(format!("Merged pull request #{number} into {base_branch}"));
                    self.pr_pending = None;
                    repaint = true;
                }
                Ok(PullRequestEvent::Failed { context, error }) => {
                    self.pr_pending = None;
                    if self.pr_merge_modal.open {
                        self.pr_merge_modal.fail(error);
                    } else {
                        self.error = Some(format!("Failed to {context}: {error}"));
                    }
                    repaint = true;
                }
                Ok(PullRequestEvent::ListReady(_)) => {
                    self.pr_pending = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.pr_merge_modal.submitting = false;
                    self.pr_pending = None;
                    self.error = Some("GitHub pull request request interrupted".into());
                    repaint = true;
                }
            }
        }

        if let Some(rx) = self.comment_pending.as_ref() {
            match rx.try_recv() {
                Ok(IssueCommentCreateEvent::Created { issue_number, html_url }) => {
                    self.comment_modal.submitting = false;
                    self.comment_modal.close();
                    self.comment_pending = None;
                    self.comment_posted_on = Some(issue_number);
                    self.auth_status = Some(format!(
                        "Comment posted on issue #{issue_number}: {html_url}"
                    ));
                    repaint = true;
                }
                Ok(IssueCommentCreateEvent::Failed { error }) => {
                    self.comment_modal.submitting = false;
                    self.comment_modal.error = Some(error);
                    self.comment_pending = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.comment_modal.submitting = false;
                    self.comment_modal.error = Some("GitHub comment request interrupted".into());
                    self.comment_pending = None;
                    repaint = true;
                }
            }
        }

        if let Some(rx) = self.verify_pending.as_ref() {
            match rx.try_recv() {
                Ok(GitHubVerifyEvent::Verified { login }) => {
                    self.auth_login = Some(login.clone());
                    self.auth_verifying = false;
                    self.auth_error = None;
                    self.auth_status = Some(format!("Signed in as @{login}"));
                    self.verify_pending = None;
                    repaint = true;
                }
                Ok(GitHubVerifyEvent::Failed { message }) => {
                    self.auth_verifying = false;
                    self.auth_error = Some(message);
                    self.auth_status = None;
                    self.verify_pending = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.auth_verifying = false;
                    self.auth_error = Some("GitHub token verification interrupted".into());
                    self.verify_pending = None;
                    repaint = true;
                }
            }
        }

        let Some(rx) = self.list_pending.as_ref() else {
            return repaint;
        };

        match rx.try_recv() {
            Ok(IssuesListEvent::Ready { repo, issues }) => {
                self.repo = Some(repo);
                self.issues = issues;
                self.loading = false;
                self.error = None;
                self.list_pending = None;
                true
            }
            Ok(IssuesListEvent::Failed(message)) => {
                self.loading = false;
                self.error = Some(message);
                self.list_pending = None;
                true
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => repaint,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.loading = false;
                self.error = Some("GitHub issue list interrupted".into());
                self.list_pending = None;
                true
            }
        }
    }

    fn handle_metadata_event(&mut self, event: MetadataEvent) {
        match event {
            MetadataEvent::LabelsListed(labels) => {
                self.labels_modal.apply_labels(labels);
            }
            MetadataEvent::LabelSaved(label) => {
                self.labels_modal.apply_saved(label);
            }
            MetadataEvent::LabelDeleted { name } => {
                self.labels_modal.apply_deleted(&name);
            }
            MetadataEvent::MilestonesListed(milestones) => {
                self.milestones_modal.apply_milestones(milestones);
            }
            MetadataEvent::MilestoneSaved(milestone) => {
                self.milestones_modal.apply_saved(milestone);
            }
            MetadataEvent::MilestoneDeleted { number } => {
                self.milestones_modal.apply_deleted(number);
            }
            MetadataEvent::IssueMetadataLoaded {
                issue_number,
                labels,
                milestones,
                selected_labels,
                milestone,
            } => {
                let _ = issue_number;
                self.issue_metadata_modal.apply_loaded(
                    labels,
                    milestones,
                    selected_labels,
                    milestone,
                );
            }
            MetadataEvent::IssueUpdated { issue_number } => {
                self.issue_metadata_modal.submitting = false;
                self.issue_metadata_modal.close();
                self.issue_updated_on = Some(issue_number);
                self.auth_status =
                    Some(format!("Updated labels and milestone on issue #{issue_number}"));
            }
            MetadataEvent::Failed { context, error } => {
                self.fail_open_modals(format!("Failed to {context}: {error}"));
            }
        }
    }

    fn fail_open_modals(&mut self, error: String) {
        if self.labels_modal.open {
            self.labels_modal.fail(error.clone());
        }
        if self.milestones_modal.open {
            self.milestones_modal.fail(error.clone());
        }
        if self.issue_metadata_modal.open {
            self.issue_metadata_modal.fail(error);
        }
    }

    fn can_spawn_metadata(&self) -> bool {
        self.metadata_pending.is_none()
    }

    pub fn request_list_labels(&mut self, owner: &str, repo: &str, http: HttpClientService) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.labels_modal.loading = true;
        self.metadata_pending = Some(spawn_list_labels(
            http,
            owner.to_string(),
            repo.to_string(),
            self.token(),
        ));
    }

    pub fn request_create_label(
        &mut self,
        owner: &str,
        repo: &str,
        name: String,
        color: String,
        description: String,
        http: HttpClientService,
    ) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.labels_modal.submitting = true;
        self.labels_modal.error = None;
        self.metadata_pending = Some(spawn_create_label(
            http,
            owner.to_string(),
            repo.to_string(),
            name,
            color,
            description,
            self.token(),
        ));
    }

    pub fn request_update_label(
        &mut self,
        owner: &str,
        repo: &str,
        original_name: String,
        name: String,
        color: String,
        description: String,
        http: HttpClientService,
    ) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.labels_modal.submitting = true;
        self.labels_modal.error = None;
        self.metadata_pending = Some(spawn_update_label(
            http,
            owner.to_string(),
            repo.to_string(),
            original_name,
            name,
            color,
            description,
            self.token(),
        ));
    }

    pub fn request_delete_label(
        &mut self,
        owner: &str,
        repo: &str,
        name: String,
        http: HttpClientService,
    ) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.labels_modal.submitting = true;
        self.labels_modal.error = None;
        self.metadata_pending = Some(spawn_delete_label(
            http,
            owner.to_string(),
            repo.to_string(),
            name,
            self.token(),
        ));
    }

    pub fn request_list_milestones(&mut self, owner: &str, repo: &str, http: HttpClientService) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.milestones_modal.loading = true;
        self.metadata_pending = Some(spawn_list_milestones(
            http,
            owner.to_string(),
            repo.to_string(),
            self.token(),
        ));
    }

    pub fn request_create_milestone(
        &mut self,
        owner: &str,
        repo: &str,
        title: String,
        description: String,
        due_on: Option<String>,
        http: HttpClientService,
    ) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.milestones_modal.submitting = true;
        self.milestones_modal.error = None;
        self.metadata_pending = Some(spawn_create_milestone(
            http,
            owner.to_string(),
            repo.to_string(),
            title,
            description,
            due_on,
            self.token(),
        ));
    }

    pub fn request_update_milestone(
        &mut self,
        owner: &str,
        repo: &str,
        number: u64,
        title: String,
        description: String,
        due_on: Option<String>,
        state: String,
        http: HttpClientService,
    ) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.milestones_modal.submitting = true;
        self.milestones_modal.error = None;
        self.metadata_pending = Some(spawn_update_milestone(
            http,
            owner.to_string(),
            repo.to_string(),
            number,
            title,
            description,
            due_on,
            state,
            self.token(),
        ));
    }

    pub fn request_delete_milestone(
        &mut self,
        owner: &str,
        repo: &str,
        number: u64,
        http: HttpClientService,
    ) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.milestones_modal.submitting = true;
        self.milestones_modal.error = None;
        self.metadata_pending = Some(spawn_delete_milestone(
            http,
            owner.to_string(),
            repo.to_string(),
            number,
            self.token(),
        ));
    }

    pub fn request_load_issue_metadata(
        &mut self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        http: HttpClientService,
    ) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.issue_metadata_modal.loading = true;
        self.issue_metadata_modal.error = None;
        self.metadata_pending = Some(spawn_load_issue_metadata(
            http,
            owner.to_string(),
            repo.to_string(),
            issue_number,
            self.token(),
        ));
    }

    pub fn request_update_issue_metadata(
        &mut self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        labels: Vec<String>,
        milestone: Option<u64>,
        http: HttpClientService,
    ) {
        if !self.can_spawn_metadata() {
            return;
        }
        self.issue_metadata_modal.submitting = true;
        self.issue_metadata_modal.error = None;
        self.metadata_pending = Some(spawn_update_issue_metadata(
            http,
            owner.to_string(),
            repo.to_string(),
            issue_number,
            labels,
            milestone,
            self.token(),
        ));
    }

    /// Spawns a background fetch for one issue into an editor tab.
    pub fn spawn_open_issue(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        tab_index: usize,
        http: HttpClientService,
    ) -> Receiver<FileLoadEvent> {
        spawn_issue_load(
            http,
            owner.to_string(),
            repo.to_string(),
            number,
            tab_index,
            self.token(),
        )
    }

    /// Spawns a background GitHub issue creation from the compose tab.
    pub fn spawn_create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: String,
        body: String,
        tab_index: usize,
        http: HttpClientService,
    ) -> Receiver<IssueCreateEvent> {
        spawn_create_issue(
            http,
            owner.to_string(),
            repo.to_string(),
            title,
            body,
            tab_index,
            self.token(),
        )
    }

    /// Spawns a background GitHub issue comment post.
    pub fn request_create_comment(
        &mut self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        body: String,
        http: HttpClientService,
    ) {
        if self.comment_pending.is_some() {
            return;
        }
        self.comment_modal.submitting = true;
        self.comment_modal.error = None;
        self.comment_pending = Some(spawn_create_comment(
            http,
            owner.to_string(),
            repo.to_string(),
            issue_number,
            body,
            self.token(),
        ));
    }
}

impl Clone for IssuesState {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            issues: self.issues.clone(),
            error: self.error.clone(),
            loading: false,
            stored_token: self.stored_token.clone(),
            auth_login: self.auth_login.clone(),
            auth_status: self.auth_status.clone(),
            auth_error: self.auth_error.clone(),
            auth_verifying: false,
            comment_modal: self.comment_modal.clone(),
            labels_modal: self.labels_modal.clone(),
            milestones_modal: self.milestones_modal.clone(),
            issue_metadata_modal: self.issue_metadata_modal.clone(),
            comment_posted_on: None,
            issue_updated_on: None,
            issue_sent_to_agent: None,
            sidebar_view: self.sidebar_view,
            pull_requests: self.pull_requests.clone(),
            pr_loading: false,
            pr_details: self.pr_details.clone(),
            pr_merge_modal: self.pr_merge_modal.clone(),
            pr_loaded: None,
            pr_merged_on: None,
            list_pending: None,
            verify_pending: None,
            comment_pending: None,
            metadata_pending: None,
            agent_issue_pending: None,
            pr_list_pending: None,
            pr_pending: None,
        }
    }
}
