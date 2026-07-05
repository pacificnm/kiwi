//! Git CLI helpers — status parsing and background commands.

use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use nest_error::{NestError, NestResult};

/// Kind of working-tree change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// Modified file.
    Modified,
    /// Added file.
    Added,
    /// Deleted file.
    Deleted,
    /// Renamed file.
    Renamed,
    /// Copied file.
    Copied,
    /// Untracked file.
    Untracked,
    /// Unknown status code.
    Other,
}

impl ChangeKind {
    fn from_codes(index: Option<char>, worktree: Option<char>, untracked: bool) -> Self {
        if untracked {
            return Self::Untracked;
        }
        let code = index.filter(|c| *c != ' ').or(worktree.filter(|c| *c != ' '));
        match code {
            Some('M') | Some('T') => Self::Modified,
            Some('A') => Self::Added,
            Some('D') => Self::Deleted,
            Some('R') => Self::Renamed,
            Some('C') => Self::Copied,
            Some('U') => Self::Modified,
            _ => Self::Other,
        }
    }

    /// Short label for the sidebar row.
    pub fn label(self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::Copied => "C",
            Self::Untracked => "U",
            Self::Other => "?",
        }
    }
}

/// One changed path in the repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitChange {
    /// Path relative to the repo root (destination path for renames).
    pub path: String,
    /// Staging-area status code, if any.
    pub index_status: Option<char>,
    /// Working-tree status code, if any.
    pub worktree_status: Option<char>,
    /// Whether the index half of the status is set (staged).
    pub staged: bool,
    /// Derived change kind for display.
    pub kind: ChangeKind,
}

impl GitChange {
    /// True when the path has unstaged edits (including untracked).
    pub fn unstaged(&self) -> bool {
        self.kind == ChangeKind::Untracked
            || self
                .worktree_status
                .is_some_and(|status| status != ' ')
    }
}

/// Parsed `git status --porcelain` snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStatus {
    /// Current branch name, or `HEAD` when detached.
    pub branch: String,
    /// All changed paths.
    pub changes: Vec<GitChange>,
}

impl GitStatus {
    /// Changes present in the index (staged).
    pub fn staged(&self) -> impl Iterator<Item = &GitChange> {
        self.changes.iter().filter(|change| change.staged)
    }

    /// Unstaged or untracked changes.
    pub fn unstaged(&self) -> impl Iterator<Item = &GitChange> {
        self.changes.iter().filter(|change| change.unstaged())
    }
}

/// Result of a background status refresh.
#[derive(Debug)]
pub enum GitStatusEvent {
    /// Parsed repository status.
    Ready(GitStatus),
    /// Root is not inside a git work tree.
    NotRepository,
    /// Git command failed.
    Failed(String),
}

/// Result of a background mutating git command.
#[derive(Debug)]
pub enum GitActionEvent {
    /// Command succeeded.
    Succeeded {
        /// Short log summary.
        summary: String,
    },
    /// Command failed.
    Failed(String),
}

/// Parses one line of `git status --porcelain=1` output.
pub fn parse_porcelain_line(line: &str) -> NestResult<GitChange> {
    let line = line.trim_end();
    if line.is_empty() {
        return Err(NestError::validation("empty porcelain line"));
    }

    if line.starts_with("?? ") {
        let path = line[3..].trim().to_string();
        return Ok(GitChange {
            path,
            index_status: None,
            worktree_status: None,
            staged: false,
            kind: ChangeKind::Untracked,
        });
    }

    if line.len() < 4 {
        return Err(NestError::validation(format!("invalid porcelain line: {line}")));
    }

    let index_status = line.chars().next().filter(|c| *c != ' ');
    let worktree_status = line.chars().nth(1).filter(|c| *c != ' ');
    let mut path = line[3..].trim().to_string();

    if let Some((_, dest)) = path.split_once(" -> ") {
        path = dest.trim().to_string();
    }

    let staged = line.as_bytes()[0] != b' ';
    let kind = ChangeKind::from_codes(index_status, worktree_status, false);

    Ok(GitChange {
        path,
        index_status,
        worktree_status,
        staged,
        kind,
    })
}

/// Reads git status on a background thread.
pub fn spawn_git_status(root: PathBuf) -> mpsc::Receiver<GitStatusEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match read_status(&root) {
            Ok(status) => GitStatusEvent::Ready(status),
            Err(error) if is_not_repo_error(&error) => GitStatusEvent::NotRepository,
            Err(error) => GitStatusEvent::Failed(error.to_string()),
        };
        let _ = tx.send(event);
    });
    rx
}

/// Stages paths on a background thread.
pub fn spawn_git_add(root: PathBuf, paths: Vec<String>) -> mpsc::Receiver<GitActionEvent> {
    spawn_action(root, move |root| {
        let mut command = git_command(root);
        command.arg("add");
        for path in &paths {
            command.arg(path);
        }
        run_git(command, "stage changes")
    })
}

/// Unstages a path on a background thread.
pub fn spawn_git_restore_staged(root: PathBuf, path: String) -> mpsc::Receiver<GitActionEvent> {
    spawn_action(root, move |root| {
        let mut command = git_command(root);
        command.arg("restore").arg("--staged").arg(&path);
        run_git(command, &format!("unstage {path}"))
    })
}

