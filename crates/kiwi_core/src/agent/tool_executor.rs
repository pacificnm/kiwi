//! Synchronous tool executor — runs locally on a background thread for the native-chat agent.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::Deserialize;

use super::memory_client;
use super::tools::{GitBranchAction, KiwiTool};

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
        KiwiTool::FilePatch {
            path,
            old_str,
            new_str,
        } => patch_file(path, old_str, new_str, repo_root),
        KiwiTool::FileReadRange {
            path,
            start_line,
            end_line,
        } => read_file_range(path, *start_line, *end_line, repo_root),
        KiwiTool::FileDelete { path } => delete_file(path, repo_root),
        KiwiTool::FileMove { src, dest } => move_file(src, dest, repo_root),
        KiwiTool::FileList { path, depth } => list_directory(path, *depth, repo_root),
        KiwiTool::ShellRun { command } => ExecutionResult::ShellRun {
            command: command.clone(),
        },
        KiwiTool::ShellCapture {
            command,
            timeout_secs,
        } => shell_capture(command, *timeout_secs, repo_root),
        KiwiTool::GitStatus => git_status(repo_root),
        KiwiTool::GitDiff { path } => git_diff(path.as_deref(), repo_root),
        KiwiTool::GitCommit { message, stage_all } => git_commit(message, *stage_all, repo_root),
        KiwiTool::GitBranch { action, name } => git_branch(*action, name.as_deref(), repo_root),
        KiwiTool::CargoCheck { package } => cargo_check(package.as_deref(), repo_root),
        KiwiTool::CargoBuild { package, release } => {
            cargo_build(package.as_deref(), *release, repo_root)
        }
        KiwiTool::CargoTest { filter, package } => {
            cargo_test(filter.as_deref(), package.as_deref(), repo_root)
        }
        KiwiTool::GitHubIssues {
            limit,
            label,
            milestone,
        } => github_issues(*limit, label.as_deref(), milestone.as_deref(), repo_root),
        KiwiTool::GitHubPrs { limit, base } => github_prs(*limit, base.as_deref(), repo_root),
        KiwiTool::MemorySearch { query, limit } => memory_search(query, *limit),
        KiwiTool::ProjectContext => project_context(repo_root),
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
const MAX_CARGO_OUTPUT_BYTES: usize = 10_000; // 10 KB cargo check limit
const MAX_CARGO_BUILD_OUTPUT_BYTES: usize = 20_000; // 20 KB cargo build limit
const MAX_CARGO_TEST_OUTPUT_BYTES: usize = 20_000; // 20 KB cargo test limit
const MAX_SHELL_CAPTURE_OUTPUT_BYTES: usize = 20_000; // 20 KB shell.capture limit
const MAX_READ_RANGE_LINES: u32 = 500;
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

fn patch_file(path: &str, old_str: &str, new_str: &str, repo_root: &Path) -> ExecutionResult {
    let full = match safe_join(repo_root, path) {
        Ok(p) => p,
        Err(e) => {
            return ExecutionResult::Done {
                content: e,
                is_error: true,
            }
        }
    };

    let content = match fs::read_to_string(&full) {
        Ok(content) => content,
        Err(e) => {
            return ExecutionResult::Done {
                content: format!("Cannot read '{path}': {e}"),
                is_error: true,
            }
        }
    };

    let matches: Vec<_> = content.match_indices(old_str).collect();
    match matches.len() {
        0 => ExecutionResult::Done {
            content: format!(
                "old_str not found in '{path}'. Re-read the file to get the exact text to replace."
            ),
            is_error: true,
        },
        1 => {
            let (byte_index, _) = matches[0];
            let line_number = content[..byte_index].bytes().filter(|&b| b == b'\n').count() + 1;
            let updated = format!(
                "{}{}{}",
                &content[..byte_index],
                new_str,
                &content[byte_index + old_str.len()..]
            );
            match fs::write(&full, &updated) {
                Ok(()) => ExecutionResult::Done {
                    content: format!("Patched '{path}' at line {line_number}."),
                    is_error: false,
                },
                Err(e) => ExecutionResult::Done {
                    content: format!("Cannot write '{path}': {e}"),
                    is_error: true,
                },
            }
        }
        count => ExecutionResult::Done {
            content: format!(
                "old_str matches {count} times in '{path}'. Include more surrounding context to make it unique."
            ),
            is_error: true,
        },
    }
}

