use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use super::cancel::SearchCancelHandle;
use super::{SearchResult, MAX_SEARCH_RESULTS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentSearchError {
    NotFound,
    Failed(String),
}

pub fn search_content(
    repo_root: &Path,
    command: &str,
    query: &str,
    cancel: &SearchCancelHandle,
) -> Result<(Vec<SearchResult>, bool), ContentSearchError> {
    if query.is_empty() {
        return Ok((Vec::new(), false));
    }

    match run_ripgrep(repo_root, command, query, cancel) {
        Ok(results) => Ok(results),
        Err(ContentSearchError::NotFound) => run_grep(repo_root, query, cancel),
        Err(err) => Err(err),
    }
}

fn run_ripgrep(
    repo_root: &Path,
    command: &str,
    query: &str,
    cancel: &SearchCancelHandle,
) -> Result<(Vec<SearchResult>, bool), ContentSearchError> {
    let mut json_command = Command::new(command);
    json_command
        .args(["--json", "-F", "--", query, "."])
        .current_dir(repo_root);

    let output = run_tracked(cancel, json_command).map_err(map_spawn_error)?;

    if should_fallback_to_line_mode(&output) {
        return run_ripgrep_line(repo_root, command, query, cancel);
    }

    if output.status.code() == Some(2) {
        return Err(ripgrep_failure_message(&output.stderr));
    }

    let (results, truncated) = collect_json_results(&output.stdout, repo_root);
    Ok((results, truncated))
}

fn run_ripgrep_line(
    repo_root: &Path,
    command: &str,
    query: &str,
    cancel: &SearchCancelHandle,
) -> Result<(Vec<SearchResult>, bool), ContentSearchError> {
    let mut line_command = Command::new(command);
    line_command
        .args(["--line-number", "--no-heading", "-F", "--", query, "."])
        .current_dir(repo_root);

    let output = run_tracked(cancel, line_command).map_err(map_spawn_error)?;

    if output.status.code() == Some(2) {
        return Err(ripgrep_failure_message(&output.stderr));
    }

    let (results, truncated) = collect_line_results(&output.stdout, repo_root, parse_ripgrep_line);
    Ok((results, truncated))
}

fn run_grep(
    repo_root: &Path,
    query: &str,
    cancel: &SearchCancelHandle,
) -> Result<(Vec<SearchResult>, bool), ContentSearchError> {
    let mut command = Command::new("grep");
    command
        .args(["-r", "-n", "-H", "-F", "--", query, "."])
        .current_dir(repo_root);

    let output = run_tracked(cancel, command).map_err(map_spawn_error)?;

    if output.status.code() == Some(2) {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ContentSearchError::Failed(if message.is_empty() {
            "grep failed".to_string()
        } else {
            message
        }));
    }

    let (results, truncated) = collect_line_results(&output.stdout, repo_root, parse_grep_line);
    Ok((results, truncated))
}

fn run_tracked(
    cancel: &SearchCancelHandle,
    command: Command,
) -> Result<std::process::Output, std::io::Error> {
    cancel.run_command(command)
}

fn map_spawn_error(err: std::io::Error) -> ContentSearchError {
    if err.kind() == std::io::ErrorKind::NotFound {
        ContentSearchError::NotFound
    } else {
        ContentSearchError::Failed(err.to_string())
    }
}

