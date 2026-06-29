//! Synchronous tool executor — runs locally on a background thread for the native-chat agent.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::tools::KiwiTool;

/// Result of executing a `KiwiTool`.
#[derive(Debug)]
pub enum ExecutionResult {
    /// Tool completed; wrap in `AgentToolResult`.
    Done { content: String, is_error: bool },
    /// Signal the service layer to route this command to the Terminal PTY.
    ShellRun { command: String },
}

/// Execute a tool synchronously. Called from a background thread (except `ShellRun`).
pub fn execute_tool(tool: &KiwiTool, repo_root: &Path) -> ExecutionResult {
    match tool {
        KiwiTool::FileRead { path } => read_file(path, repo_root),
        KiwiTool::FileWrite { path, content } => write_file(path, content, repo_root),
        KiwiTool::FileList { path, depth } => list_directory(path, *depth, repo_root),
        KiwiTool::ShellRun { command } => ExecutionResult::ShellRun {
            command: command.clone(),
        },
        KiwiTool::GitStatus => git_status(repo_root),
        KiwiTool::GitDiff { path } => git_diff(path.as_deref(), repo_root),
        KiwiTool::GitCommit { message, stage_all } => {
            git_commit(message, *stage_all, repo_root)
        }
        KiwiTool::FileSearch { query } => search_files(query, repo_root),
        KiwiTool::FileGrep { query, path } => search_content(query, path.as_deref(), repo_root),
    }
}

// ---------------------------------------------------------------------------
// Path safety
// ---------------------------------------------------------------------------

/// Join `repo_root` + `relative`, rejecting absolute paths and `..` traversal.
fn safe_join(repo_root: &Path, relative: &str) -> Result<PathBuf, String> {
    if Path::new(relative).is_absolute() {
        return Err("absolute paths are not allowed".to_string());
    }
    // Reject any path component that is ".." to prevent traversal.
    for component in Path::new(relative).components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err("path traversal ('..') is not allowed".to_string());
        }
    }
    Ok(repo_root.join(relative))
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

const MAX_FILE_BYTES: usize = 100_000; // 100 KB read limit
const MAX_SEARCH_RESULTS: usize = 100;

fn read_file(path: &str, repo_root: &Path) -> ExecutionResult {
    let full = match safe_join(repo_root, path) {
        Ok(p) => p,
        Err(e) => {
            return ExecutionResult::Done {
                content: e,
                is_error: true,
            }
        }
    };
    match fs::read_to_string(&full) {
        Ok(content) => {
            let content = if content.len() > MAX_FILE_BYTES {
                format!(
                    "{}\n\n[... output truncated at {} bytes ...]",
                    &content[..MAX_FILE_BYTES],
                    MAX_FILE_BYTES
                )
            } else {
                content
            };
            ExecutionResult::Done {
                content,
                is_error: false,
            }
        }
        Err(e) => ExecutionResult::Done {
            content: format!("Cannot read '{path}': {e}"),
            is_error: true,
        },
    }
}

fn write_file(path: &str, content: &str, repo_root: &Path) -> ExecutionResult {
    let full = match safe_join(repo_root, path) {
        Ok(p) => p,
        Err(e) => {
            return ExecutionResult::Done {
                content: e,
                is_error: true,
            }
        }
    };
    if let Some(parent) = full.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return ExecutionResult::Done {
                content: format!("Cannot create directories for '{path}': {e}"),
                is_error: true,
            };
        }
    }
    match fs::write(&full, content) {
        Ok(()) => ExecutionResult::Done {
            content: format!("Wrote {} bytes to '{path}'.", content.len()),
            is_error: false,
        },
        Err(e) => ExecutionResult::Done {
            content: format!("Cannot write '{path}': {e}"),
            is_error: true,
        },
    }
}

fn list_directory(path: &str, depth: u8, repo_root: &Path) -> ExecutionResult {
    let full = match safe_join(repo_root, path) {
        Ok(p) => p,
        Err(e) => {
            return ExecutionResult::Done {
                content: e,
                is_error: true,
            }
        }
    };

    let mut lines = Vec::new();
    let walker = walkdir::WalkDir::new(&full)
        .max_depth(depth as usize)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            e.file_name()
                .to_str()
                .map(|n| n != ".git" && n != "target")
                .unwrap_or(true)
        });

    for entry in walker.flatten() {
        if let Ok(rel) = entry.path().strip_prefix(&full) {
            if rel.as_os_str().is_empty() {
                continue;
            }
            let indent = "  ".repeat(entry.depth().saturating_sub(1));
            let name = entry.file_name().to_string_lossy();
            let suffix = if entry.file_type().is_dir() { "/" } else { "" };
            lines.push(format!("{indent}{name}{suffix}"));
        }
    }

    ExecutionResult::Done {
        content: if lines.is_empty() {
            "(empty directory)".to_string()
        } else {
            lines.join("\n")
        },
        is_error: false,
    }
}

