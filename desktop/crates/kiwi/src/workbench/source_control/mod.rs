//! Git status and commands for the Source Control sidebar.

mod branch_modal;
mod git;
mod panel;

pub use branch_modal::{
    show_window as show_branch_create_modal, BranchCreateModalAction, BranchCreateModalState,
};
pub use git::{
    spawn_git_add, spawn_git_branch_create, spawn_git_branch_list, spawn_git_checkout,
    spawn_git_commit, spawn_git_diff, spawn_git_log, spawn_git_push, spawn_git_restore_staged,
    spawn_git_status, ChangeKind, DiffSide, GitActionEvent, GitBranchesEvent, GitChange,
    GitCommitEntry, GitDiffEvent, GitLogEvent, GitStatus, GitStatusEvent,
};
pub use panel::{GitOutputEntry, GitOutputLog, GitPanelView};

use std::path::PathBuf;
use std::sync::mpsc::Receiver;

/// Source control sidebar state.
#[derive(Debug)]
pub struct SourceControlState {
    /// Last successful status snapshot.
    pub status: Option<GitStatus>,
    /// Last error message from git or parsing.
    pub error: Option<String>,
    /// True while a status refresh is in flight.
    pub loading: bool,
    /// Project root is not inside a git work tree.
    pub not_repo: bool,
    /// Draft commit message.
    pub commit_message: String,
    /// Git command output for the bottom panel.
    pub git_output: GitOutputLog,
    /// Bottom Git panel sub-view.
    pub git_panel_view: GitPanelView,
    /// Recent commits for the history view.
    pub git_commits: Vec<GitCommitEntry>,
    /// Whether commit history is loading.
    pub git_commits_loading: bool,
    /// Last commit history error.
    pub git_commits_error: Option<String>,
    /// Expanded commit row in history view.
    pub git_selected_commit: Option<usize>,
    /// Switch to the bottom Git tab on the next frame.
    pub focus_git_panel: bool,
    /// Local branch names for the branch picker.
    pub branches: Vec<String>,
    /// Whether the branch list is loading.
    pub branches_loading: bool,
    /// Last branch list error.
    pub branches_error: Option<String>,
    /// Create-branch modal state.
    pub branch_create_modal: BranchCreateModalState,
    /// Background status refresh channel.
    status_pending: Option<Receiver<GitStatusEvent>>,
    /// Background mutating git command channel.
    action_pending: Option<Receiver<GitActionEvent>>,
    /// Background commit history channel.
    history_pending: Option<Receiver<GitLogEvent>>,
    /// Background branch list channel.
    branches_pending: Option<Receiver<GitBranchesEvent>>,
    /// Set when UI should re-request status after an action completes.
    refresh_after_action: bool,
    /// Set when commit history should reload after an action completes.
    refresh_history_after_action: bool,
    /// Set when branch list should reload after an action completes.
    refresh_branches_after_action: bool,
}

impl Default for SourceControlState {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl SourceControlState {
    /// Creates state for the given project root.
    pub fn new(root: PathBuf) -> Self {
        let _ = root;
        Self {
            status: None,
            error: None,
            loading: false,
            not_repo: false,
            commit_message: String::new(),
            git_output: GitOutputLog::default(),
            git_panel_view: GitPanelView::default(),
            git_commits: Vec::new(),
            git_commits_loading: false,
            git_commits_error: None,
            git_selected_commit: None,
            focus_git_panel: false,
            branches: Vec::new(),
            branches_loading: false,
            branches_error: None,
            branch_create_modal: BranchCreateModalState::default(),
            status_pending: None,
            action_pending: None,
            history_pending: None,
            branches_pending: None,
            refresh_after_action: false,
            refresh_history_after_action: false,
            refresh_branches_after_action: false,
        }
    }

    /// Returns true when a background git command is in flight.
    pub fn has_pending_io(&self) -> bool {
        self.status_pending.is_some()
            || self.action_pending.is_some()
            || self.history_pending.is_some()
            || self.branches_pending.is_some()
    }

    /// Returns true when any git operation is in flight.
    pub fn busy(&self) -> bool {
        self.loading || self.has_pending_io()
    }

    /// Starts a background `git status` for `root` unless one is already running.
    ///
    /// When a snapshot already exists, the refresh runs quietly without clearing the UI.
    pub fn request_refresh(&mut self, root: &PathBuf) {
        if self.status_pending.is_some() {
            return;
        }
        if self.status.is_none() {
            self.loading = true;
        }
        self.error = None;
        self.status_pending = Some(spawn_git_status(root.clone()));
    }

