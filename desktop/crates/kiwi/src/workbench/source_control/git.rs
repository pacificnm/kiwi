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
    /// Commits on this branch not on the upstream remote, when tracked.
    pub ahead: u32,
    /// Commits on upstream not on this branch, when tracked.
    pub behind: u32,
    /// Whether `HEAD` has an upstream tracking branch.
    pub has_upstream: bool,
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
        /// Command line label.
        command: String,
        /// Short log summary.
        summary: String,
        /// Combined stdout/stderr.
        output: String,
    },
    /// Command failed.
    Failed {
        /// Command line label.
        command: String,
        /// Short error summary.
        summary: String,
        /// Combined stdout/stderr or error detail.
        output: String,
    },
}

/// One commit in the history list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommitEntry {
    /// Full commit hash.
    pub hash: String,
    /// Short hash for display.
    pub short_hash: String,
    /// Author name.
    pub author: String,
    /// Commit date (`YYYY-MM-DD`).
    pub date: String,
    /// First line of the commit message.
    pub subject: String,
}

/// Result of a background commit history load.
#[derive(Debug)]
pub enum GitLogEvent {
    /// Parsed recent commits.
    Ready(Vec<GitCommitEntry>),
    /// Failed to read history.
    Failed(String),
}

/// Result of a background branch list load.
#[derive(Debug)]
pub enum GitBranchesEvent {
    /// Local branch names sorted by ref name.
    Ready(Vec<String>),
    /// Failed to list branches.
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
    let command = if paths.len() == 1 && paths[0] == "." {
        "git add .".into()
    } else {
        format!("git add {}", paths.join(" "))
    };
    spawn_action(root, command, move |root| {
        let mut cmd = git_command(root);
        cmd.arg("add");
        for path in &paths {
            cmd.arg(path);
        }
        run_git(cmd, "stage changes")
    })
}

/// Unstages a path on a background thread.
pub fn spawn_git_restore_staged(root: PathBuf, path: String) -> mpsc::Receiver<GitActionEvent> {
    let command = format!("git restore --staged {path}");
    spawn_action(root, command, move |root| {
        let mut cmd = git_command(root);
        cmd.arg("restore").arg("--staged").arg(&path);
        run_git(cmd, &format!("unstage {path}"))
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
    let preview = message.lines().next().unwrap_or(&message);
    let preview = if preview.len() > 48 {
        format!("{}…", &preview[..48])
    } else {
        preview.to_string()
    };
    let command = format!("git commit -m \"{preview}\"");
    spawn_action(root, command, move |root| {
        let mut cmd = git_command(root);
        cmd.arg("commit").arg("-m").arg(&message);
        run_git(cmd, "create commit")
    })
}

/// Pushes the current branch on a background thread.
pub fn spawn_git_push(
    root: PathBuf,
    branch: String,
    has_upstream: bool,
) -> mpsc::Receiver<GitActionEvent> {
    let command = if has_upstream {
        "git push".into()
    } else {
        format!("git push -u origin {branch}")
    };
    spawn_action(root, command, move |root| {
        let mut cmd = git_command(root);
        cmd.arg("push");
        if !has_upstream {
            cmd.arg("-u").arg("origin").arg(&branch);
        }
        run_git(cmd, "push")
    })
}

/// Reads recent commits on a background thread.
pub fn spawn_git_log(root: PathBuf) -> mpsc::Receiver<GitLogEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match read_log(&root) {
            Ok(commits) => GitLogEvent::Ready(commits),
            Err(error) => GitLogEvent::Failed(error.to_string()),
        };
        let _ = tx.send(event);
    });
    rx
}

/// Lists local branches on a background thread.
pub fn spawn_git_branch_list(root: PathBuf) -> mpsc::Receiver<GitBranchesEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match read_branch_list(&root) {
            Ok(branches) => GitBranchesEvent::Ready(branches),
            Err(error) => GitBranchesEvent::Failed(error.to_string()),
        };
        let _ = tx.send(event);
    });
    rx
}

/// Checks out a branch on a background thread.
pub fn spawn_git_checkout(root: PathBuf, branch: String) -> mpsc::Receiver<GitActionEvent> {
    let command = format!("git switch {branch}");
    spawn_action(root, command, move |root| checkout_branch(root, &branch))
}

/// Creates and checks out a new branch on a background thread.
pub fn spawn_git_branch_create(
    root: PathBuf,
    name: String,
    start_point: Option<String>,
) -> mpsc::Receiver<GitActionEvent> {
    let command = match &start_point {
        Some(start) => format!("git switch -c {name} {start}"),
        None => format!("git switch -c {name}"),
    };
    spawn_action(root, command, move |root| {
        let mut cmd = git_command(root);
        cmd.arg("switch").arg("-c").arg(&name);
        if let Some(start) = &start_point {
            cmd.arg(start);
        }
        match run_git(cmd, &format!("create branch {name}")) {
            Ok(result) => Ok(result),
            Err(_) => {
                let mut cmd = git_command(root);
                cmd.arg("checkout").arg("-b").arg(&name);
                if let Some(start) = start_point {
                    cmd.arg(start);
                }
                run_git(cmd, &format!("create branch {name}"))
            }
        }
    })
}

