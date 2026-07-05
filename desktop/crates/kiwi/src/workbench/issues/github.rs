//! GitHub REST API helpers and git remote parsing.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use nest_error::{NestError, NestResult};
use nest_http_client::{HttpClientService, HttpRequest};
use serde::Deserialize;
use serde::Serialize;

use super::{IssueListItem, IssuesListEvent};
use crate::workbench::editor_files::FileLoadEvent;

pub(crate) const GITHUB_API: &str = "https://api.github.com";

/// Result of creating a GitHub issue from the editor compose tab.
#[derive(Debug)]
pub enum IssueCreateEvent {
    /// Issue was created on GitHub.
    Created {
        /// Editor tab that submitted the form.
        tab_index: usize,
        /// Repository owner.
        owner: String,
        /// Repository name.
        repo: String,
        /// Assigned issue number.
        number: u64,
        /// Link to the issue on GitHub.
        html_url: String,
        /// Formatted issue body for the editor tab.
        content: String,
    },
    /// Failed to create the issue.
    Failed {
        /// Editor tab that submitted the form.
        tab_index: usize,
        /// Error message.
        error: String,
    },
}

#[derive(Debug, Serialize)]
struct CreateIssueBody<'a> {
    title: &'a str,
    body: &'a str,
}

#[derive(Debug, Serialize)]
struct CreateCommentBody<'a> {
    body: &'a str,
}

/// Result of posting a GitHub issue comment.
#[derive(Debug)]
pub enum IssueCommentCreateEvent {
    /// Comment was posted.
    Created {
        /// Issue the comment was posted on.
        issue_number: u64,
        /// Link to the comment on GitHub.
        html_url: String,
    },
    /// Failed to post the comment.
    Failed {
        /// Error message.
        error: String,
    },
}

/// Result of loading an issue for the agent prompt.
#[derive(Debug)]
pub enum IssueSendToAgentEvent {
    /// Issue body loaded for the agent.
    Ready {
        /// Issue number.
        number: u64,
        /// Issue title.
        title: String,
        /// Formatted issue body for the prompt.
        content: String,
    },
    /// Failed to load the issue.
    Failed {
        /// Error message.
        error: String,
    },
}

/// Reads a token from `gh auth token` when the GitHub CLI is installed and signed in.
pub fn gh_cli_token() -> Option<String> {
    run_gh(&["auth", "token"])
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|token| !token.is_empty())
}

/// Reads the signed-in GitHub username from the GitHub CLI.
pub fn gh_cli_login() -> Option<String> {
    run_gh(&["api", "user", "-q", ".login"])
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|login| !login.is_empty())
}

/// Returns true when `gh auth token` succeeds.
pub fn gh_auth_available() -> bool {
    gh_cli_token().is_some()
}

fn run_gh(args: &[&str]) -> Option<std::process::Output> {
    Command::new("gh")
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
}