    /// Loads recent commits for the bottom panel history view.
    pub fn request_commit_history(&mut self, root: &PathBuf) {
        if self.history_pending.is_some() || self.not_repo {
            return;
        }
        self.git_commits_loading = true;
        self.git_commits_error = None;
        self.history_pending = Some(spawn_git_log(root.clone()));
    }

    /// Loads local branch names for the branch picker.
    pub fn request_branch_list(&mut self, root: &PathBuf) {
        if self.branches_pending.is_some() || self.not_repo {
            return;
        }
        self.branches_loading = true;
        self.branches_error = None;
        self.branches_pending = Some(spawn_git_branch_list(root.clone()));
    }

    /// Opens the create-branch modal.
    pub fn open_create_branch(&mut self) {
        let current = self
            .status
            .as_ref()
            .map(|status| status.branch.as_str())
            .unwrap_or("HEAD");
        self.branch_create_modal
            .open_with_branches(&self.branches, current);
    }

    /// Opens the create-branch modal with a name suggested from a GitHub issue.
    pub fn open_create_branch_from_issue(&mut self, issue_number: u64, issue_title: &str) {
        let current = self
            .status
            .as_ref()
            .map(|status| status.branch.as_str())
            .unwrap_or("HEAD");
        self.branch_create_modal.open_for_issue(
            &self.branches,
            current,
            issue_number,
            issue_title,
        );
    }

    /// Checks out an existing local branch.
    pub fn checkout_branch(&mut self, root: &PathBuf, branch: String) {
        if self.action_pending.is_some() {
            return;
        }
        self.action_pending = Some(spawn_git_checkout(root.clone(), branch));
    }

    /// Creates a new branch and checks it out.
    pub fn create_branch(
        &mut self,
        root: &PathBuf,
        name: String,
        start_branch: Option<String>,
    ) {
        if self.action_pending.is_some() {
            return;
        }
        self.branch_create_modal.submitting = true;
        self.branch_create_modal.error = None;
        self.action_pending = Some(spawn_git_branch_create(root.clone(), name, start_branch));
    }

    /// Takes the focus-git-panel flag for the workbench shell.
    pub fn take_focus_git_panel(&mut self) -> bool {
        std::mem::take(&mut self.focus_git_panel)
    }

    /// Polls background channels; returns true when UI should repaint.
    pub fn poll(&mut self, root: &PathBuf) -> bool {
        let mut repaint = false;

        if let Some(rx) = self.status_pending.as_ref() {
            match rx.try_recv() {
                Ok(GitStatusEvent::Ready(status)) => {
                    self.status = Some(status);
                    self.not_repo = false;
                    self.error = None;
                    self.loading = false;
                    self.status_pending = None;
                    repaint = true;
                }
                Ok(GitStatusEvent::NotRepository) => {
                    self.status = None;
                    self.not_repo = true;
                    self.error = None;
                    self.loading = false;
                    self.status_pending = None;
                    repaint = true;
                }
                Ok(GitStatusEvent::Failed(message)) => {
                    self.error = Some(message);
                    self.loading = false;
                    self.status_pending = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.error = Some("Git status interrupted".into());
                    self.loading = false;
                    self.status_pending = None;
                    repaint = true;
                }
            }
        }

        if let Some(rx) = self.action_pending.as_ref() {
            match rx.try_recv() {
                Ok(GitActionEvent::Succeeded {
                    command,
                    summary,
                    output,
                }) => {
                    self.error = None;
                    self.action_pending = None;
                    self.refresh_after_action = true;
                    self.refresh_history_after_action = true;
                    self.refresh_branches_after_action = true;
                    self.branch_create_modal.submitting = false;
                    self.branch_create_modal.close();
                    self.git_output.push(command, true, output);
                    if summary.contains("create commit") {
                        self.commit_message.clear();
                    }
                    tracing::info!(target: "kiwi::git", "{summary}");
                    repaint = true;
                }
                Ok(GitActionEvent::Failed {
                    command,
                    summary,
                    output,
                }) => {
                    self.error = Some(summary.clone());
                    self.action_pending = None;
                    self.branch_create_modal.submitting = false;
                    if self.branch_create_modal.open {
                        self.branch_create_modal.fail(summary.clone());
                    }
                    self.git_output.push(command, false, output);
                    self.focus_git_panel = true;
                    tracing::warn!(target: "kiwi::git", "{summary}");
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.error = Some("Git command interrupted".into());
                    self.action_pending = None;
                    self.branch_create_modal.submitting = false;
                    self.focus_git_panel = true;
                    repaint = true;
                }
            }
        }

        if let Some(rx) = self.branches_pending.as_ref() {
            match rx.try_recv() {
                Ok(GitBranchesEvent::Ready(branches)) => {
                    self.branches = branches;
                    self.branches_loading = false;
                    self.branches_error = None;
                    self.branches_pending = None;
                    repaint = true;
                }
                Ok(GitBranchesEvent::Failed(message)) => {
                    self.branches_loading = false;
                    self.branches_error = Some(message);
                    self.branches_pending = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.branches_loading = false;
                    self.branches_error = Some("Branch list load interrupted".into());
                    self.branches_pending = None;
                    repaint = true;
                }
            }
        }

        if let Some(rx) = self.history_pending.as_ref() {
            match rx.try_recv() {
                Ok(GitLogEvent::Ready(commits)) => {
                    self.git_commits = commits;
                    self.git_commits_loading = false;
                    self.git_commits_error = None;
                    self.history_pending = None;
                    repaint = true;
                }
                Ok(GitLogEvent::Failed(message)) => {
                    self.git_commits_loading = false;
                    self.git_commits_error = Some(message);
                    self.history_pending = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.git_commits_loading = false;
                    self.git_commits_error = Some("Commit history load interrupted".into());
                    self.history_pending = None;
                    repaint = true;
                }
            }
        }

        if self.refresh_after_action && self.action_pending.is_none() && self.status_pending.is_none()
        {
            self.refresh_after_action = false;
            self.request_refresh(root);
        }

        if self.refresh_history_after_action
            && self.action_pending.is_none()
            && self.history_pending.is_none()
        {
            self.refresh_history_after_action = false;
            self.request_commit_history(root);
        }

        if self.refresh_branches_after_action
            && self.action_pending.is_none()
            && self.branches_pending.is_none()
        {
            self.refresh_branches_after_action = false;
            self.request_branch_list(root);
        }

        repaint
    }