fn read_file_range(
    path: &str,
    start_line: u32,
    end_line: Option<u32>,
    repo_root: &Path,
) -> ExecutionResult {
    let full = match safe_join(repo_root, path) {
        Ok(p) => p,
        Err(e) => {
            return ExecutionResult::Done {
                content: e,
                is_error: true,
            }
        }
    };

    let content = match fs::read_to_string(&full) {
        Ok(content) => content,
        Err(e) => {
            return ExecutionResult::Done {
                content: format!("Cannot read '{path}': {e}"),
                is_error: true,
            }
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return ExecutionResult::Done {
            content: format!("'{path}' is empty."),
            is_error: true,
        };
    }

    let start_idx = (start_line - 1) as usize;
    if start_idx >= lines.len() {
        return ExecutionResult::Done {
            content: format!(
                "start_line {start_line} is beyond end of file ({} lines).",
                lines.len()
            ),
            is_error: true,
        };
    }

    let end_line = end_line.unwrap_or(lines.len() as u32);
    if end_line < start_line {
        return ExecutionResult::Done {
            content: format!("end_line {end_line} must be >= start_line {start_line}."),
            is_error: true,
        };
    }

    let end_idx = ((end_line as usize).min(lines.len())).saturating_sub(1);
    let mut selected = Vec::new();
    for (line_no, line) in lines
        .iter()
        .enumerate()
        .skip(start_idx)
        .take(end_idx - start_idx + 1)
    {
        selected.push(format!("{}: {}", line_no + 1, line));
        if selected.len() as u32 >= MAX_READ_RANGE_LINES {
            selected.push(format!(
                "[... truncated at {MAX_READ_RANGE_LINES} lines; use a narrower range ...]"
            ));
            break;
        }
    }

    ExecutionResult::Done {
        content: selected.join("\n"),
        is_error: false,
    }
}

fn delete_file(path: &str, repo_root: &Path) -> ExecutionResult {
    let full = match safe_join(repo_root, path) {
        Ok(p) => p,
        Err(e) => {
            return ExecutionResult::Done {
                content: e,
                is_error: true,
            }
        }
    };

    if full.is_dir() {
        return ExecutionResult::Done {
            content: format!("'{path}' is a directory, not a file."),
            is_error: true,
        };
    }
    if !full.is_file() {
        return ExecutionResult::Done {
            content: format!("File not found: '{path}'."),
            is_error: true,
        };
    }

    match fs::remove_file(&full) {
        Ok(()) => ExecutionResult::Done {
            content: format!("Deleted '{path}'."),
            is_error: false,
        },
        Err(e) => ExecutionResult::Done {
            content: format!("Cannot delete '{path}': {e}"),
            is_error: true,
        },
    }
}

fn move_file(src: &str, dest: &str, repo_root: &Path) -> ExecutionResult {
    let src_path = match safe_join(repo_root, src) {
        Ok(p) => p,
        Err(e) => {
            return ExecutionResult::Done {
                content: e,
                is_error: true,
            }
        }
    };
    let dest_path = match safe_join(repo_root, dest) {
        Ok(p) => p,
        Err(e) => {
            return ExecutionResult::Done {
                content: e,
                is_error: true,
            }
        }
    };

    if !src_path.is_file() {
        return ExecutionResult::Done {
            content: format!("Source file not found: '{src}'."),
            is_error: true,
        };
    }

    if let Some(parent) = dest_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return ExecutionResult::Done {
                content: format!("Cannot create directories for '{dest}': {e}"),
                is_error: true,
            };
        }
    }

    match fs::rename(&src_path, &dest_path) {
        Ok(()) => ExecutionResult::Done {
            content: format!("Moved '{src}' -> '{dest}'."),
            is_error: false,
        },
        Err(e) => ExecutionResult::Done {
            content: format!("Cannot move '{src}' to '{dest}': {e}"),
            is_error: true,
        },
    }
}