fn spawn_action(
    root: PathBuf,
    command: String,
    action: impl FnOnce(&PathBuf) -> NestResult<GitCommandResult> + Send + 'static,
) -> mpsc::Receiver<GitActionEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match action(&root) {
            Ok(result) => GitActionEvent::Succeeded {
                command,
                summary: result.summary,
                output: result.output,
            },
            Err(error) => GitActionEvent::Failed {
                command,
                summary: error.to_string(),
                output: error.to_string(),
            },
        };
        let _ = tx.send(event);
    });
    rx
}

struct GitCommandResult {
    summary: String,
    output: String,
}

fn read_status(root: &PathBuf) -> NestResult<GitStatus> {
    if !inside_work_tree(root)? {
        return Err(NestError::validation("not a git repository"));
    }

    let branch = read_branch(root)?;
    let (ahead, behind, has_upstream) = read_upstream_counts(root)?;
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

    Ok(GitStatus {
        branch,
        changes,
        ahead,
        behind,
        has_upstream,
    })
}

fn read_upstream_counts(root: &PathBuf) -> NestResult<(u32, u32, bool)> {
    let output = git_command(root)
        .arg("rev-list")
        .arg("--left-right")
        .arg("--count")
        .arg("HEAD...@{upstream}")
        .output()
        .map_err(|error| NestError::io(format!("git rev-list failed: {error}")))?;

    if !output.status.success() {
        return Ok((0, 0, false));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let (ahead, behind) = parse_upstream_counts(&stdout).unwrap_or((0, 0));
    Ok((ahead, behind, true))
}

fn parse_upstream_counts(stdout: &str) -> Option<(u32, u32)> {
    let mut parts = stdout.split_whitespace();
    let ahead = parts.next()?.parse().ok()?;
    let behind = parts.next()?.parse().ok()?;
    Some((ahead, behind))
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

fn read_branch_list(root: &PathBuf) -> NestResult<Vec<String>> {
    if !inside_work_tree(root)? {
        return Err(NestError::validation("not a git repository"));
    }

    let output = git_command(root)
        .arg("for-each-ref")
        .arg("--format=%(refname:short)")
        .arg("refs/heads/")
        .arg("--sort=refname")
        .output()
        .map_err(|error| NestError::io(format!("git for-each-ref failed: {error}")))?;

    if !output.status.success() {
        return Err(NestError::io(format!(
            "git for-each-ref failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn checkout_branch(root: &PathBuf, branch: &str) -> NestResult<GitCommandResult> {
    let mut cmd = git_command(root);
    cmd.arg("switch").arg(branch);
    match run_git(cmd, &format!("switch to {branch}")) {
        Ok(result) => Ok(result),
        Err(_) => {
            let mut cmd = git_command(root);
            cmd.arg("checkout").arg(branch);
            run_git(cmd, &format!("checkout {branch}"))
        }
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

fn run_git(mut command: Command, action: &str) -> NestResult<GitCommandResult> {
    let output = command
        .output()
        .map_err(|error| NestError::io(format!("failed to {action}: {error}")))?;

    let combined = format_command_output(&output.stdout, &output.stderr);

    if output.status.success() {
        Ok(GitCommandResult {
            summary: format!("Git: {action}"),
            output: combined,
        })
    } else {
        Err(NestError::io(format!(
            "git {action} failed: {}",
            combined.trim()
        )))
    }
}

fn format_command_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);
    let mut combined = String::new();
    if !stdout.trim().is_empty() {
        combined.push_str(stdout.trim_end());
    }
    if !stderr.trim().is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(stderr.trim_end());
    }
    if combined.is_empty() {
        "(no output)".into()
    } else {
        combined
    }
}

fn read_log(root: &PathBuf) -> NestResult<Vec<GitCommitEntry>> {
    if !inside_work_tree(root)? {
        return Err(NestError::validation("not a git repository"));
    }

    let output = git_command(root)
        .arg("log")
        .arg("-50")
        .arg("--pretty=format:%H%x09%h%x09%an%x09%ad%x09%s")
        .arg("--date=short")
        .output()
        .map_err(|error| NestError::io(format!("git log failed: {error}")))?;

    if !output.status.success() {
        return Err(NestError::io(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter_map(parse_log_line)
        .collect())
}

fn parse_log_line(line: &str) -> Option<GitCommitEntry> {
    let mut parts = line.splitn(5, '\t');
    let hash = parts.next()?.to_string();
    let short_hash = parts.next()?.to_string();
    let author = parts.next()?.to_string();
    let date = parts.next()?.to_string();
    let subject = parts.next()?.to_string();
    Some(GitCommitEntry {
        hash,
        short_hash,
        author,
        date,
        subject,
    })
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

    #[test]
    fn parses_upstream_ahead_behind_counts() {
        assert_eq!(parse_upstream_counts("2\t0\n"), Some((2, 0)));
        assert_eq!(parse_upstream_counts("0 3"), Some((0, 3)));
        assert_eq!(parse_upstream_counts("bad"), None);
    }

    #[test]
    fn parses_log_line_fields() {
        let entry = parse_log_line(
            "abcd1234ef567890abcd1234ef567890abcd1234\tabcd123\tJane Doe\t2026-07-04\tFix title bar",
        )
        .unwrap();
        assert_eq!(entry.short_hash, "abcd123");
        assert_eq!(entry.author, "Jane Doe");
        assert_eq!(entry.subject, "Fix title bar");
    }
}
