//! GitHub integration for the Issues activity panel.
//!
//! Uses the `gh` CLI (same auth as `gh auth login`) and resolves the repository
//! from the workspace `origin` remote.

use std::path::Path;
use std::process::Command;

use nest_error::{NestError, NestResult};
use serde::Deserialize;
use serde::Serialize;

/// GitHub auth snapshot for the Issues panel header.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubAuthStatus {
    pub authenticated: bool,
    pub login: Option<String>,
}

/// Resolved `owner/repo` for the workspace.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepoInfo {
    pub repo: String,
    pub html_url: String,
}

/// GitHub user reference (issue author).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubUser {
    pub login: String,
}

/// One row in the Issues sidebar list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubIssueListItem {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub updated_at: String,
    pub created_at: String,
    pub author: Option<GitHubUser>,
    pub comments: u32,
}

/// Repository label (sidebar chips + manage modal).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubLabel {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

/// Repository milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubMilestone {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub description: Option<String>,
    pub due_on: Option<String>,
}

/// Milestone as embedded on an issue. `gh issue view --json milestone` omits
/// some fields the REST milestone list has, so all but `number`/`title` are optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubIssueMilestone {
    pub number: u64,
    pub title: String,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub due_on: Option<String>,
}

/// A related issue in a dependency relationship (blocked by / blocking).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubIssueDependency {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
}

/// Full issue body for editor tabs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubIssue {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub author: Option<GitHubUser>,
    pub comments: u32,
    pub milestone: Option<GitHubIssueMilestone>,
    /// Issues that block this one.
    pub blocked_by: Vec<GitHubIssueDependency>,
    /// Issues that this one blocks.
    pub blocking: Vec<GitHubIssueDependency>,
}

/// Result of creating an issue or posting a comment.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubIssueActionResult {
    pub number: u64,
    pub html_url: String,
}

/// Reads whether `gh` is signed in.
pub fn auth_status() -> GitHubAuthStatus {
    let output = match Command::new("gh").args(["auth", "status", "-h", "github.com"]).output() {
        Ok(value) => value,
        Err(_) => {
            return GitHubAuthStatus {
                authenticated: false,
                login: None,
            };
        }
    };
    if !output.status.success() {
        return GitHubAuthStatus {
            authenticated: false,
            login: None,
        };
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let login = text
        .lines()
        .find_map(|line| {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("account ") {
                return rest.split_whitespace().next().map(str::to_string);
            }
            None
        })
        .or_else(|| {
            text.lines().find_map(|line| {
                let line = line.trim();
                line.strip_prefix("Logged in to github.com as ")
                    .and_then(|rest| rest.split_whitespace().next())
                    .map(str::to_string)
            })
        });
    GitHubAuthStatus {
        authenticated: true,
        login,
    }
}

/// Resolves `owner/repo` from `git remote get-url origin` under `root`.
pub fn read_repo(root: &Path) -> NestResult<Option<GitHubRepoInfo>> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|error| NestError::io(format!("git remote failed: {error}")))?;
    if !output.status.success() {
        return Ok(None);
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let repo = parse_github_remote(&url)?;
    let Some(repo) = repo else {
        return Ok(None);
    };
    Ok(Some(GitHubRepoInfo {
        html_url: format!("https://github.com/{repo}"),
        repo,
    }))
}

/// Lists issues for `repo` (`state`: `open`, `closed`, or `all`).
pub fn issue_list(repo: &str, state: &str, limit: u32) -> NestResult<Vec<GitHubIssueListItem>> {
    let state = match state {
        "open" | "closed" | "all" => state,
        _ => "open",
    };
    // REST API returns `comments` as a count; `gh issue list --json comments` is an array.
    let json = gh_json(&[
        "api",
        &format!("repos/{repo}/issues?state={state}&per_page={limit}"),
    ])?;
    let raw: Vec<RawRestIssueListItem> = serde_json::from_str(&json)
        .map_err(|error| NestError::io(format!("failed to parse gh issue list: {error}")))?;
    Ok(raw
        .into_iter()
        .filter(|item| item.pull_request.is_none())
        .map(|item| GitHubIssueListItem {
            number: item.number,
            title: item.title,
            state: item.state,
            labels: item.labels,
            updated_at: item.updated_at,
            created_at: item.created_at,
            author: item.user,
            comments: item.comments,
        })
        .collect())
}