fn should_fallback_to_line_mode(output: &std::process::Output) -> bool {
    if output.status.code() != Some(2) {
        return false;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    stderr.contains("--json")
        && (stderr.contains("unknown")
            || stderr.contains("unrecognized")
            || stderr.contains("invalid"))
}

fn ripgrep_failure_message(stderr: &[u8]) -> ContentSearchError {
    let message = String::from_utf8_lossy(stderr).trim().to_string();
    ContentSearchError::Failed(if message.is_empty() {
        "ripgrep failed".to_string()
    } else {
        message
    })
}

fn collect_json_results(stdout: &[u8], repo_root: &Path) -> (Vec<SearchResult>, bool) {
    let mut results = Vec::new();
    let mut truncated = false;
    let stdout = String::from_utf8_lossy(stdout);

    for line in stdout.lines() {
        if results.len() >= MAX_SEARCH_RESULTS {
            truncated = true;
            break;
        }

        if let Some(result) = parse_ripgrep_json_line(line, repo_root) {
            results.push(result);
        }
    }

    (results, truncated)
}

fn collect_line_results(
    stdout: &[u8],
    repo_root: &Path,
    parse_line: fn(&str, &Path) -> Option<SearchResult>,
) -> (Vec<SearchResult>, bool) {
    let mut results = Vec::new();
    let mut truncated = false;
    let stdout = String::from_utf8_lossy(stdout);

    for line in stdout.lines() {
        if results.len() >= MAX_SEARCH_RESULTS {
            truncated = true;
            break;
        }

        if let Some(result) = parse_line(line, repo_root) {
            results.push(result);
        }
    }

    (results, truncated)
}

#[derive(Debug, Deserialize)]
struct RgMessage {
    #[serde(rename = "type")]
    message_type: String,
    data: Option<RgMatchData>,
}

#[derive(Debug, Deserialize)]
struct RgMatchData {
    path: RgPath,
    lines: RgLines,
    line_number: u32,
}

#[derive(Debug, Deserialize)]
struct RgPath {
    text: String,
}

#[derive(Debug, Deserialize)]
struct RgLines {
    text: String,
}

fn parse_ripgrep_json_line(line: &str, repo_root: &Path) -> Option<SearchResult> {
    let message: RgMessage = serde_json::from_str(line).ok()?;
    if message.message_type != "match" {
        return None;
    }

    let data = message.data?;
    let preview = data.lines.text.trim_end_matches('\n').to_string();
    let path = normalize_result_path(repo_root, &data.path.text);
    Some(SearchResult::content(path, data.line_number, preview))
}

fn parse_ripgrep_line(line: &str, repo_root: &Path) -> Option<SearchResult> {
    let (path_part, rest) = line.split_once(':')?;
    let (line_number, preview) = rest.split_once(':')?;
    let line_number = line_number.parse().ok()?;
    let path = normalize_result_path(repo_root, path_part);
    Some(SearchResult::content(
        path,
        line_number,
        preview.to_string(),
    ))
}

fn parse_grep_line(line: &str, repo_root: &Path) -> Option<SearchResult> {
    parse_ripgrep_line(line, repo_root)
}

fn normalize_result_path(repo_root: &Path, path_text: &str) -> PathBuf {
    let trimmed = path_text
        .strip_prefix("./")
        .or_else(|| path_text.strip_prefix(".\\"))
        .unwrap_or(path_text);
    repo_root.join(trimmed)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    use super::*;

    fn temp_repo(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("kiwi-search-content-{name}-{}", std::process::id()))
    }

    #[test]
    fn parse_ripgrep_json_line_builds_content_result() {
        let root = Path::new("/tmp/kiwi");
        let line = r#"{"type":"match","data":{"path":{"text":"./src/main.rs"},"lines":{"text":"fn main() {}\n"},"line_number":10,"absolute_offset":0,"submatches":[]}}"#;
        let result = parse_ripgrep_json_line(line, root).expect("parse");
        assert_eq!(result.line, Some(10));
        assert_eq!(result.preview, "fn main() {}");
        assert_eq!(result.path, root.join("src/main.rs"));
    }

    #[test]
    fn parse_ripgrep_line_builds_content_result() {
        let root = Path::new("/tmp/kiwi");
        let result = parse_ripgrep_line("src/main.rs:10:fn main() {}", root).expect("parse");
        assert_eq!(result.line, Some(10));
        assert_eq!(result.preview, "fn main() {}");
        assert_eq!(result.path, root.join("src/main.rs"));
    }

    #[test]
    fn search_content_reports_not_found_when_no_search_tools_exist() {
        if Command::new("grep").arg("--version").output().is_ok() {
            return;
        }

        let root = std::env::temp_dir();
        let cancel = SearchCancelHandle::default();
        let err = search_content(&root, "kiwi-missing-rg-binary", "needle", &cancel).unwrap_err();
        assert_eq!(err, ContentSearchError::NotFound);
    }

    #[test]
    fn search_content_returns_empty_for_no_matches() {
        if Command::new("rg").arg("--version").output().is_err() {
            return;
        }

        let root = temp_repo("empty");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("mkdir");
        fs::write(root.join("hello.txt"), "hello\n").expect("write");

        let cancel = SearchCancelHandle::default();
        let (results, truncated) =
            search_content(&root, "rg", "missing-needle-xyz", &cancel).expect("search");
        assert!(results.is_empty());
        assert!(!truncated);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn search_content_finds_string_with_ripgrep() {
        if Command::new("rg").arg("--version").output().is_err() {
            return;
        }

        let root = temp_repo("hit");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).expect("mkdir");
        fs::write(
            root.join("src/main.rs"),
            "fn main() { println!(\"kiwi\"); }\n",
        )
        .expect("write");

        let cancel = SearchCancelHandle::default();
        let (results, truncated) = search_content(&root, "rg", "println", &cancel).expect("search");
        assert!(!truncated);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, Some(1));
        assert!(results[0].preview.contains("println"));
        assert!(results[0].path.ends_with("src/main.rs"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn search_content_uses_grep_when_rg_missing() {
        if Command::new("grep").arg("--version").output().is_err() {
            return;
        }

        let root = temp_repo("grep");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("mkdir");
        fs::write(root.join("notes.txt"), "find the kiwi fruit\n").expect("write");

        let cancel = SearchCancelHandle::default();
        let (results, truncated) = search_content(
            &root,
            "kiwi-missing-rg-binary-for-grep-fallback",
            "kiwi fruit",
            &cancel,
        )
        .expect("search");
        assert!(!truncated);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, Some(1));
        assert!(results[0].preview.contains("kiwi fruit"));

        let _ = fs::remove_dir_all(root);
    }
}