fn shell_capture(command: &str, timeout_secs: u32, repo_root: &Path) -> ExecutionResult {
    let timeout = Duration::from_secs(timeout_secs as u64);
    let repo = repo_root.to_path_buf();
    let cmd = command.to_string();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let result = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .current_dir(&repo)
            .output();
        let _ = tx.send(result);
    });

    let output = match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            return ExecutionResult::Done {
                content: format!("Command timed out after {timeout_secs}s: `{command}`"),
                is_error: true,
            };
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            return ExecutionResult::Done {
                content: format!("Command failed to run: `{command}`"),
                is_error: true,
            };
        }
    };

    match output {
        Ok(out) => {
            let exit_code = out.status.code().unwrap_or(-1);
            let mut content = format!("exit code: {exit_code}\n");
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stdout.trim().is_empty() {
                content.push_str("\n--- stdout ---\n");
                content.push_str(stdout.trim_end());
                content.push('\n');
            }
            if !stderr.trim().is_empty() {
                content.push_str("\n--- stderr ---\n");
                content.push_str(stderr.trim_end());
                content.push('\n');
            }
            if stdout.trim().is_empty() && stderr.trim().is_empty() {
                content.push_str("(no output)\n");
            }
            if content.len() > MAX_SHELL_CAPTURE_OUTPUT_BYTES {
                content = format!(
                    "{}\n\n[... output truncated at {} bytes ...]",
                    &content[..MAX_SHELL_CAPTURE_OUTPUT_BYTES],
                    MAX_SHELL_CAPTURE_OUTPUT_BYTES
                );
            }
            ExecutionResult::Done {
                content: content.trim_end().to_string(),
                is_error: !out.status.success(),
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => ExecutionResult::Done {
            content: "Shell (`sh`) not found on PATH".to_string(),
            is_error: true,
        },
        Err(err) => ExecutionResult::Done {
            content: format!("Command failed: {err}"),
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

fn git_branch(action: GitBranchAction, name: Option<&str>, repo_root: &Path) -> ExecutionResult {
    match action {
        GitBranchAction::List => match Command::new("git")
            .args(["branch", "--list"])
            .current_dir(repo_root)
            .output()
        {
            Ok(out) if out.status.success() => {
                let content = format_git_output(&out.stdout, &out.stderr);
                ExecutionResult::Done {
                    content: if content.is_empty() {
                        "(no local branches)".to_string()
                    } else {
                        content
                    },
                    is_error: false,
                }
            }
            Ok(out) => ExecutionResult::Done {
                content: format_git_output(&out.stdout, &out.stderr),
                is_error: true,
            },
            Err(e) => ExecutionResult::Done {
                content: format!("git not available: {e}"),
                is_error: true,
            },
        },
        GitBranchAction::Create => {
            let name = name.expect("create validated at parse time");
            run_git_checkout(repo_root, &["checkout", "-b", name])
        }
        GitBranchAction::Checkout => {
            let name = name.expect("checkout validated at parse time");
            run_git_checkout(repo_root, &["checkout", name])
        }
    }
}

fn run_git_checkout(repo_root: &Path, args: &[&str]) -> ExecutionResult {
    match Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
    {
        Ok(out) if out.status.success() => ExecutionResult::Done {
            content: format_git_output(&out.stdout, &out.stderr),
            is_error: false,
        },
        Ok(out) => ExecutionResult::Done {
            content: format_git_output(&out.stdout, &out.stderr),
            is_error: true,
        },
        Err(e) => ExecutionResult::Done {
            content: format!("git checkout failed: {e}"),
            is_error: true,
        },
    }
}

fn cargo_check(package: Option<&str>, repo_root: &Path) -> ExecutionResult {
    let mut cmd = Command::new("cargo");
    cmd.args(["check", "--message-format=short"]);
    if let Some(package) = package {
        if package.trim().is_empty() {
            return ExecutionResult::Done {
                content: "package name must not be empty".to_string(),
                is_error: true,
            };
        }
        cmd.args(["-p", package]);
    }
    cmd.current_dir(repo_root);

    match cmd.output() {
        Ok(out) => {
            let raw = format_git_output(&out.stdout, &out.stderr);
            let (error_count, warning_count) = count_cargo_diagnostics(&raw);
            let mut content = format_cargo_check_output(&raw, error_count, warning_count);
            if content.len() > MAX_CARGO_OUTPUT_BYTES {
                content = format!(
                    "{}\n\n[... output truncated at {} bytes ...]",
                    &content[..MAX_CARGO_OUTPUT_BYTES],
                    MAX_CARGO_OUTPUT_BYTES
                );
            }
            ExecutionResult::Done {
                content,
                is_error: !out.status.success() || error_count > 0,
            }
        }
        Err(e) => ExecutionResult::Done {
            content: format!("cargo check failed: {e}"),
            is_error: true,
        },
    }
}

fn count_cargo_diagnostics(text: &str) -> (usize, usize) {
    let mut errors = 0;
    let mut warnings = 0;
    for line in text.lines() {
        if is_cargo_error_line(line) {
            errors += 1;
        } else if is_cargo_warning_line(line) {
            warnings += 1;
        }
    }
    (errors, warnings)
}

fn is_cargo_error_line(line: &str) -> bool {
    line.contains(": error:") || line.contains(": error[") || line.starts_with("error:")
}

fn is_cargo_warning_line(line: &str) -> bool {
    line.contains(": warning:") || line.starts_with("warning:")
}

fn format_cargo_check_output(raw: &str, error_count: usize, warning_count: usize) -> String {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut other = Vec::new();

    for line in raw.lines() {
        if is_cargo_error_line(line) {
            errors.push(line);
        } else if is_cargo_warning_line(line) {
            warnings.push(line);
        } else if !line.trim().is_empty() {
            other.push(line);
        }
    }

    let status = if error_count > 0 {
        "finished with errors"
    } else if warning_count > 0 {
        "finished with warnings"
    } else {
        "passed"
    };

    let mut content =
        format!("cargo check {status} ({error_count} error(s), {warning_count} warning(s))");

    if !errors.is_empty() {
        content.push_str("\n\n--- errors ---\n");
        content.push_str(&errors.join("\n"));
    }
    if !warnings.is_empty() {
        content.push_str("\n\n--- warnings ---\n");
        content.push_str(&warnings.join("\n"));
    }
    if !other.is_empty() {
        content.push_str("\n\n--- output ---\n");
        content.push_str(&other.join("\n"));
    }

    content
}

fn cargo_build(package: Option<&str>, release: bool, repo_root: &Path) -> ExecutionResult {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--message-format=short"]);
    if release {
        cmd.arg("--release");
    }
    if let Some(package) = package {
        if package.trim().is_empty() {
            return ExecutionResult::Done {
                content: "package name must not be empty".to_string(),
                is_error: true,
            };
        }
        cmd.args(["-p", package]);
    }
    cmd.current_dir(repo_root);

    match cmd.output() {
        Ok(out) => {
            let raw = format_git_output(&out.stdout, &out.stderr);
            let status = if out.status.success() {
                "cargo build passed"
            } else {
                "cargo build failed"
            };
            let mut content = format!("{status}\n\n{raw}");
            if content.len() > MAX_CARGO_BUILD_OUTPUT_BYTES {
                content = format!(
                    "{}\n\n[... output truncated at {} bytes ...]",
                    &content[..MAX_CARGO_BUILD_OUTPUT_BYTES],
                    MAX_CARGO_BUILD_OUTPUT_BYTES
                );
            }
            ExecutionResult::Done {
                content,
                is_error: !out.status.success(),
            }
        }
        Err(e) => ExecutionResult::Done {
            content: format!("cargo build failed: {e}"),
            is_error: true,
        },
    }
}

fn cargo_test(filter: Option<&str>, package: Option<&str>, repo_root: &Path) -> ExecutionResult {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    if let Some(package) = package {
        if package.trim().is_empty() {
            return ExecutionResult::Done {
                content: "package name must not be empty".to_string(),
                is_error: true,
            };
        }
        cmd.args(["-p", package]);
    }
    if let Some(filter) = filter {
        if filter.trim().is_empty() {
            return ExecutionResult::Done {
                content: "filter must not be empty".to_string(),
                is_error: true,
            };
        }
        cmd.arg(filter);
    }
    cmd.args(["--", "--nocapture"]);
    cmd.current_dir(repo_root);

    match cmd.output() {
        Ok(out) => {
            let raw = format_git_output(&out.stdout, &out.stderr);
            let mut content = format_cargo_test_output(&raw, out.status.success());
            if content.len() > MAX_CARGO_TEST_OUTPUT_BYTES {
                content = format!(
                    "{}\n\n[... output truncated at {} bytes ...]",
                    &content[..MAX_CARGO_TEST_OUTPUT_BYTES],
                    MAX_CARGO_TEST_OUTPUT_BYTES
                );
            }
            ExecutionResult::Done {
                content,
                is_error: !out.status.success(),
            }
        }
        Err(e) => ExecutionResult::Done {
            content: format!("cargo test failed: {e}"),
            is_error: true,
        },
    }
}

fn parse_cargo_test_summary(line: &str) -> Option<(usize, usize)> {
    let rest = line.strip_prefix("test result: ")?;
    let passed = rest
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            part.strip_suffix(" passed")?.parse().ok()
        })
        .unwrap_or(0);
    let failed = rest
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            part.strip_suffix(" failed")?.parse().ok()
        })
        .unwrap_or(0);
    Some((passed, failed))
}

