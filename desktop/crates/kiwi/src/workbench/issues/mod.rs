//! GitHub issues sidebar and issue loading for editor tabs.

pub mod auth;
pub mod comment_modal;
mod github;

use std::path::Path;
use std::sync::mpsc::Receiver;

use nest_http_client::HttpClientService;

use crate::workbench::editor_files::FileLoadEvent;

pub use auth::load_token_from_config;
pub use comment_modal::{CommentModalAction, CommentModalState};
pub use github::{read_github_repo, GitHubVerifyEvent, IssueCommentCreateEvent, IssueCreateEvent};

use github::{
    spawn_create_comment, spawn_create_issue, spawn_issue_list, spawn_issue_load,
    spawn_verify_token,
};

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
    /// Set when a comment posts successfully; consumed to refresh the open issue tab.
    pub comment_posted_on: Option<u64>,
    list_pending: Option<Receiver<IssuesListEvent>>,
    verify_pending: Option<Receiver<GitHubVerifyEvent>>,
    comment_pending: Option<Receiver<IssueCommentCreateEvent>>,
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
            comment_posted_on: None,
            list_pending: None,
            verify_pending: None,
            comment_pending: None,
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

    /// Returns true while a list, verify, or comment request is in flight.
    pub fn busy(&self) -> bool {
        self.loading
            || self.list_pending.is_some()
            || self.auth_verifying
            || self.verify_pending.is_some()
            || self.comment_pending.is_some()
            || self.comment_modal.submitting
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
            comment_posted_on: None,
            list_pending: None,
            verify_pending: None,
            comment_pending: None,
        }
    }
}