/// Which side of a change to diff against HEAD / index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffSide {
    /// Staged index changes (`git diff --cached`).
    Staged,
    /// Working tree changes (`git diff`).
    Unstaged,
    /// New untracked file (`git diff --no-index /dev/null path`).
    Untracked,
}

/// Reads a unified diff for one path on a background thread.
pub fn spawn_git_diff(
    root: PathBuf,
    path: String,
    side: DiffSide,
) -> mpsc::Receiver<GitDiffEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match read_diff(&root, &path, side) {
            Ok(diff) => GitDiffEvent::Ready { path, diff },
            Err(error) => GitDiffEvent::Failed {
                path,
                error: error.to_string(),
            },
        };
        let _ = tx.send(event);
    });
    rx
}

/// Result of a background diff read.
#[derive(Debug)]
pub enum GitDiffEvent {
    /// Unified diff text for the path.
    Ready {
        /// Repository-relative path.
        path: String,
        /// Unified diff output.
        diff: String,
    },
    /// Failed to produce a diff.
    Failed {
        /// Repository-relative path.
        path: String,
        /// Error message.
        error: String,
    },
}

/// Creates a commit on a background thread.
pub fn spawn_git_commit(root: PathBuf, message: String) -> mpsc::Receiver<GitActionEvent> {
    spawn_action(root, move |root| {
        let mut command = git_command(root);
        command.arg("commit").arg("-m").arg(&message);
        run_git(command, "create commit")
    })
}

fn spawn_action(
    root: PathBuf,
    action: impl FnOnce(&PathBuf) -> NestResult<String> + Send + 'static,
) -> mpsc::Receiver<GitActionEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match action(&root) {
            Ok(summary) => GitActionEvent::Succeeded { summary },
            Err(error) => GitActionEvent::Failed(error.to_string()),
        };
        let _ = tx.send(event);
    });
    rx
}

fn read_status(root: &PathBuf) -> NestResult<GitStatus> {
    if !inside_work_tree(root)? {
        return Err(NestError::validation("not a git repository"));
    }

    let branch = read_branch(root)?;
    let output = git_command(root)
        .arg("status")
        .arg("--porcelain=1")
        .output()
        .map_err(|error| NestError::io(format!("git status failed: {error}")))?;

    if !output.status.success() {
        return Err(NestError::io(format!(
            "git status exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut changes = Vec::new();
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        changes.push(parse_porcelain_line(line)?);
    }

    Ok(GitStatus { branch, changes })
}

fn read_branch(root: &PathBuf) -> NestResult<String> {
    let output = git_command(root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .map_err(|error| NestError::io(format!("git branch failed: {error}")))?;

    if !output.status.success() {
        return Err(NestError::io(format!(
            "git branch exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        Ok("HEAD".into())
    } else {
        Ok(branch)
    }
}

fn inside_work_tree(root: &PathBuf) -> NestResult<bool> {
    let output = git_command(root)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map_err(|error| NestError::io(format!("git rev-parse failed: {error}")))?;

    if !output.status.success() {
        return Ok(false);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
}

fn git_command(root: &PathBuf) -> Command {
    let mut command = Command::new("git");
    command.current_dir(root);
    command
}

/// Reads unified diff text for a single path.
pub fn read_diff(root: &PathBuf, path: &str, side: DiffSide) -> NestResult<String> {
    let mut command = git_command(root);
    command.arg("diff");
    match side {
        DiffSide::Staged => {
            command.arg("--cached").arg("--").arg(path);
        }
        DiffSide::Unstaged => {
            command.arg("--").arg(path);
        }
        DiffSide::Untracked => {
            command.arg("--no-index").arg("--").arg("/dev/null").arg(path);
        }
    }
    run_git_diff(command, path)
}

fn run_git_diff(mut command: Command, path: &str) -> NestResult<String> {
    let output = command
        .output()
        .map_err(|error| NestError::io(format!("git diff failed for {path}: {error}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let code = output.status.code();

    if output.status.success() || code == Some(1) {
        if stdout.trim().is_empty() {
            Ok(format!("(no diff for {path})\n"))
        } else {
            Ok(stdout)
        }
    } else {
        Err(NestError::io(format!(
            "git diff for {path} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn run_git(mut command: Command, action: &str) -> NestResult<String> {
    let output = command
        .output()
        .map_err(|error| NestError::io(format!("failed to {action}: {error}")))?;

    if output.status.success() {
        Ok(format!("Git: {action}"))
    } else {
        Err(NestError::io(format!(
            "git {action} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn is_not_repo_error(error: &NestError) -> bool {
    error.to_string().contains("not a git repository")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_staged_and_unstaged_pairs() {
        let both = parse_porcelain_line("MM file.rs").unwrap();
        assert_eq!(both.path, "file.rs");
        assert!(both.staged);
        assert_eq!(both.worktree_status, Some('M'));

        let rename = parse_porcelain_line("R  old.rs -> new.rs").unwrap();
        assert_eq!(rename.path, "new.rs");
        assert_eq!(rename.kind, ChangeKind::Renamed);
    }
}