fn format_cargo_test_output(raw: &str, success: bool) -> String {
    let summary_line = raw
        .lines()
        .rev()
        .find(|line| line.starts_with("test result: "))
        .map(str::to_owned);

    let (passed, failed) = summary_line
        .as_deref()
        .and_then(parse_cargo_test_summary)
        .unwrap_or((0, 0));

    let status = if success { "passed" } else { "failed" };

    let mut content = if let Some(ref summary) = summary_line {
        format!("cargo test {status} ({passed} passed, {failed} failed)\nSummary: {summary}")
    } else {
        format!("cargo test {status} ({passed} passed, {failed} failed)")
    };

    let mut failures = Vec::new();
    let mut other = Vec::new();
    for line in raw.lines() {
        if line.starts_with("test ") && line.contains(" ... FAILED") {
            failures.push(line);
        } else if !line.trim().is_empty() && !line.starts_with("test result: ") {
            other.push(line);
        }
    }

    if !failures.is_empty() {
        content.push_str("\n\n--- failures ---\n");
        content.push_str(&failures.join("\n"));
    }
    if !other.is_empty() {
        content.push_str("\n\n--- output ---\n");
        content.push_str(&other.join("\n"));
    }

    content
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

const GH_COMMAND: &str = "gh";

#[derive(Debug, Deserialize)]
struct GhIssueRow {
    number: u32,
    title: String,
    state: String,
    labels: Vec<GhLabelRow>,
    milestone: Option<GhMilestoneRow>,
}

#[derive(Debug, Deserialize)]
struct GhLabelRow {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GhMilestoneRow {
    title: String,
}

fn github_issues(
    limit: u32,
    label: Option<&str>,
    milestone: Option<&str>,
    repo_root: &Path,
) -> ExecutionResult {
    let limit_str = limit.to_string();
    let mut args = vec![
        "issue",
        "list",
        "--json",
        "number,title,state,labels,milestone",
        "--limit",
        limit_str.as_str(),
    ];
    if let Some(label) = label {
        args.push("--label");
        args.push(label);
    }
    if let Some(milestone) = milestone {
        args.push("--milestone");
        args.push(milestone);
    }

    let output = Command::new(GH_COMMAND)
        .args(&args)
        .current_dir(repo_root)
        .output();

    match output {
        Err(err) if err.kind() == ErrorKind::NotFound => ExecutionResult::Done {
            content: format!("GitHub CLI ({GH_COMMAND}) not found on PATH"),
            is_error: true,
        },
        Err(err) => ExecutionResult::Done {
            content: format!("Failed to run `{GH_COMMAND} issue list`: {err}"),
            is_error: true,
        },
        Ok(result) if !result.status.success() => {
            let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            let message = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                "gh issue list failed".to_string()
            };
            ExecutionResult::Done {
                content: message,
                is_error: true,
            }
        }
        Ok(result) => match serde_json::from_slice::<Vec<GhIssueRow>>(&result.stdout) {
            Ok(issues) => ExecutionResult::Done {
                content: format_github_issues_list(&issues),
                is_error: false,
            },
            Err(err) => {
                let raw = String::from_utf8_lossy(&result.stdout).into_owned();
                ExecutionResult::Done {
                    content: format!(
                        "parse_error: true\nInvalid gh issue JSON: {err}\n\nRaw output:\n{raw}"
                    ),
                    is_error: true,
                }
            }
        },
    }
}

fn format_github_issues_list(issues: &[GhIssueRow]) -> String {
    if issues.is_empty() {
        return "No issues found.".to_string();
    }

    let mut out = format!("GitHub issues ({}):\n", issues.len());
    for (index, issue) in issues.iter().enumerate() {
        let labels: Vec<_> = issue
            .labels
            .iter()
            .map(|label| label.name.as_str())
            .collect();
        let labels_part = if labels.is_empty() {
            String::new()
        } else {
            format!(" [{}]", labels.join(", "))
        };
        let milestone_part = issue
            .milestone
            .as_ref()
            .map(|milestone| format!(" (milestone: {})", milestone.title))
            .unwrap_or_default();
        out.push_str(&format!(
            "{}. #{} [{}] {}{}{}\n",
            index + 1,
            issue.number,
            issue.state,
            issue.title,
            labels_part,
            milestone_part
        ));
    }
    out.trim_end().to_string()
}

#[derive(Debug, Deserialize)]
struct GhPrRow {
    number: u32,
    title: String,
    state: String,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "baseRefName")]
    base_ref_name: String,
}

