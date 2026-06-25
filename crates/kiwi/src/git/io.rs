use std::path::{Path, PathBuf};

use crate::git::GitFileEntry;
use crate::state::{AppEvent, EventSender};

use super::repository::{load_repo_snapshot, GitError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRefreshSnapshot {
    pub branch: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub file_entries: Vec<GitFileEntry>,
    pub error: Option<String>,
}

pub fn load_git_snapshot(repo_root: &Path, show_untracked: bool) -> GitRefreshSnapshot {
    match load_repo_snapshot(repo_root, show_untracked) {
        Ok(snapshot) => GitRefreshSnapshot {
            branch: Some(snapshot.branch.branch),
            ahead: snapshot.branch.ahead,
            behind: snapshot.branch.behind,
            file_entries: snapshot.file_entries,
            error: None,
        },
        Err(GitError::Open(message)) => GitRefreshSnapshot {
            branch: None,
            ahead: 0,
            behind: 0,
            file_entries: Vec::new(),
            error: Some(message),
        },
        Err(GitError::Head(message) | GitError::Status(message)) => GitRefreshSnapshot {
            branch: None,
            ahead: 0,
            behind: 0,
            file_entries: Vec::new(),
            error: Some(message),
        },
    }
}

pub fn spawn_git_refresh(repo_root: PathBuf, show_untracked: bool, sender: EventSender) {
    std::thread::spawn(move || {
        let snapshot = load_git_snapshot(&repo_root, show_untracked);
        let _ = sender.send(AppEvent::GitStatusUpdated {
            branch: snapshot.branch,
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