fn git_status(repo_root: &Path) -> ExecutionResult {
    match Command::new("git")
        .args(["status", "--short"])
        .current_dir(repo_root)
        .output()
    {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            ExecutionResult::Done {
                content: if text.trim().is_empty() {
                    "No changes — working tree is clean.".to_string()
                } else {
                    text
                },
                is_error: false,
            }
        }
        Ok(out) => ExecutionResult::Done {
            content: String::from_utf8_lossy(&out.stderr).to_string(),
            is_error: true,
        },
        Err(e) => ExecutionResult::Done {
            content: format!("git not available: {e}"),
            is_error: true,
        },
    }
}

fn git_commit(message: &str, stage_all: bool, repo_root: &Path) -> ExecutionResult {
    if stage_all {
        match Command::new("git")
            .args(["add", "-A"])
            .current_dir(repo_root)
            .output()
        {
            Ok(out) if out.status.success() => {}
            Ok(out) => {
                return ExecutionResult::Done {
                    content: format_git_output(&out.stdout, &out.stderr),
                    is_error: true,
                };
            }
            Err(e) => {
                return ExecutionResult::Done {
                    content: format!("git not available: {e}"),
                    is_error: true,
                };
            }
        }
    }

    match Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(repo_root)
        .status()
    {
        Ok(status) if status.code() == Some(0) => {
            return ExecutionResult::Done {
                content: "Nothing to commit — working tree is clean.".to_string(),
                is_error: true,
            };
        }
        Ok(status) if status.code() == Some(1) => {}
        Ok(status) => {
            return ExecutionResult::Done {
                content: format!("git diff --cached failed (exit {:?})", status.code()),
                is_error: true,
            };
        }
        Err(e) => {
            return ExecutionResult::Done {
                content: format!("git not available: {e}"),
                is_error: true,
            };
        }
    }

    match Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(repo_root)
        .output()
    {
        Ok(out) if out.status.success() => {
            let hash = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(repo_root)
                .output()
                .ok()
                .filter(|out| out.status.success())
                .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string());

            let mut content = format_git_output(&out.stdout, &out.stderr);
            if let Some(hash) = hash {
                if !content.is_empty() {
                    content.push('\n');
                }
                content.push_str(&format!("commit: {hash}"));
            }
            ExecutionResult::Done {
                content,
                is_error: false,
            }
        }
        Ok(out) => ExecutionResult::Done {
            content: format_git_output(&out.stdout, &out.stderr),
            is_error: true,
        },
        Err(e) => ExecutionResult::Done {
            content: format!("git commit failed: {e}"),
            is_error: true,
        },
    }
}

fn format_git_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => String::new(),
    }
}

fn git_diff(path: Option<&str>, repo_root: &Path) -> ExecutionResult {
    let mut cmd = Command::new("git");
    cmd.arg("diff").arg("HEAD").current_dir(repo_root);
    if let Some(p) = path {
        cmd.arg("--").arg(p);
    }
    match cmd.output() {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            let content = if text.trim().is_empty() {
                "No uncommitted changes.".to_string()
            } else if text.len() > MAX_FILE_BYTES {
                format!(
                    "{}\n\n[... diff truncated at {} bytes ...]",
                    &text[..MAX_FILE_BYTES],
                    MAX_FILE_BYTES
                )
            } else {
                text
            };
            ExecutionResult::Done {
                content,
                is_error: !out.status.success(),
            }
        }
        Err(e) => ExecutionResult::Done {
            content: format!("git diff failed: {e}"),
            is_error: true,
        },
    }
}

fn search_files(query: &str, repo_root: &Path) -> ExecutionResult {
    let query_lower = query.to_lowercase();
    let mut matches: Vec<String> = Vec::new();

    let walker = walkdir::WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|e| {
            e.file_name()
                .to_str()
                .map(|n| n != ".git" && n != "target")
                .unwrap_or(true)
        });

    for entry in walker.flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.contains(&query_lower) {
            if let Ok(rel) = entry.path().strip_prefix(repo_root) {
                matches.push(rel.to_string_lossy().to_string());
            }
        }
        if matches.len() >= MAX_SEARCH_RESULTS {
            matches.push(format!("... (capped at {MAX_SEARCH_RESULTS} results)"));
            break;
        }
    }

    ExecutionResult::Done {
        content: if matches.is_empty() {
            format!("No files matching '{query}' found.")
        } else {
            matches.join("\n")
        },
        is_error: false,
    }
}