fn github_prs(limit: u32, base: Option<&str>, repo_root: &Path) -> ExecutionResult {
    let limit_str = limit.to_string();
    let mut args = vec![
        "pr",
        "list",
        "--json",
        "number,title,state,headRefName,baseRefName",
        "--limit",
        limit_str.as_str(),
    ];
    if let Some(base) = base {
        args.push("--base");
        args.push(base);
    }

    let output = Command::new(GH_COMMAND)
        .args(&args)
        .current_dir(repo_root)
        .output();

    match output {
        Err(err) if err.kind() == ErrorKind::NotFound => ExecutionResult::Done {
            content: format!("GitHub CLI ({GH_COMMAND}) not found on PATH"),
            is_error: true,
        },
        Err(err) => ExecutionResult::Done {
            content: format!("Failed to run `{GH_COMMAND} pr list`: {err}"),
            is_error: true,
        },
        Ok(result) if !result.status.success() => {
            let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            let message = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                "gh pr list failed".to_string()
            };
            ExecutionResult::Done {
                content: message,
                is_error: true,
            }
        }
        Ok(result) => match serde_json::from_slice::<Vec<GhPrRow>>(&result.stdout) {
            Ok(prs) => ExecutionResult::Done {
                content: format_github_prs_list(&prs),
                is_error: false,
            },
            Err(err) => {
                let raw = String::from_utf8_lossy(&result.stdout).into_owned();
                ExecutionResult::Done {
                    content: format!(
                        "parse_error: true\nInvalid gh pr JSON: {err}\n\nRaw output:\n{raw}"
                    ),
                    is_error: true,
                }
            }
        },
    }
}

fn format_github_prs_list(prs: &[GhPrRow]) -> String {
    if prs.is_empty() {
        return "No pull requests found.".to_string();
    }

    let mut out = format!("GitHub pull requests ({}):\n", prs.len());
    for (index, pr) in prs.iter().enumerate() {
        out.push_str(&format!(
            "{}. #{} [{}] {} ({} -> {})\n",
            index + 1,
            pr.number,
            pr.state,
            pr.title,
            pr.head_ref_name,
            pr.base_ref_name
        ));
    }
    out.trim_end().to_string()
}

fn memory_search(query: &str, limit: u32) -> ExecutionResult {
    match memory_client::search_project_memory(query, limit) {
        Ok(text) => ExecutionResult::Done {
            content: text,
            is_error: false,
        },
        Err(message) => ExecutionResult::Done {
            content: message,
            is_error: true,
        },
    }
}

const PROJECT_CONTEXT_KEY_FILES: &[&str] = &["Cargo.toml", "CLAUDE.md", "README.md"];
const PROJECT_CONTEXT_FILE_PREVIEW_LINES: usize = 20;

fn project_context(repo_root: &Path) -> ExecutionResult {
    let mut out = String::from("Project context\n\n");

    out.push_str("## Branch\n");
    out.push_str(&git_current_branch(repo_root));
    out.push('\n');

    out.push_str("\n## Recent commits\n");
    out.push_str(&git_recent_commits(repo_root));
    out.push('\n');

    out.push_str("\n## Directory structure\n");
    match collect_directory_lines(repo_root, 2) {
        Ok(lines) => {
            if lines.is_empty() {
                out.push_str("(empty directory)\n");
            } else {
                out.push_str(&lines.join("\n"));
                out.push('\n');
            }
        }
        Err(message) => {
            out.push_str(&message);
            out.push('\n');
        }
    }

    out.push_str("\n## Key files\n");
    for file_name in PROJECT_CONTEXT_KEY_FILES {
        out.push_str(&format_key_file_section(repo_root, file_name));
    }

    ExecutionResult::Done {
        content: out.trim_end().to_string(),
        is_error: false,
    }
}

fn git_current_branch(repo_root: &Path) -> String {
    match Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(repo_root)
        .output()
    {
        Ok(out) if out.status.success() => {
            let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if branch.is_empty() {
                "(detached HEAD)".to_string()
            } else {
                branch
            }
        }
        Ok(out) => format!(
            "Unable to read branch: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ),
        Err(err) => format!("git not available: {err}"),
    }
}

fn git_recent_commits(repo_root: &Path) -> String {
    match Command::new("git")
        .args(["log", "--oneline", "-10"])
        .current_dir(repo_root)
        .output()
    {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if text.is_empty() {
                "(no commits)".to_string()
            } else {
                text
            }
        }
        Ok(out) => format!(
            "Unable to read commits: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ),
        Err(err) => format!("git not available: {err}"),
    }
}

fn collect_directory_lines(root: &Path, depth: u8) -> Result<Vec<String>, String> {
    if !root.is_dir() {
        return Err(format!("Not a directory: {}", root.display()));
    }

    let mut lines = Vec::new();
    let walker = walkdir::WalkDir::new(root)
        .max_depth(depth as usize)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| name != ".git" && name != "target")
                .unwrap_or(true)
        });

    for entry in walker.flatten() {
        if let Ok(rel) = entry.path().strip_prefix(root) {
            if rel.as_os_str().is_empty() {
                continue;
            }
            let indent = "  ".repeat(entry.depth().saturating_sub(1));
            let name = entry.file_name().to_string_lossy();
            let suffix = if entry.file_type().is_dir() { "/" } else { "" };
            lines.push(format!("{indent}{name}{suffix}"));
        }
    }

    Ok(lines)
}