    /// Stages a single path relative to the repository root.
    pub fn stage_path(&mut self, root: &PathBuf, path: &str) {
        if self.action_pending.is_some() {
            return;
        }
        self.action_pending = Some(spawn_git_add(root.clone(), vec![path.to_string()]));
    }

    /// Unstages a single path.
    pub fn unstage_path(&mut self, root: &PathBuf, path: &str) {
        if self.action_pending.is_some() {
            return;
        }
        self.action_pending = Some(spawn_git_restore_staged(root.clone(), path.to_string()));
    }

    /// Stages all tracked and untracked changes.
    pub fn stage_all(&mut self, root: &PathBuf) {
        if self.action_pending.is_some() {
            return;
        }
        self.action_pending = Some(spawn_git_add(root.clone(), vec![".".into()]));
    }

    /// Creates a commit with the current draft message.
    pub fn commit(&mut self, root: &PathBuf) {
        let message = self.commit_message.trim();
        if message.is_empty() || self.action_pending.is_some() {
            return;
        }
        let message = message.to_string();
        self.action_pending = Some(spawn_git_commit(root.clone(), message));
    }

    /// Pushes the current branch to its upstream remote.
    pub fn push(&mut self, root: &PathBuf) {
        if self.action_pending.is_some() {
            return;
        }
        let Some(status) = self.status.as_ref() else {
            return;
        };
        if status.branch == "HEAD" {
            return;
        }
        self.action_pending = Some(spawn_git_push(
            root.clone(),
            status.branch.clone(),
            status.has_upstream,
        ));
    }
}

impl Clone for SourceControlState {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            error: self.error.clone(),
            loading: false,
            not_repo: self.not_repo,
            commit_message: self.commit_message.clone(),
            git_output: self.git_output.clone(),
            git_panel_view: self.git_panel_view,
            git_commits: self.git_commits.clone(),
            git_commits_loading: false,
            git_commits_error: self.git_commits_error.clone(),
            git_selected_commit: self.git_selected_commit,
            focus_git_panel: false,
            branches: self.branches.clone(),
            branches_loading: false,
            branches_error: self.branches_error.clone(),
            branch_create_modal: self.branch_create_modal.clone(),
            status_pending: None,
            action_pending: None,
            history_pending: None,
            branches_pending: None,
            refresh_after_action: false,
            refresh_history_after_action: false,
            refresh_branches_after_action: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::git::parse_porcelain_line;

    #[test]
    fn parses_modified_and_untracked() {
        let change = parse_porcelain_line(" M src/main.rs").unwrap();
        assert_eq!(change.path, "src/main.rs");
        assert!(!change.staged);
        assert_eq!(change.worktree_status, Some('M'));

        let staged = parse_porcelain_line("M  src/main.rs").unwrap();
        assert!(staged.staged);
        assert_eq!(staged.index_status, Some('M'));

        let untracked = parse_porcelain_line("?? new.txt").unwrap();
        assert_eq!(untracked.path, "new.txt");
        assert!(!untracked.staged);
    }
}
