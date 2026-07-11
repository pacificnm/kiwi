//! Git integration for the Source Control panel.
//!
//! Thin, synchronous wrappers over the `git` CLI (Tauri runs commands on a
//! worker thread, so blocking here is fine). Types serialize as camelCase and
//! errors are `NestError`.

use std::path::Path;
use std::process::Command;

use nest_error::{NestError, NestResult};
use serde::Serialize;

/// Kind of working-tree change (drives the badge letter in the UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
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
}

/// One changed path in the repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitChange {
    /// Path relative to the repo root (destination path for renames).
    pub path: String,
    /// Whether the index half of the status is set (staged).
    pub staged: bool,
    /// Whether the working tree half is set (unstaged / untracked edits).
    pub unstaged: bool,
    /// Derived change kind for display.
    pub kind: ChangeKind,
}

/// Parsed `git status` snapshot for the whole repo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    /// Whether the workspace root is inside a git work tree.
    pub is_repo: bool,
    /// Current branch name, or `HEAD` when detached.
    pub branch: String,
    /// All changed paths (staged + unstaged + untracked).
    pub changes: Vec<GitChange>,
    /// Commits on this branch not on upstream, when tracked.
    pub ahead: u32,
    /// Commits on upstream not on this branch, when tracked.
    pub behind: u32,
    /// Whether `HEAD` has an upstream tracking branch.
    pub has_upstream: bool,
}

impl GitStatus {
    fn not_repo() -> Self {
        Self {
            is_repo: false,
            branch: String::new(),
            changes: Vec::new(),
            ahead: 0,
            behind: 0,
            has_upstream: false,
        }
    }
}

/// One commit in the history graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommit {
    /// Full commit hash.
    pub hash: String,
    /// Short hash for display.
    pub short_hash: String,
    /// Author name.
    pub author: String,
    /// Author email.
    pub email: String,
    /// Absolute date (`YYYY-MM-DD HH:MM`).
    pub date: String,
    /// Relative date (`3 days ago`).
    pub relative_date: String,
    /// First line of the commit message.
    pub subject: String,
    /// Parent hashes (2+ ⇒ merge commit).
    pub parents: Vec<String>,
}

/// One file changed in a commit (for the Open Changes view).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommitFileChange {
    /// Path relative to the repo root (destination path for renames).
    pub path: String,
    /// Source path for renames.
    pub old_path: Option<String>,
    /// Porcelain status code (`A`, `M`, `D`, `R`, …).
    pub status: String,
    /// Unified diff for this file.
    pub diff: String,
}

/// Full commit diff payload for an editor tab.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommitChanges {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub files: Vec<GitCommitFileChange>,
}

// --- Public operations -----------------------------------------------------

