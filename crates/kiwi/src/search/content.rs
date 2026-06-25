use std::path::Path;
use std::process::Command;

use super::cancel::SearchCancelHandle;
use super::types::{SearchResult, MAX_SEARCH_RESULTS};

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

    let mut command = Command::new(command);
    command
        .args(["--line-number", "--no-heading", query, "."])
        .current_dir(repo_root);

    let output = cancel.run_command(command).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            ContentSearchError::NotFound
        } else {
            ContentSearchError::Failed(err.to_string())
        }
    })?;

    if output.status.code() == Some(2) {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ContentSearchError::Failed(if message.is_empty() {
            "ripgrep failed".to_string()
        } else {
            message
        }));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();
    let mut truncated = false;

    for line in stdout.lines() {
        if results.len() >= MAX_SEARCH_RESULTS {
            truncated = true;
            break;
        }

        if let Some(result) = parse_ripgrep_line(line, repo_root) {
            results.push(result);
        }
    }

    Ok((results, truncated))
}

fn parse_ripgrep_line(line: &str, repo_root: &Path) -> Option<SearchResult> {
    let (path_part, rest) = line.split_once(':')?;
    let (line_number, preview) = rest.split_once(':')?;
    let line_number = line_number.parse().ok()?;
    let path = repo_root.join(path_part);
    Some(SearchResult::content(
        path,
        line_number,
        preview.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ripgrep_line_builds_content_result() {
        let root = Path::new("/tmp/kiwi");
        let result = parse_ripgrep_line("src/main.rs:10:fn main() {}", root).expect("parse");
        assert_eq!(result.line, Some(10));
        assert_eq!(result.preview, "fn main() {}");
        assert_eq!(result.path, root.join("src/main.rs"));
    }

    #[test]
    fn search_content_reports_not_found_for_missing_rg() {
        let root = std::env::temp_dir();
        let cancel = SearchCancelHandle::default();
        let err = search_content(&root, "kiwi-missing-rg-binary", "needle", &cancel).unwrap_err();
        assert_eq!(err, ContentSearchError::NotFound);
    }
}