/// Loads one issue (body + metadata) for an editor tab.
pub fn issue_view(repo: &str, number: u64) -> NestResult<GitHubIssue> {
    let json = gh_json(&[
        "issue",
        "view",
        &number.to_string(),
        "--repo",
        repo,
        "--json",
        "number,title,body,state,labels,url,createdAt,updatedAt,author,milestone,blockedBy,blocking",
    ])?;
    let raw: RawIssue = serde_json::from_str(&json)
        .map_err(|error| NestError::io(format!("failed to parse gh issue view: {error}")))?;
    let comments = issue_comment_count(repo, number)?;
    // `gh` gives us the dependency counts cheaply in the call above; only fetch
    // the full lists (an extra REST call each) when there's actually something.
    let blocked_by = if raw.blocked_by.as_ref().map_or(0, |c| c.total_count) > 0 {
        issue_dependencies(repo, number, "blocked_by")
    } else {
        Vec::new()
    };
    let blocking = if raw.blocking.as_ref().map_or(0, |c| c.total_count) > 0 {
        issue_dependencies(repo, number, "blocking")
    } else {
        Vec::new()
    };
    Ok(GitHubIssue {
        number: raw.number,
        title: raw.title,
        body: raw.body.unwrap_or_default(),
        state: raw.state,
        labels: raw.labels,
        html_url: raw.url,
        created_at: raw.created_at,
        updated_at: raw.updated_at,
        author: raw.author,
        comments,
        milestone: raw.milestone,
        blocked_by,
        blocking,
    })
}

/// Fetches issue dependencies in one direction (`blocked_by` or `blocking`).
/// Best-effort: returns an empty list on any error (e.g. GitHub Enterprise
/// without the dependencies feature) so it never blocks opening an issue.
fn issue_dependencies(repo: &str, number: u64, direction: &str) -> Vec<GitHubIssueDependency> {
    let json = match gh_json(&[
        "api",
        &format!("repos/{repo}/issues/{number}/dependencies/{direction}"),
    ]) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let raw: Vec<RawDependency> = serde_json::from_str(&json).unwrap_or_default();
    raw.into_iter()
        .map(|item| GitHubIssueDependency {
            number: item.number,
            title: item.title,
            state: item.state,
            html_url: item.html_url,
        })
        .collect()
}