/// Resolves a GitHub token from the GitHub CLI, environment, or saved config.
pub fn resolve_github_token(stored: Option<&str>) -> Option<String> {
    gh_cli_token()
        .or_else(github_token_from_env)
        .or_else(|| {
            stored
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

/// Reads a GitHub token from the environment (`GITHUB_TOKEN` or `GH_TOKEN`).
pub fn github_token_from_env() -> Option<String> {
    std::env::var("GITHUB_TOKEN")
        .ok()
        .or_else(|| std::env::var("GH_TOKEN").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Parses `owner` and `repo` from a GitHub remote URL.
pub fn parse_github_remote(url: &str) -> Option<(String, String)> {
    let url = url.trim();
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        return split_owner_repo(rest.trim_end_matches(".git"));
    }
    if let Some(rest) = url.strip_prefix("ssh://git@github.com/") {
        return split_owner_repo(rest.trim_end_matches(".git"));
    }
    if let Some(idx) = url.find("github.com") {
        let path = url[idx + "github.com".len()..]
            .trim_start_matches('/')
            .trim_start_matches(':');
        return split_owner_repo(path.trim_end_matches(".git"));
    }
    None
}

fn split_owner_repo(path: &str) -> Option<(String, String)> {
    let (owner, repo) = path.split_once('/')?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// Reads `origin` and parses it as a GitHub repository.
pub fn read_github_repo(root: &Path) -> NestResult<(String, String)> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|error| NestError::io(format!("git remote failed: {error}")))?;

    if !output.status.success() {
        return Err(NestError::validation(
            "no GitHub origin remote configured (git remote get-url origin)",
        ));
    }

    let url = String::from_utf8_lossy(&output.stdout);
    parse_github_remote(&url).ok_or_else(|| {
        NestError::validation(format!("origin is not a GitHub remote: {}", url.trim()))
    })
}

/// Result of verifying a GitHub token against the `/user` endpoint.
#[derive(Debug)]
pub enum GitHubVerifyEvent {
    /// Token is valid for the given GitHub username.
    Verified {
        login: String,
    },
    /// Token verification failed.
    Failed {
        message: String,
    },
}

/// Verifies a token on a background thread.
pub fn spawn_verify_token(
    http: HttpClientService,
    token: String,
) -> mpsc::Receiver<GitHubVerifyEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let event = runtime.block_on(async {
            match verify_token(&http, &token).await {
                Ok(login) => GitHubVerifyEvent::Verified { login },
                Err(error) => GitHubVerifyEvent::Failed {
                    message: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    });
    rx
}

async fn verify_token(http: &HttpClientService, token: &str) -> NestResult<String> {
    let url = format!("{GITHUB_API}/user");
    let user: GithubUser = github_get_json(http, &url, Some(token)).await?;
    Ok(user.login)
}

/// Loads open issues on a background thread.
pub fn spawn_issue_list(
    http: HttpClientService,
    root: PathBuf,
    token: Option<String>,
) -> mpsc::Receiver<IssuesListEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let event = runtime.block_on(async {
            match load_issue_list(&http, &root, token.as_deref()).await {
                Ok((repo, issues)) => IssuesListEvent::Ready { repo, issues },
                Err(error) => IssuesListEvent::Failed(error.to_string()),
            }
        });
        let _ = tx.send(event);
    });
    rx
}

/// Loads one issue body on a background thread.
pub fn spawn_issue_load(
    http: HttpClientService,
    owner: String,
    repo: String,
    number: u64,
    tab_index: usize,
    token: Option<String>,
) -> mpsc::Receiver<FileLoadEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let event = runtime.block_on(async {
            match fetch_issue(&http, &owner, &repo, number, token.as_deref()).await {
                Ok(content) => FileLoadEvent::Loaded { tab_index, content },
                Err(error) => FileLoadEvent::Failed {
                    tab_index,
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    });
    rx
}

/// Loads a GitHub issue body for the agent prompt on a background thread.
pub fn spawn_issue_for_agent(
    http: HttpClientService,
    owner: String,
    repo: String,
    number: u64,
    token: Option<String>,
) -> mpsc::Receiver<IssueSendToAgentEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let event = runtime.block_on(async {
            let url = format!("{GITHUB_API}/repos/{owner}/{repo}/issues/{number}");
            match github_get_json::<GithubIssueDetail>(
                &http,
                &url,
                token.as_deref(),
            )
            .await
            {
                Ok(issue) => IssueSendToAgentEvent::Ready {
                    number: issue.number,
                    title: issue.title.clone(),
                    content: format_issue(&issue),
                },
                Err(error) => IssueSendToAgentEvent::Failed {
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    });
    rx
}

/// Creates a GitHub issue on a background thread.
pub fn spawn_create_issue(
    http: HttpClientService,
    owner: String,
    repo: String,
    title: String,
    body: String,
    tab_index: usize,
    token: Option<String>,
) -> mpsc::Receiver<IssueCreateEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let event = runtime.block_on(async {
            match create_issue(
                &http,
                &owner,
                &repo,
                &title,
                &body,
                token.as_deref(),
            )
            .await
            {
                Ok(issue) => IssueCreateEvent::Created {
                    tab_index,
                    owner,
                    repo,
                    number: issue.number,
                    html_url: issue.html_url.clone(),
                    content: format_issue(&issue),
                },
                Err(error) => IssueCreateEvent::Failed {
                    tab_index,
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    });
    rx
}

/// Posts a GitHub issue comment on a background thread.
pub fn spawn_create_comment(
    http: HttpClientService,
    owner: String,
    repo: String,
    issue_number: u64,
    body: String,
    token: Option<String>,
) -> mpsc::Receiver<IssueCommentCreateEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let event = runtime.block_on(async {
            match create_issue_comment(
                &http,
                &owner,
                &repo,
                issue_number,
                &body,
                token.as_deref(),
            )
            .await
            {
                Ok(comment) => IssueCommentCreateEvent::Created {
                    issue_number,
                    html_url: comment.html_url,
                },
                Err(error) => IssueCommentCreateEvent::Failed {
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    });
    rx
}

async fn create_issue(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    title: &str,
    body: &str,
    token: Option<&str>,
) -> NestResult<GithubIssueDetail> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/issues");
    let payload = CreateIssueBody { title, body };
    github_post_json(http, &url, &payload, token).await
}

async fn create_issue_comment(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    issue_number: u64,
    body: &str,
    token: Option<&str>,
) -> NestResult<GithubCommentDetail> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/issues/{issue_number}/comments");
    let payload = CreateCommentBody { body };
    github_post_json(http, &url, &payload, token).await
}

async fn load_issue_list(
    http: &HttpClientService,
    root: &Path,
    token: Option<&str>,
) -> NestResult<((String, String), Vec<IssueListItem>)> {
    let repo = read_github_repo(root)?;
    let issues = fetch_issues(http, &repo.0, &repo.1, token).await?;
    Ok((repo, issues))
}

async fn fetch_issues(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    token: Option<&str>,
) -> NestResult<Vec<IssueListItem>> {
    let url = format!(
        "{GITHUB_API}/repos/{owner}/{repo}/issues?state=open&per_page=50&sort=updated"
    );
    let raw: Vec<GithubIssueSummary> = github_get_json(http, &url, token).await?;
    Ok(raw
        .into_iter()
        .filter(|issue| issue.pull_request.is_none())
        .map(|issue| IssueListItem {
            number: issue.number,
            title: issue.title,
            state: issue.state,
            author: issue.user.login,
            html_url: issue.html_url,
        })
        .collect())
}

async fn fetch_issue(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    number: u64,
    token: Option<&str>,
) -> NestResult<String> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/issues/{number}");
    let issue: GithubIssueDetail = github_get_json(http, &url, token).await?;
    Ok(format_issue(&issue))
}

pub fn format_issue(issue: &GithubIssueDetail) -> String {
    let labels = issue
        .labels
        .iter()
        .map(|label| label.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let labels_line = if labels.is_empty() {
        String::new()
    } else {
        format!("Labels: {labels}\n")
    };
    let body = issue.body.as_deref().unwrap_or("_No description provided._");

    format!(
        "# {} (#{})\n\n\
         **State:** {}  \n\
         **Author:** @{}  \n\
         **Created:** {}  \n\
         **Updated:** {}  \n\
         {labels_line}\
         **URL:** {}\n\n\
         ---\n\n\
         {body}\n",
        issue.title,
        issue.number,
        issue.state,
        issue.user.login,
        issue.created_at,
        issue.updated_at,
        issue.html_url,
    )
}

pub(crate) fn map_github_api_error(status_code: u16, body: &[u8]) -> NestError {
    if status_code == 401 {
        return NestError::validation(
            "GitHub authentication required. Run `gh auth login` in a terminal, then retry.",
        );
    }
    let message = String::from_utf8_lossy(body);
    NestError::validation(format!(
        "GitHub API error {status_code}: {}",
        message.trim()
    ))
}

async fn github_get_json<T>(
    http: &HttpClientService,
    url: &str,
    token: Option<&str>,
) -> NestResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut request = HttpRequest::get(url)
        .with_header("Accept", "application/vnd.github+json")
        .with_header("X-GitHub-Api-Version", "2022-11-28");
    if let Some(token) = token.filter(|value| !value.is_empty()) {
        request = request.with_header("Authorization", format!("Bearer {token}"));
    }

    let response = http.send(request).await?;
    if !response.status.is_success() {
        return Err(map_github_api_error(response.status.code(), &response.body));
    }
    serde_json::from_slice(&response.body).map_err(|error| {
        NestError::validation(format!("failed to decode GitHub JSON: {error}"))
    })
}

async fn github_post_json<T, B>(
    http: &HttpClientService,
    url: &str,
    body: &B,
    token: Option<&str>,
) -> NestResult<T>
where
    T: for<'de> Deserialize<'de>,
    B: Serialize + ?Sized,
{
    let json = serde_json::to_vec(body).map_err(|error| {
        NestError::validation(format!("failed to encode GitHub JSON: {error}"))
    })?;
    let mut request = HttpRequest::post(url)
        .with_header("Accept", "application/vnd.github+json")
        .with_header("X-GitHub-Api-Version", "2022-11-28")
        .with_header("Content-Type", "application/json")
        .with_body(json);
    if let Some(token) = token.filter(|value| !value.is_empty()) {
        request = request.with_header("Authorization", format!("Bearer {token}"));
    }

    let response = http.send(request).await?;
    if !response.status.is_success() {
        return Err(map_github_api_error(response.status.code(), &response.body));
    }
    serde_json::from_slice(&response.body).map_err(|error| {
        NestError::validation(format!("failed to decode GitHub JSON: {error}"))
    })
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GithubLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GithubCommentDetail {
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubIssueSummary {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    user: GithubUser,
    pull_request: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct GithubIssueDetail {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    user: GithubUser,
    body: Option<String>,
    created_at: String,
    updated_at: String,
    labels: Vec<GithubLabel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ssh_github_remote() {
        assert_eq!(
            parse_github_remote("git@github.com:octocat/Hello-World.git"),
            Some(("octocat".into(), "Hello-World".into()))
        );
    }

    #[test]
    fn parses_https_github_remote() {
        assert_eq!(
            parse_github_remote("https://github.com/octocat/Hello-World.git"),
            Some(("octocat".into(), "Hello-World".into()))
        );
    }

    #[test]
    fn formats_issue_markdown() {
        let issue = GithubIssueDetail {
            number: 42,
            title: "Bug report".into(),
            state: "open".into(),
            html_url: "https://github.com/o/r/issues/42".into(),
            user: GithubUser {
                login: "octocat".into(),
            },
            body: Some("Steps to reproduce…".into()),
            created_at: "2026-07-01T12:00:00Z".into(),
            updated_at: "2026-07-02T12:00:00Z".into(),
            labels: vec![GithubLabel {
                name: "bug".into(),
            }],
        };
        let text = format_issue(&issue);
        assert!(text.contains("# Bug report (#42)"));
        assert!(text.contains("Steps to reproduce"));
    }
}