/// Reads the repository status for `root`.
///
/// Returns a snapshot with `is_repo = false` (rather than an error) when the
/// workspace is not a git work tree, so the UI can show an empty state.
pub fn status(root: &Path) -> NestResult<GitStatus> {
    if !inside_work_tree(root)? {
        return Ok(GitStatus::not_repo());
    }

    let branch = read_branch(root)?;
    let (ahead, behind, has_upstream) = read_upstream_counts(root);
    let output = git(root)
        .arg("status")
        .arg("--porcelain=1")
        .output()
        .map_err(|error| NestError::io(format!("git status failed: {error}")))?;
    if !output.status.success() {
        return Err(NestError::io(format!(
            "git status exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
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
        is_repo: true,
        branch,
        changes,
        ahead,
        behind,
        has_upstream,
    })
}

/// Stages one path (`git add <path>`).
pub fn stage(root: &Path, path: &str) -> NestResult<()> {
    run(git(root).arg("add").arg("--").arg(path), "stage")
}

/// Stages every change (`git add -A`).
pub fn stage_all(root: &Path) -> NestResult<()> {
    run(git(root).arg("add").arg("-A"), "stage all")
}

/// Unstages one path (`git restore --staged <path>`).
pub fn unstage(root: &Path, path: &str) -> NestResult<()> {
    run(
        git(root).arg("restore").arg("--staged").arg("--").arg(path),
        "unstage",
    )
}

/// Discards working-tree changes for one path (`git restore <path>`).
pub fn discard(root: &Path, path: &str) -> NestResult<()> {
    run(git(root).arg("restore").arg("--").arg(path), "discard")
}

/// Commits staged changes. Stages everything first when `stage_all` is set.
pub fn commit(root: &Path, message: &str, stage_all_first: bool) -> NestResult<()> {
    if message.trim().is_empty() {
        return Err(NestError::validation("commit message is empty"));
    }
    if stage_all_first {
        stage(root, ".")?;
    }
    run(git(root).arg("commit").arg("-m").arg(message), "commit")
}

/// Pushes the current branch to its upstream, or publishes it to `origin` on first push.
pub fn push(root: &Path) -> NestResult<()> {
    if !inside_work_tree(root)? {
        return Err(NestError::validation("not a git repository"));
    }
    let (_, _, has_upstream) = read_upstream_counts(root);
    if has_upstream {
        run(git(root).arg("push"), "push")
    } else {
        let branch = read_branch(root)?;
        run(
            git(root).args(["push", "-u", "origin", &branch]),
            "push",
        )
    }
}

/// Pulls from the current branch's upstream.
pub fn pull(root: &Path) -> NestResult<()> {
    if !inside_work_tree(root)? {
        return Err(NestError::validation("not a git repository"));
    }
    let (_, _, has_upstream) = read_upstream_counts(root);
    if !has_upstream {
        return Err(NestError::validation(
            "no upstream branch configured — push first to set upstream",
        ));
    }
    run(git(root).arg("pull"), "pull")
}

/// Reads up to `limit` recent commits for the graph section.
pub fn log(root: &Path, limit: u32) -> NestResult<Vec<GitCommit>> {
    if !inside_work_tree(root)? {
        return Ok(Vec::new());
    }
    // Unit separator (0x1f) between fields; one commit per line.
    let format = "%H%x1f%h%x1f%an%x1f%ae%x1f%ad%x1f%ar%x1f%P%x1f%s";
    let output = git(root)
        .arg("log")
        .arg(format!("-{limit}"))
        .arg(format!("--pretty=format:{format}"))
        .arg("--date=format:%Y-%m-%d %H:%M")
        .output()
        .map_err(|error| NestError::io(format!("git log failed: {error}")))?;
    if !output.status.success() {
        // A repo with no commits yet exits non-zero — treat as empty history.
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().filter_map(parse_log_line).collect())
}

/// Loads per-file diffs for a commit (Open Changes view).
pub fn commit_changes(root: &Path, hash: &str) -> NestResult<GitCommitChanges> {
    if !inside_work_tree(root)? {
        return Err(NestError::validation("not a git repository"));
    }
    if hash.trim().is_empty() {
        return Err(NestError::validation("commit hash is empty"));
    }

    let meta = capture(
        git(root).args(["show", "-s", "--format=%H%x1f%h%x1f%s", hash]),
        "show commit",
    )?;
    let meta_line = meta.lines().next().unwrap_or("").trim();
    let mut meta_parts = meta_line.splitn(3, '\u{1f}');
    let full_hash = meta_parts.next().unwrap_or("").trim().to_string();
    let short_hash = meta_parts.next().unwrap_or("").trim().to_string();
    let subject = meta_parts.next().unwrap_or("").trim().to_string();
    if full_hash.is_empty() {
        return Err(NestError::validation(format!("unknown commit: {hash}")));
    }

    let status_lines = capture(
        git(root).args(["show", "--name-status", "--format=", hash]),
        "show name-status",
    )?;
    let patch = capture(
        git(root).args(["show", "--format=", "--patch", "--no-color", hash]),
        "show patch",
    )?;
    let diff_by_path = split_unified_patch(&patch);

    let mut files = Vec::new();
    for line in status_lines.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(parsed) = parse_name_status_line(line) else {
            continue;
        };
        let diff = diff_by_path
            .get(&parsed.lookup_key)
            .cloned()
            .unwrap_or_default();
        files.push(GitCommitFileChange {
            path: parsed.path,
            old_path: parsed.old_path,
            status: parsed.status,
            diff,
        });
    }

    Ok(GitCommitChanges {
        hash: full_hash,
        short_hash,
        subject,
        files,
    })
}

// --- Parsing ---------------------------------------------------------------

fn parse_porcelain_line(line: &str) -> NestResult<GitChange> {
    let line = line.trim_end();
    if line.is_empty() {
        return Err(NestError::validation("empty porcelain line"));
    }

    if let Some(rest) = line.strip_prefix("?? ") {
        return Ok(GitChange {
            path: rest.trim().to_string(),
            staged: false,
            unstaged: true,
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

    Ok(GitChange {
        path,
        staged: index_status.is_some(),
        unstaged: worktree_status.is_some(),
        kind: ChangeKind::from_codes(index_status, worktree_status, false),
    })
}

fn parse_log_line(line: &str) -> Option<GitCommit> {
    let mut parts = line.splitn(8, '\u{1f}');
    let hash = parts.next()?.to_string();
    let short_hash = parts.next()?.to_string();
    let author = parts.next()?.to_string();
    let email = parts.next()?.to_string();
    let date = parts.next()?.to_string();
    let relative_date = parts.next()?.to_string();
    let parents = parts.next()?;
    let subject = parts.next()?.to_string();
    let parents = parents
        .split_whitespace()
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    Some(GitCommit {
        hash,
        short_hash,
        author,
        email,
        date,
        relative_date,
        subject,
        parents,
    })
}

fn parse_upstream_counts(stdout: &str) -> Option<(u32, u32)> {
    let mut parts = stdout.split_whitespace();
    let ahead = parts.next()?.parse().ok()?;
    let behind = parts.next()?.parse().ok()?;
    Some((ahead, behind))
}

struct ParsedNameStatus {
    path: String,
    old_path: Option<String>,
    status: String,
    lookup_key: String,
}

fn parse_name_status_line(line: &str) -> Option<ParsedNameStatus> {
    let mut parts = line.split('\t');
    let status_code = parts.next()?.trim();
    if status_code.is_empty() {
        return None;
    }
    let status = status_code
        .chars()
        .next()
        .map(|value| value.to_string())
        .unwrap_or_else(|| status_code.to_string());

    if status == "R" || status == "C" {
        let old_path = parts.next()?.trim().to_string();
        let new_path = parts.next()?.trim().to_string();
        return Some(ParsedNameStatus {
            lookup_key: new_path.clone(),
            path: new_path,
            old_path: Some(old_path),
            status,
        });
    }

    let path = parts.next()?.trim().to_string();
    Some(ParsedNameStatus {
        lookup_key: path.clone(),
        path,
        old_path: None,
        status,
    })
}

fn split_unified_patch(patch: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    let trimmed = patch.trim();
    if trimmed.is_empty() {
        return out;
    }

    for chunk in trimmed.split("\ndiff --git ") {
        if chunk.trim().is_empty() {
            continue;
        }
        let chunk = if chunk.starts_with("diff --git ") {
            chunk.to_string()
        } else {
            format!("diff --git {chunk}")
        };
        let Some(path) = path_from_diff_header(&chunk) else {
            continue;
        };
        out.insert(path, chunk);
    }
    out
}

fn path_from_diff_header(chunk: &str) -> Option<String> {
    let header = chunk.lines().next()?.trim();
    let rest = header.strip_prefix("diff --git ")?;
    let mut tokens = rest.split_whitespace();
    let old_path = tokens.next()?;
    let new_path = tokens.next()?;
    let old_path = strip_diff_path(old_path);
    let new_path = strip_diff_path(new_path);
    if new_path == "dev/null" {
        return Some(old_path);
    }
    Some(new_path)
}

fn strip_diff_path(token: &str) -> String {
    token.trim_start_matches("a/").trim_start_matches("b/").to_string()
}

// --- Git helpers -----------------------------------------------------------

fn read_branch(root: &Path) -> NestResult<String> {
    let output = git(root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .map_err(|error| NestError::io(format!("git branch failed: {error}")))?;
    if !output.status.success() {
        return Ok("HEAD".into());
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if branch.is_empty() {
        "HEAD".into()
    } else {
        branch
    })
}

fn read_upstream_counts(root: &Path) -> (u32, u32, bool) {
    let Ok(output) = git(root)
        .arg("rev-list")
        .arg("--left-right")
        .arg("--count")
        .arg("HEAD...@{upstream}")
        .output()
    else {
        return (0, 0, false);
    };
    if !output.status.success() {
        return (0, 0, false);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let (ahead, behind) = parse_upstream_counts(&stdout).unwrap_or((0, 0));
    (ahead, behind, true)
}

fn inside_work_tree(root: &Path) -> NestResult<bool> {
    let output = git(root)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map_err(|error| NestError::io(format!("git rev-parse failed: {error}")))?;
    if !output.status.success() {
        return Ok(false);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
}

fn git(root: &Path) -> Command {
    let mut command = Command::new("git");
    command.current_dir(root);
    command
}

fn run(command: &mut Command, action: &str) -> NestResult<()> {
    let output = command
        .output()
        .map_err(|error| NestError::io(format!("git {action} failed: {error}")))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if !stderr.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };
    Err(NestError::io(format!("git {action} failed: {detail}")))
}

fn capture(command: &mut Command, action: &str) -> NestResult<String> {
    let output = command
        .output()
        .map_err(|error| NestError::io(format!("git {action} failed: {error}")))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if !stderr.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };
    Err(NestError::io(format!("git {action} failed: {detail}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_staged_and_unstaged_pairs() {
        let both = parse_porcelain_line("MM file.rs").unwrap();
        assert_eq!(both.path, "file.rs");
        assert!(both.staged);
        assert!(both.unstaged);
        assert_eq!(both.kind, ChangeKind::Modified);

        let staged_only = parse_porcelain_line("A  new.rs").unwrap();
        assert!(staged_only.staged);
        assert!(!staged_only.unstaged);
        assert_eq!(staged_only.kind, ChangeKind::Added);

        let rename = parse_porcelain_line("R  old.rs -> new.rs").unwrap();
        assert_eq!(rename.path, "new.rs");
        assert_eq!(rename.kind, ChangeKind::Renamed);

        let untracked = parse_porcelain_line("?? scratch.txt").unwrap();
        assert!(!untracked.staged);
        assert!(untracked.unstaged);
        assert_eq!(untracked.kind, ChangeKind::Untracked);
    }

    #[test]
    fn parses_upstream_ahead_behind_counts() {
        assert_eq!(parse_upstream_counts("2\t0\n"), Some((2, 0)));
        assert_eq!(parse_upstream_counts("0 3"), Some((0, 3)));
        assert_eq!(parse_upstream_counts("bad"), None);
    }

    #[test]
    fn parses_log_line_with_parents() {
        let line = "abcd1234ef\u{1f}abcd123\u{1f}Jane Doe\u{1f}jane@example.com\u{1f}2026-07-04 10:30\u{1f}3 days ago\u{1f}f00 ba7\u{1f}Fix title bar";
        let commit = parse_log_line(line).unwrap();
        assert_eq!(commit.short_hash, "abcd123");
        assert_eq!(commit.author, "Jane Doe");
        assert_eq!(commit.email, "jane@example.com");
        assert_eq!(commit.relative_date, "3 days ago");
        assert_eq!(commit.parents, vec!["f00", "ba7"]);
        assert_eq!(commit.subject, "Fix title bar");
    }

    #[test]
    fn parses_name_status_lines() {
        let added = parse_name_status_line("A\tnew.rs").unwrap();
        assert_eq!(added.path, "new.rs");
        assert_eq!(added.status, "A");
        assert_eq!(added.old_path, None);

        let renamed = parse_name_status_line("R100\told.rs\tnew.rs").unwrap();
        assert_eq!(renamed.path, "new.rs");
        assert_eq!(renamed.old_path.as_deref(), Some("old.rs"));
        assert_eq!(renamed.status, "R");
    }

    #[test]
    fn splits_unified_patch_by_file() {
        let patch = "diff --git a/a.txt b/a.txt\n--- a/a.txt\n+++ b/a.txt\n@@\n-old\n+new\n\
                     diff --git a/b.txt b/b.txt\n--- a/b.txt\n+++ b/b.txt\n@@\n-x\n+y\n";
        let files = split_unified_patch(patch);
        assert_eq!(files.len(), 2);
        assert!(files.contains_key("a.txt"));
        assert!(files.contains_key("b.txt"));
    }
}