/// Creates a new GitHub issue.
pub fn issue_create(repo: &str, title: &str, body: &str) -> NestResult<GitHubIssueActionResult> {
    if title.trim().is_empty() {
        return Err(NestError::validation("issue title is empty"));
    }
    let mut command = gh();
    command.args(["issue", "create", "--repo", repo, "--title", title]);
    if !body.trim().is_empty() {
        command.args(["--body", body]);
    }
    let output = command
        .output()
        .map_err(|error| NestError::io(format!("gh issue create failed: {error}")))?;
    if !output.status.success() {
        return Err(gh_error("issue create", &output.stderr, &output.stdout));
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let number = url
        .rsplit('/')
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    Ok(GitHubIssueActionResult { number, html_url: url })
}

/// Posts a comment on an issue.
pub fn issue_comment(repo: &str, number: u64, body: &str) -> NestResult<GitHubIssueActionResult> {
    if body.trim().is_empty() {
        return Err(NestError::validation("comment body is empty"));
    }
    let output = gh()
        .args([
            "issue",
            "comment",
            &number.to_string(),
            "--repo",
            repo,
            "--body",
            body,
        ])
        .output()
        .map_err(|error| NestError::io(format!("gh issue comment failed: {error}")))?;
    if !output.status.success() {
        return Err(gh_error("issue comment", &output.stderr, &output.stdout));
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(GitHubIssueActionResult {
        number,
        html_url: url,
    })
}

/// Lists repository labels.
pub fn label_list(repo: &str) -> NestResult<Vec<GitHubLabel>> {
    let json = gh_json(&[
        "label",
        "list",
        "--repo",
        repo,
        "--limit",
        "100",
        "--json",
        "name,color,description",
    ])?;
    serde_json::from_str(&json)
        .map_err(|error| NestError::io(format!("failed to parse gh label list: {error}")))
}

/// Lists open milestones.
pub fn milestone_list(repo: &str) -> NestResult<Vec<GitHubMilestone>> {
    let json = gh_json(&[
        "api",
        &format!("repos/{repo}/milestones?state=open&per_page=100"),
        "--jq",
        ".[] | {number, title, state, description, due_on}",
    ])?;
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }
    // `gh api --jq` emits one JSON object per line for arrays.
    let mut out = Vec::new();
    for line in json.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let item: GitHubMilestone = serde_json::from_str(line).map_err(|error| {
            NestError::io(format!("failed to parse gh milestone list: {error}"))
        })?;
        out.push(item);
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
struct RawRestIssueListItem {
    number: u64,
    title: String,
    state: String,
    labels: Vec<GitHubLabel>,
    #[serde(rename = "updated_at")]
    updated_at: String,
    #[serde(rename = "created_at")]
    created_at: String,
    user: Option<GitHubUser>,
    comments: u32,
    pull_request: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RawIssue {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    labels: Vec<GitHubLabel>,
    url: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    author: Option<GitHubUser>,
    #[serde(default)]
    milestone: Option<GitHubIssueMilestone>,
    #[serde(rename = "blockedBy", default)]
    blocked_by: Option<RawDependencyConnection>,
    #[serde(default)]
    blocking: Option<RawDependencyConnection>,
}

/// `gh`'s `{ nodes, totalCount }` shape for the `blockedBy` / `blocking`
/// connection fields — we only need the count to decide whether to fetch details.
#[derive(Debug, Deserialize)]
struct RawDependencyConnection {
    #[serde(rename = "totalCount")]
    total_count: u32,
}

/// One element of the REST `dependencies/{blocked_by,blocking}` response
/// (a standard issue object; only the fields we render are read).
#[derive(Debug, Deserialize)]
struct RawDependency {
    number: u64,
    title: String,
    state: String,
    html_url: String,
}

fn issue_comment_count(repo: &str, number: u64) -> NestResult<u32> {
    let json = gh_json(&[
        "api",
        &format!("repos/{repo}/issues/{number}"),
        "--jq",
        ".comments",
    ])?;
    json.trim()
        .parse::<u32>()
        .map_err(|error| NestError::io(format!("failed to parse issue comment count: {error}")))
}

fn gh() -> Command {
    Command::new("gh")
}

fn gh_json(args: &[&str]) -> NestResult<String> {
    let output = gh()
        .args(args)
        .output()
        .map_err(|error| NestError::io(format!("gh {} failed: {error}", args.first().copied().unwrap_or("?"))))?;
    if !output.status.success() {
        return Err(gh_error(
            args.first().copied().unwrap_or("gh"),
            &output.stderr,
            &output.stdout,
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn gh_error(action: &str, stderr: &[u8], stdout: &[u8]) -> NestError {
    let detail = {
        let stderr = String::from_utf8_lossy(stderr).trim().to_string();
        if !stderr.is_empty() {
            stderr
        } else {
            String::from_utf8_lossy(stdout).trim().to_string()
        }
    };
    if detail.contains("not logged in") || detail.contains("auth login") {
        return NestError::validation("GitHub CLI is not authenticated — run `gh auth login`");
    }
    NestError::io(format!("gh {action} failed: {detail}"))
}

/// Parses `owner/repo` from common GitHub remote URL shapes.
fn parse_github_remote(url: &str) -> NestResult<Option<String>> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let path = trimmed
        .strip_prefix("git@github.com:")
        .or_else(|| trimmed.strip_prefix("ssh://git@github.com/"))
        .or_else(|| trimmed.strip_prefix("https://github.com/"))
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .unwrap_or(trimmed);

    let path = path.trim_end_matches(".git");
    let path = path.trim_start_matches('/');
    let mut parts = path.split('/').filter(|part| !part.is_empty());
    let owner = parts.next();
    let name = parts.next();
    match (owner, name) {
        (Some(owner), Some(name)) if !owner.contains(':') => Ok(Some(format!("{owner}/{name}"))),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_https_remote() {
        assert_eq!(
            parse_github_remote("https://github.com/pacificnm/nest.git").unwrap(),
            Some("pacificnm/nest".into())
        );
    }

    #[test]
    fn parses_ssh_remote() {
        assert_eq!(
            parse_github_remote("git@github.com:pacificnm/nest.git").unwrap(),
            Some("pacificnm/nest".into())
        );
    }

    #[test]
    fn rejects_non_github_remote() {
        assert_eq!(parse_github_remote("git@gitlab.com:org/repo.git").unwrap(), None);
    }
}
