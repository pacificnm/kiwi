use std::path::{Path, PathBuf};

use crate::git::{BranchEntry, GitFileEntry};
use crate::state::{AppEvent, EventSender};

use super::branches::{checkout_local_branch, list_local_branches};
use super::repository::{load_repo_snapshot, GitError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRefreshSnapshot {
    pub branch: Option<String>,
    pub remote_repo: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub file_entries: Vec<GitFileEntry>,
    pub error: Option<String>,
}

pub fn load_git_snapshot(repo_root: &Path, show_untracked: bool) -> GitRefreshSnapshot {
    match load_repo_snapshot(repo_root, show_untracked) {
        Ok(snapshot) => GitRefreshSnapshot {
            branch: Some(snapshot.branch.branch),
            remote_repo: snapshot.remote_repo,
            ahead: snapshot.branch.ahead,
            behind: snapshot.branch.behind,
            file_entries: snapshot.file_entries,
            error: None,
        },
        Err(GitError::Open(message)) => GitRefreshSnapshot {
            branch: None,
            remote_repo: None,
            ahead: 0,
            behind: 0,
            file_entries: Vec::new(),
            error: Some(message),
        },
        Err(GitError::Head(message) | GitError::Status(message)) => GitRefreshSnapshot {
            branch: None,
            remote_repo: None,
            ahead: 0,
            behind: 0,
            file_entries: Vec::new(),
            error: Some(message),
        },
        Err(GitError::Branches(message) | GitError::Checkout(message)) => GitRefreshSnapshot {
            branch: None,
            remote_repo: None,
            ahead: 0,
            behind: 0,
            file_entries: Vec::new(),
            error: Some(message),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchListSnapshot {
    pub entries: Vec<BranchEntry>,
    pub error: Option<String>,
}

pub fn load_branch_list_snapshot(repo_root: &Path) -> BranchListSnapshot {
    match list_local_branches(repo_root) {
        Ok(entries) => BranchListSnapshot {
            entries,
            error: None,
        },
        Err(GitError::Open(message)) => BranchListSnapshot {
            entries: Vec::new(),
            error: Some(message),
        },
        Err(GitError::Branches(message)) => BranchListSnapshot {
            entries: Vec::new(),
            error: Some(message),
        },
        Err(err) => BranchListSnapshot {
            entries: Vec::new(),
            error: Some(err.to_string()),
        },
    }
}

pub fn spawn_branch_list(repo_root: PathBuf, sender: EventSender) {
    std::thread::spawn(move || {
        let snapshot = load_branch_list_snapshot(&repo_root);
        let _ = sender.send(AppEvent::BranchListLoaded {
            entries: snapshot.entries,
            error: snapshot.error,
        });
    });
}

pub fn spawn_branch_checkout(repo_root: PathBuf, branch_name: String, sender: EventSender) {
    std::thread::spawn(move || {
        let error = checkout_local_branch(&repo_root, &branch_name)
            .err()
            .map(|err| err.to_string());
        let _ = sender.send(AppEvent::BranchCheckoutCompleted { branch_name, error });
    });
}

pub fn spawn_git_refresh(repo_root: PathBuf, show_untracked: bool, sender: EventSender) {
    std::thread::spawn(move || {
        let snapshot = load_git_snapshot(&repo_root, show_untracked);
        let _ = sender.send(AppEvent::GitStatusUpdated {
            branch: snapshot.branch,
            remote_repo: snapshot.remote_repo,
            ahead: snapshot.ahead,
            behind: snapshot.behind,
            file_entries: snapshot.file_entries,
            error: snapshot.error,
        });
    });
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;
    use std::time::Duration;

    use super::*;
    use crate::git::GitFileStatus;
    use crate::state::EventChannel;

    fn init_git_repo(path: &std::path::Path) {
        let status = Command::new("git")
            .args(["init", path.to_str().expect("utf8")])
            .status()
            .expect("git init");
        assert!(status.success());
        let status = Command::new("git")
            .args(["config", "user.email", "kiwi@test.local"])
            .current_dir(path)
            .status()
            .expect("git config email");
        assert!(status.success());
        let status = Command::new("git")
            .args(["config", "user.name", "Kiwi Test"])
            .current_dir(path)
            .status()
            .expect("git config name");
        assert!(status.success());
    }

    #[test]
    fn spawn_git_refresh_enqueues_branch_and_file_statuses() {
        let temp = std::env::temp_dir().join(format!("kiwi-git-io-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        init_git_repo(&temp);
        fs::write(temp.join("README.md"), "hello\n").expect("write");
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&temp)
            .status()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&temp)
            .status()
            .expect("git commit");
        fs::write(temp.join("README.md"), "changed\n").expect("modify");

        let mut channel = EventChannel::new();
        spawn_git_refresh(temp.clone(), true, channel.sender());

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut updated = None;
        while std::time::Instant::now() < deadline {
            for event in channel.drain_coalesced() {
                if let AppEvent::GitStatusUpdated {
                    branch,
                    remote_repo: _,
                    ahead,
                    behind,
                    file_entries,
                    error,
                } = event
                {
                    assert!(error.is_none());
                    assert!(branch.is_some());
                    assert_eq!(ahead, 0);
                    assert_eq!(behind, 0);
                    assert!(file_entries.iter().any(|entry| {
                        entry.path == "README.md" && entry.status == GitFileStatus::Modified
                    }));
                    updated = Some(());
                    break;
                }
            }
            if updated.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        updated.expect("git status updated event");
        let _ = fs::remove_dir_all(temp);
    }
}