fn search_content(query: &str, path: Option<&str>, repo_root: &Path) -> ExecutionResult {
    let search_in = match path {
        Some(p) => match safe_join(repo_root, p) {
            Ok(full) => full,
            Err(e) => {
                return ExecutionResult::Done {
                    content: e,
                    is_error: true,
                }
            }
        },
        None => repo_root.to_path_buf(),
    };

    // Try ripgrep first, fall back to grep.
    let output = Command::new("rg")
        .args(["--no-heading", "-n", "--max-count=5", "-m", "100"])
        .arg(query)
        .arg(&search_in)
        .current_dir(repo_root)
        .output()
        .or_else(|_| {
            Command::new("grep")
                .args(["-r", "-n", "--max-count=100"])
                .arg(query)
                .arg(&search_in)
                .current_dir(repo_root)
                .output()
        });

    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            let content = if text.trim().is_empty() {
                format!("No matches for '{query}'.")
            } else if text.len() > MAX_FILE_BYTES {
                format!(
                    "{}\n\n[... output truncated at {} bytes ...]",
                    &text[..MAX_FILE_BYTES],
                    MAX_FILE_BYTES
                )
            } else {
                text
            };
            ExecutionResult::Done {
                content,
                is_error: false,
            }
        }
        Err(e) => ExecutionResult::Done {
            content: format!("Search failed: {e}"),
            is_error: true,
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::agent::tools::KiwiTool;

    fn temp_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        let _ = Command::new("git")
            .args(["init", "-q"])
            .current_dir(dir.path())
            .status();
        let _ = Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir.path())
            .status();
        let _ = Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(dir.path())
            .status();
        dir
    }

    #[test]
    fn read_file_success() {
        let dir = temp_repo();
        fs::write(dir.path().join("hello.txt"), "world").unwrap();

        let result = execute_tool(
            &KiwiTool::FileRead {
                path: "hello.txt".to_string(),
            },
            dir.path(),
        );
        assert!(
            matches!(result, ExecutionResult::Done { content, is_error: false } if content == "world")
        );
    }

    #[test]
    fn read_file_missing_returns_error() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::FileRead {
                path: "nope.txt".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done { is_error: true, .. }
        ));
    }

    #[test]
    fn write_file_creates_file() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::FileWrite {
                path: "new.txt".to_string(),
                content: "hello".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                is_error: false,
                ..
            }
        ));
        assert_eq!(
            fs::read_to_string(dir.path().join("new.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn write_file_creates_parent_dirs() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::FileWrite {
                path: "a/b/c.txt".to_string(),
                content: "deep".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                is_error: false,
                ..
            }
        ));
        assert!(dir.path().join("a/b/c.txt").exists());
    }

    #[test]
    fn path_traversal_blocked() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::FileRead {
                path: "../etc/passwd".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done { is_error: true, .. }
        ));
    }

    #[test]
    fn absolute_path_blocked() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::FileRead {
                path: "/etc/passwd".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done { is_error: true, .. }
        ));
    }

    #[test]
    fn run_bash_returns_run_bash_result() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::ShellRun {
                command: "echo hi".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(result, ExecutionResult::ShellRun { .. }));
    }

    #[test]
    fn list_directory_lists_files() {
        let dir = temp_repo();
        fs::write(dir.path().join("a.rs"), "").unwrap();
        fs::write(dir.path().join("b.rs"), "").unwrap();

        let result = execute_tool(
            &KiwiTool::FileList {
                path: ".".to_string(),
                depth: 1,
            },
            dir.path(),
        );
        assert!(
            matches!(result, ExecutionResult::Done { ref content, is_error: false } if content.contains("a.rs"))
        );
    }

    #[test]
    fn search_files_finds_match() {
        let dir = temp_repo();
        fs::write(dir.path().join("main.rs"), "").unwrap();
        fs::write(dir.path().join("lib.rs"), "").unwrap();

        let result = execute_tool(
            &KiwiTool::FileSearch {
                query: "main".to_string(),
            },
            dir.path(),
        );
        assert!(
            matches!(result, ExecutionResult::Done { ref content, is_error: false } if content.contains("main.rs"))
        );
    }

    #[test]
    fn search_files_no_match() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::FileSearch {
                query: "zzz_nonexistent".to_string(),
            },
            dir.path(),
        );
        assert!(
            matches!(result, ExecutionResult::Done { ref content, is_error: false } if content.contains("No files"))
        );
    }

    #[test]
    fn git_status_runs_without_panic() {
        let dir = temp_repo();
        // Just verify it doesn't panic — output depends on git availability.
        let _ = execute_tool(&KiwiTool::GitStatus, dir.path());
    }

    #[test]
    fn git_commit_stages_all_and_returns_hash() {
        let dir = temp_repo();
        fs::write(dir.path().join("tracked.txt"), "hello").unwrap();

        let result = execute_tool(
            &KiwiTool::GitCommit {
                message: "add tracked file".to_string(),
                stage_all: true,
            },
            dir.path(),
        );

        match result {
            ExecutionResult::Done { content, is_error: false } => {
                assert!(content.contains("commit:"));
            }
            other => panic!("expected successful commit, got {other:?}"),
        }
    }

    #[test]
    fn git_commit_clean_tree_returns_error() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::GitCommit {
                message: "empty".to_string(),
                stage_all: true,
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                is_error: true,
                ..
            }
        ));
    }
}