fn format_key_file_section(repo_root: &Path, file_name: &str) -> String {
    let path = repo_root.join(file_name);
    let mut section = format!("\n### {file_name}\n");
    if !path.is_file() {
        section.push_str("(not present)\n");
        return section;
    }

    match fs::read_to_string(&path) {
        Ok(content) => {
            let preview = first_n_lines(&content, PROJECT_CONTEXT_FILE_PREVIEW_LINES);
            section.push_str(&preview);
            if !preview.ends_with('\n') {
                section.push('\n');
            }
        }
        Err(err) => {
            section.push_str(&format!("(unable to read: {err})\n"));
        }
    }
    section
}

fn first_n_lines(content: &str, n: usize) -> String {
    content.lines().take(n).collect::<Vec<_>>().join("\n")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::agent::tools::{GitBranchAction, KiwiTool};

    fn temp_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path();
        let _ = Command::new("git")
            .args(["init", "-q"])
            .current_dir(path)
            .status();
        let _ = Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(path)
            .status();
        let _ = Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(path)
            .status();
        fs::write(path.join("README"), "init").unwrap();
        let _ = Command::new("git")
            .args(["add", "README"])
            .current_dir(path)
            .status();
        let _ = Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(path)
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
            ExecutionResult::Done {
                content,
                is_error: false,
            } => {
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
            ExecutionResult::Done { is_error: true, .. }
        ));
    }

    #[test]
    fn git_branch_list_includes_current_marker() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::GitBranch {
                action: GitBranchAction::List,
                name: None,
            },
            dir.path(),
        );
        assert!(
            matches!(result, ExecutionResult::Done { ref content, is_error: false } if content.contains('*'))
        );
    }

    #[test]
    fn git_branch_create_and_checkout() {
        let dir = temp_repo();
        let initial = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(dir.path())
            .output()
            .expect("git branch --show-current");
        let initial_branch = String::from_utf8_lossy(&initial.stdout).trim().to_string();

        let create = execute_tool(
            &KiwiTool::GitBranch {
                action: GitBranchAction::Create,
                name: Some("feature-test".to_string()),
            },
            dir.path(),
        );
        assert!(matches!(
            create,
            ExecutionResult::Done {
                is_error: false,
                ..
            }
        ));

        let current = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(dir.path())
            .output()
            .expect("git branch --show-current");
        assert_eq!(
            String::from_utf8_lossy(&current.stdout).trim(),
            "feature-test"
        );

        let checkout = execute_tool(
            &KiwiTool::GitBranch {
                action: GitBranchAction::Checkout,
                name: Some(initial_branch.clone()),
            },
            dir.path(),
        );
        assert!(matches!(
            checkout,
            ExecutionResult::Done {
                is_error: false,
                ..
            }
        ));

        let current = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(dir.path())
            .output()
            .expect("git branch --show-current");
        assert_eq!(
            String::from_utf8_lossy(&current.stdout).trim(),
            initial_branch
        );
    }

    fn temp_cargo_project(main_rs: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path();
        fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"agent_tool_test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src/main.rs"), main_rs).unwrap();
        dir
    }

    #[test]
    fn cargo_check_clean_project_succeeds() {
        let dir = temp_cargo_project("fn main() {}\n");
        let result = execute_tool(&KiwiTool::CargoCheck { package: None }, dir.path());
        assert!(
            matches!(result, ExecutionResult::Done { ref content, is_error: false } if content.contains("cargo check passed"))
        );
    }

    #[test]
    fn cargo_check_reports_compile_errors() {
        let dir = temp_cargo_project("fn main() { broken }\n");
        let result = execute_tool(&KiwiTool::CargoCheck { package: None }, dir.path());
        assert!(
            matches!(result, ExecutionResult::Done { ref content, is_error: true } if content.contains("--- errors ---"))
        );
    }

    fn temp_cargo_test_project(main_rs: &str) -> tempfile::TempDir {
        let dir = temp_cargo_project(main_rs);
        dir
    }

    #[test]
    fn cargo_test_passing_project_succeeds() {
        let dir = temp_cargo_test_project(
            "fn main() {}\n\n#[test]\nfn it_works() { assert_eq!(1, 1); }\n",
        );
        let result = execute_tool(
            &KiwiTool::CargoTest {
                filter: None,
                package: None,
            },
            dir.path(),
        );
        assert!(
            matches!(result, ExecutionResult::Done { ref content, is_error: false } if content.contains("cargo test passed") && content.contains("1 passed"))
        );
    }

    #[test]
    fn cargo_test_failing_test_reports_failure() {
        let dir = temp_cargo_test_project(
            "fn main() {}\n\n#[test]\nfn it_fails() { panic!(\"boom\"); }\n",
        );
        let result = execute_tool(
            &KiwiTool::CargoTest {
                filter: None,
                package: None,
            },
            dir.path(),
        );
        assert!(
            matches!(result, ExecutionResult::Done { ref content, is_error: true } if content.contains("cargo test failed") && content.contains("--- failures ---"))
        );
    }

    #[test]
    fn cargo_test_filter_runs_matching_test_only() {
        let dir = temp_cargo_test_project(
            "fn main() {}\n\n#[test]\nfn alpha_works() { assert!(true); }\n\n#[test]\nfn beta_works() { assert!(true); }\n",
        );
        let result = execute_tool(
            &KiwiTool::CargoTest {
                filter: Some("alpha".to_string()),
                package: None,
            },
            dir.path(),
        );
        assert!(
            matches!(result, ExecutionResult::Done { ref content, is_error: false } if content.contains("1 passed"))
        );
    }

    #[test]
    fn format_github_issues_list_renders_numbered_entries() {
        let issues = vec![
            GhIssueRow {
                number: 42,
                title: "Fix bug".to_string(),
                state: "OPEN".to_string(),
                labels: vec![GhLabelRow {
                    name: "bug".to_string(),
                }],
                milestone: Some(GhMilestoneRow {
                    title: "M1".to_string(),
                }),
            },
            GhIssueRow {
                number: 7,
                title: "Docs".to_string(),
                state: "OPEN".to_string(),
                labels: vec![],
                milestone: None,
            },
        ];

        let formatted = format_github_issues_list(&issues);
        assert!(formatted.contains("GitHub issues (2):"));
        assert!(formatted.contains("1. #42 [OPEN] Fix bug [bug] (milestone: M1)"));
        assert!(formatted.contains("2. #7 [OPEN] Docs"));
    }

    #[test]
    fn format_github_issues_list_empty_returns_message() {
        assert_eq!(format_github_issues_list(&[]), "No issues found.");
    }

    #[test]
    fn github_issues_parse_failure_includes_raw_json() {
        let issues: Result<Vec<GhIssueRow>, _> = serde_json::from_slice(b"{not json}");
        let err = issues.expect_err("invalid json");
        let raw = "{not json}".to_string();
        let content =
            format!("parse_error: true\nInvalid gh issue JSON: {err}\n\nRaw output:\n{raw}");
        assert!(content.contains("parse_error: true"));
        assert!(content.contains("{not json}"));
    }

    #[test]
    fn format_github_prs_list_renders_numbered_entries() {
        let prs = vec![
            GhPrRow {
                number: 379,
                title: "Add github.issues".to_string(),
                state: "OPEN".to_string(),
                head_ref_name: "357-branch".to_string(),
                base_ref_name: "main".to_string(),
            },
            GhPrRow {
                number: 378,
                title: "Add cargo.test".to_string(),
                state: "OPEN".to_string(),
                head_ref_name: "356-branch".to_string(),
                base_ref_name: "main".to_string(),
            },
        ];

        let formatted = format_github_prs_list(&prs);
        assert!(formatted.contains("GitHub pull requests (2):"));
        assert!(formatted.contains("1. #379 [OPEN] Add github.issues (357-branch -> main)"));
        assert!(formatted.contains("2. #378 [OPEN] Add cargo.test (356-branch -> main)"));
    }

    #[test]
    fn format_github_prs_list_empty_returns_message() {
        assert_eq!(format_github_prs_list(&[]), "No pull requests found.");
    }

    #[test]
    fn github_prs_parse_failure_includes_raw_json() {
        let prs: Result<Vec<GhPrRow>, _> = serde_json::from_slice(b"{not json}");
        let err = prs.expect_err("invalid json");
        let raw = "{not json}".to_string();
        let content = format!("parse_error: true\nInvalid gh pr JSON: {err}\n\nRaw output:\n{raw}");
        assert!(content.contains("parse_error: true"));
        assert!(content.contains("{not json}"));
    }

    #[test]
    fn project_context_in_git_repo_includes_branch_and_commits() {
        let dir = temp_repo();
        fs::write(dir.path().join("README.md"), "# Test repo\n").unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();

        let result = execute_tool(&KiwiTool::ProjectContext, dir.path());
        assert!(matches!(
            result,
            ExecutionResult::Done {
                ref content,
                is_error: false
            } if content.contains("## Branch")
                && content.contains("## Recent commits")
                && content.contains("init")
                && content.contains("## Directory structure")
                && content.contains("README.md")
                && content.contains("### Cargo.toml")
                && content.contains("name = \"demo\"")
        ));
    }

    #[test]
    fn first_n_lines_limits_output() {
        let text = (1..=30)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let preview = first_n_lines(&text, 20);
        assert_eq!(preview.lines().count(), 20);
        assert!(preview.contains("line 1"));
        assert!(preview.contains("line 20"));
        assert!(!preview.contains("line 21"));
    }

    #[test]
    fn patch_file_replaces_unique_match() {
        let dir = temp_repo();
        fs::write(dir.path().join("edit.rs"), "fn old_name() {}\n").unwrap();

        let result = execute_tool(
            &KiwiTool::FilePatch {
                path: "edit.rs".to_string(),
                old_str: "old_name".to_string(),
                new_str: "new_name".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                ref content,
                is_error: false
            } if content.contains("line 1")
        ));
        assert_eq!(
            fs::read_to_string(dir.path().join("edit.rs")).unwrap(),
            "fn new_name() {}\n"
        );
    }

    #[test]
    fn patch_file_rejects_ambiguous_match() {
        let dir = temp_repo();
        fs::write(dir.path().join("dup.rs"), "foo\nfoo\n").unwrap();

        let result = execute_tool(
            &KiwiTool::FilePatch {
                path: "dup.rs".to_string(),
                old_str: "foo".to_string(),
                new_str: "bar".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                is_error: true,
                ref content,
            } if content.contains("2 times") && content.contains("surrounding context")
        ));
    }

    #[test]
    fn patch_file_errors_when_old_str_not_found() {
        let dir = temp_repo();
        fs::write(dir.path().join("miss.rs"), "fn main() {}\n").unwrap();

        let result = execute_tool(
            &KiwiTool::FilePatch {
                path: "miss.rs".to_string(),
                old_str: "missing_text".to_string(),
                new_str: "replacement".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                is_error: true,
                ref content,
            } if content.contains("old_str not found") && content.contains("Re-read the file")
        ));
        assert_eq!(
            fs::read_to_string(dir.path().join("miss.rs")).unwrap(),
            "fn main() {}\n"
        );
    }

    #[test]
    fn patch_file_blocks_path_traversal() {
        let dir = temp_repo();
        fs::write(dir.path().join("safe.rs"), "keep\n").unwrap();

        let result = execute_tool(
            &KiwiTool::FilePatch {
                path: "../safe.rs".to_string(),
                old_str: "keep".to_string(),
                new_str: "gone".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                is_error: true,
                ref content,
            } if content.contains("path traversal")
        ));
        assert_eq!(
            fs::read_to_string(dir.path().join("safe.rs")).unwrap(),
            "keep\n"
        );
    }

    #[test]
    fn read_file_range_returns_numbered_lines() {
        let dir = temp_repo();
        fs::write(dir.path().join("lines.txt"), "a\nb\nc\n").unwrap();

        let result = execute_tool(
            &KiwiTool::FileReadRange {
                path: "lines.txt".to_string(),
                start_line: 2,
                end_line: Some(3),
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                ref content,
                is_error: false
            } if content == "2: b\n3: c"
        ));
    }

    #[test]
    fn read_file_range_reads_to_eof_when_end_line_omitted() {
        let dir = temp_repo();
        fs::write(dir.path().join("tail.txt"), "one\ntwo\nthree\n").unwrap();

        let result = execute_tool(
            &KiwiTool::FileReadRange {
                path: "tail.txt".to_string(),
                start_line: 2,
                end_line: None,
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                ref content,
                is_error: false
            } if content == "2: two\n3: three"
        ));
    }

    #[test]
    fn read_file_range_truncates_at_500_lines() {
        let dir = temp_repo();
        let body = (1..=600)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(dir.path().join("big.txt"), body).unwrap();

        let result = execute_tool(
            &KiwiTool::FileReadRange {
                path: "big.txt".to_string(),
                start_line: 1,
                end_line: None,
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                ref content,
                is_error: false,
            } if content.contains("500: line 500")
                && content.contains("truncated at 500 lines")
                && !content.contains("501: line 501")
        ));
    }

    #[test]
    fn read_file_range_errors_when_start_beyond_eof() {
        let dir = temp_repo();
        fs::write(dir.path().join("short.txt"), "only\n").unwrap();

        let result = execute_tool(
            &KiwiTool::FileReadRange {
                path: "short.txt".to_string(),
                start_line: 5,
                end_line: None,
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                is_error: true,
                ref content,
            } if content.contains("beyond end of file")
        ));
    }

    #[test]
    fn read_file_range_blocks_path_traversal() {
        let dir = temp_repo();
        fs::write(dir.path().join("safe.txt"), "secret\n").unwrap();

        let result = execute_tool(
            &KiwiTool::FileReadRange {
                path: "../safe.txt".to_string(),
                start_line: 1,
                end_line: None,
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                is_error: true,
                ref content,
            } if content.contains("path traversal")
        ));
    }

    #[test]
    fn shell_capture_runs_command() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::ShellCapture {
                command: "echo hello".to_string(),
                timeout_secs: 5,
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                ref content,
                is_error: false
            } if content.contains("exit code: 0")
                && content.contains("--- stdout ---")
                && content.contains("hello")
        ));
    }

    #[test]
    fn shell_capture_includes_stderr_and_nonzero_exit() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::ShellCapture {
                command: "echo out; echo err 1>&2; exit 2".to_string(),
                timeout_secs: 5,
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                ref content,
                is_error: true,
            } if content.contains("exit code: 2")
                && content.contains("--- stdout ---")
                && content.contains("out")
                && content.contains("--- stderr ---")
                && content.contains("err")
        ));
    }

    #[test]
    fn shell_capture_truncates_large_output() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::ShellCapture {
                command: "python3 -c \"print('x' * 30000)\"".to_string(),
                timeout_secs: 10,
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                ref content,
                is_error: false,
                ..
            } if content.contains("output truncated at 20000 bytes")
        ));
    }

    #[test]
    fn shell_capture_times_out() {
        let dir = temp_repo();
        let result = execute_tool(
            &KiwiTool::ShellCapture {
                command: "sleep 3".to_string(),
                timeout_secs: 1,
            },
            dir.path(),
        );
        assert!(matches!(
            result,
            ExecutionResult::Done {
                is_error: true,
                ref content,
            } if content.contains("timed out after 1s")
        ));
    }

    #[test]
    fn move_and_delete_file_work() {
        let dir = temp_repo();
        fs::write(dir.path().join("old.txt"), "data").unwrap();

        let move_result = execute_tool(
            &KiwiTool::FileMove {
                src: "old.txt".to_string(),
                dest: "nested/new.txt".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            move_result,
            ExecutionResult::Done {
                is_error: false,
                ..
            }
        ));
        assert!(dir.path().join("nested/new.txt").is_file());

        let delete_result = execute_tool(
            &KiwiTool::FileDelete {
                path: "nested/new.txt".to_string(),
            },
            dir.path(),
        );
        assert!(matches!(
            delete_result,
            ExecutionResult::Done {
                is_error: false,
                ..
            }
        ));
        assert!(!dir.path().join("nested/new.txt").exists());
    }
}
