//! Git status and commands for the Source Control sidebar.

mod git;

pub use git::{
    spawn_git_add, spawn_git_commit, spawn_git_diff, spawn_git_restore_staged, spawn_git_status,
    ChangeKind, DiffSide, GitActionEvent, GitChange, GitDiffEvent, GitStatus, GitStatusEvent,
};

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
    /// Background status refresh channel.
    status_pending: Option<Receiver<GitStatusEvent>>,
    /// Background mutating git command channel.
    action_pending: Option<Receiver<GitActionEvent>>,
    /// Set when UI should re-request status after an action completes.
    refresh_after_action: bool,
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
            status_pending: None,
            action_pending: None,
            refresh_after_action: false,
        }
    }

    /// Returns true when a background git command is in flight.
    pub fn has_pending_io(&self) -> bool {
        self.status_pending.is_some() || self.action_pending.is_some()
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
                Ok(GitActionEvent::Succeeded { summary }) => {
                    self.error = None;
                    self.action_pending = None;
                    self.refresh_after_action = true;
                    if summary.contains("create commit") {
                        self.commit_message.clear();
                    }
                    tracing::info!(target: "kiwi::git", "{summary}");
                    repaint = true;
                }
                Ok(GitActionEvent::Failed(message)) => {
                    self.error = Some(message);
                    self.action_pending = None;
                    repaint = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.error = Some("Git command interrupted".into());
                    self.action_pending = None;
                    repaint = true;
                }
            }
        }

        if self.refresh_after_action && self.action_pending.is_none() && self.status_pending.is_none()
        {
            self.refresh_after_action = false;
            self.request_refresh(root);
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
}

impl Clone for SourceControlState {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            error: self.error.clone(),
            loading: false,
            not_repo: self.not_repo,
            commit_message: self.commit_message.clone(),
            status_pending: None,
            action_pending: None,
            refresh_after_action: false,
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
