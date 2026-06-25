use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::issue::command_on_path;
use super::pr_detail::PrState;

pub const PR_LIST_CACHE_SECS: u64 = 60;
pub const PR_LIST_LIMIT: &str = "100";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequest {
    pub number: u32,
    pub title: String,
    pub state: PrState,
    pub author: String,
    pub is_draft: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrListLoadResult {
    pub prs: Vec<PullRequest>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhPullRequest {
    number: u32,
    title: String,
    state: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    author: GhUser,
}

#[derive(Debug, Deserialize)]
struct GhUser {
    login: String,
}

pub fn load_pr_list(repo_root: &Path, command: &str) -> PrListLoadResult {
    if !command_on_path(command) {
        return PrListLoadResult {
            prs: Vec::new(),
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
        };
    }

    let output = Command::new(command)
        .args([
            "pr",
            "list",
            "--json",
            "number,title,state,isDraft,author",
            "--limit",
            PR_LIST_LIMIT,
        ])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => match parse_pr_list_json(&result.stdout) {
            Ok(prs) => PrListLoadResult { prs, error: None },
            Err(message) => PrListLoadResult {
                prs: Vec::new(),
                error: Some(message),
            },
        },
        Ok(result) => PrListLoadResult {
            prs: Vec::new(),
            error: Some(format_pr_list_failure(&result.stderr, &result.stdout)),
        },
        Err(err) => PrListLoadResult {
            prs: Vec::new(),
            error: Some(format!("Failed to run `{command} pr list`: {err}")),
        },
    }
}

fn parse_pr_list_json(bytes: &[u8]) -> Result<Vec<PullRequest>, String> {
    let raw: Vec<GhPullRequest> =
        serde_json::from_slice(bytes).map_err(|err| format!("Invalid gh pr JSON: {err}"))?;

    Ok(raw
        .into_iter()
        .map(|pr| PullRequest {
            number: pr.number,
            title: pr.title,
            state: PrState::parse(&pr.state, pr.is_draft),
            author: pr.author.login,
            is_draft: pr.is_draft,
        })
        .collect())
}

fn format_pr_list_failure(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }

    "gh pr list failed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pr_list_json_maps_fields() {
        let json = r#"[
            {
                "number": 60,
                "title": "PR list via gh json",
                "state": "OPEN",
                "isDraft": false,
                "author": {"login": "octocat"}
            },
            {
                "number": 12,
                "title": "Draft work",
                "state": "OPEN",
                "isDraft": true,
                "author": {"login": "hubot"}
            },
            {
                "number": 3,
                "title": "Shipped",
                "state": "MERGED",
                "isDraft": false,
                "author": {"login": "octocat"}
            }
        ]"#;

        let prs = parse_pr_list_json(json.as_bytes()).expect("parse");
        assert_eq!(prs.len(), 3);
        assert_eq!(prs[0].number, 60);
        assert_eq!(prs[0].title, "PR list via gh json");
        assert_eq!(prs[0].state, PrState::Open);
        assert_eq!(prs[0].author, "octocat");
        assert!(!prs[0].is_draft);
        assert_eq!(prs[1].state, PrState::Draft);
        assert_eq!(prs[2].state, PrState::Merged);
    }

    #[test]
    fn parse_pr_list_json_rejects_invalid_payload() {
        let err = parse_pr_list_json(b"{not json}").expect_err("invalid");
        assert!(err.contains("Invalid gh pr JSON"));
    }
}
