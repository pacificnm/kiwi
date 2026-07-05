//! GitHub pull request list, detail, and merge API.

use std::sync::mpsc;
use std::thread;

use nest_error::{NestError, NestResult};
use nest_http::HttpMethod;
use nest_http_client::{HttpClientService, HttpRequest};
use serde::{Deserialize, Serialize};

use super::github::{map_github_api_error, GITHUB_API};

/// One pull request row in the sidebar list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestListItem {
    /// PR number.
    pub number: u64,
    /// PR title.
    pub title: String,
    /// Author login.
    pub author: String,
    /// Link on GitHub.
    pub html_url: String,
    /// Head branch name.
    pub head_branch: String,
    /// Base branch to merge into.
    pub base_branch: String,
    /// Draft PR flag.
    pub draft: bool,
}

/// Loaded pull request metadata for merge and display.
#[derive(Debug, Clone)]
pub struct PullRequestDetail {
    /// PR number.
    pub number: u64,
    /// PR title.
    pub title: String,
    /// Head branch.
    pub head_branch: String,
    /// Base branch.
    pub base_branch: String,
    /// Whether GitHub reports the PR as mergeable.
    pub mergeable: Option<bool>,
    /// Merge state (`clean`, `blocked`, etc.).
    pub mergeable_state: String,
    /// Draft PR flag.
    pub draft: bool,
    /// Already merged on GitHub.
    pub merged: bool,
    /// Link on GitHub (reserved for future browser actions).
    pub html_url: String,
    /// Formatted markdown for the editor tab.
    pub content: String,
}

/// Background pull request operation result.
#[derive(Debug)]
pub enum PullRequestEvent {
    /// Open pull requests loaded.
    ListReady(Vec<PullRequestListItem>),
    /// One pull request loaded for the editor.
    Loaded {
        /// Tab index receiving content.
        tab_index: usize,
        /// Parsed PR detail.
        detail: PullRequestDetail,
    },
    /// Pull request merged into its base branch.
    Merged {
        /// Merged PR number.
        number: u64,
        /// Base branch name.
        base_branch: String,
    },
    /// Operation failed.
    Failed {
        /// Short UI context.
        context: &'static str,
        /// Error message.
        error: String,
    },
}

/// Lists open pull requests on a background thread.
pub fn spawn_pull_request_list(
    http: HttpClientService,
    owner: String,
    repo: String,
    token: Option<String>,
) -> mpsc::Receiver<PullRequestEvent> {
    spawn_pr(http, move |http| {
        match block_on(fetch_pull_requests(&http, &owner, &repo, token.as_deref())) {
            Ok(items) => PullRequestEvent::ListReady(items),
            Err(error) => PullRequestEvent::Failed {
                context: "list pull requests",
                error: error.to_string(),
            },
        }
    })
}

/// Loads one pull request into an editor tab on a background thread.
pub fn spawn_pull_request_load(
    http: HttpClientService,
    owner: String,
    repo: String,
    number: u64,
    tab_index: usize,
    token: Option<String>,
) -> mpsc::Receiver<PullRequestEvent> {
    spawn_pr(http, move |http| {
        match block_on(fetch_pull_request(&http, &owner, &repo, number, token.as_deref())) {
            Ok(detail) => PullRequestEvent::Loaded { tab_index, detail },
            Err(error) => PullRequestEvent::Failed {
                context: "load pull request",
                error: error.to_string(),
            },
        }
    })
}

/// Merges a pull request on a background thread.
pub fn spawn_pull_request_merge(
    http: HttpClientService,
    owner: String,
    repo: String,
    number: u64,
    merge_method: String,
    token: Option<String>,
) -> mpsc::Receiver<PullRequestEvent> {
    spawn_pr(http, move |http| {
        match block_on(merge_pull_request(
            &http,
            &owner,
            &repo,
            number,
            &merge_method,
            token.as_deref(),
        )) {
            Ok(base_branch) => PullRequestEvent::Merged {
                number,
                base_branch,
            },
            Err(error) => PullRequestEvent::Failed {
                context: "merge pull request",
                error: error.to_string(),
            },
        }
    })
}

fn spawn_pr(
    http: HttpClientService,
    action: impl FnOnce(HttpClientService) -> PullRequestEvent + Send + 'static,
) -> mpsc::Receiver<PullRequestEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(action(http));
    });
    rx
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(future)
}

async fn fetch_pull_requests(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    token: Option<&str>,
) -> NestResult<Vec<PullRequestListItem>> {
    let url = format!(
        "{GITHUB_API}/repos/{owner}/{repo}/pulls?state=open&sort=updated&per_page=50"
    );
    let raw: Vec<GithubPullSummary> = github_get_json(http, &url, token).await?;
    Ok(raw
        .into_iter()
        .map(|item| PullRequestListItem {
            number: item.number,
            title: item.title,
            author: item.user.login,
            html_url: item.html_url,
            head_branch: item.head.r#ref,
            base_branch: item.base.r#ref,
            draft: item.draft.unwrap_or(false),
        })
        .collect())
}

async fn fetch_pull_request(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    number: u64,
    token: Option<&str>,
) -> NestResult<PullRequestDetail> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/pulls/{number}");
    let pr: GithubPullDetail = github_get_json(http, &url, token).await?;
    Ok(PullRequestDetail {
        number: pr.number,
        title: pr.title.clone(),
        head_branch: pr.head.r#ref.clone(),
        base_branch: pr.base.r#ref.clone(),
        mergeable: pr.mergeable,
        mergeable_state: pr
            .mergeable_state
            .clone()
            .unwrap_or_else(|| "unknown".into()),
        draft: pr.draft.unwrap_or(false),
        merged: pr.merged_at.is_some(),
        html_url: pr.html_url.clone(),
        content: format_pull_request(&pr),
    })
}

async fn merge_pull_request(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    number: u64,
    merge_method: &str,
    token: Option<&str>,
) -> NestResult<String> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/pulls/{number}/merge");
    let body = MergePullRequestBody {
        merge_method: merge_method.to_string(),
    };
    let response: MergePullResponse = github_put_json(http, &url, &body, token).await?;
    if !response.merged {
        return Err(NestError::validation(
            response
                .message
                .unwrap_or_else(|| "GitHub did not merge the pull request".into()),
        ));
    }
    let detail: GithubPullDetail = github_get_json(
        http,
        &format!("{GITHUB_API}/repos/{owner}/{repo}/pulls/{number}"),
        token,
    )
    .await?;
    Ok(detail.base.r#ref)
}

fn format_pull_request(pr: &GithubPullDetail) -> String {
    let body = pr.body.as_deref().unwrap_or("_No description provided._");
    let merged_line = if pr.merged_at.is_some() {
        "**Merged:** yes  \n"
    } else {
        ""
    };
    let mergeable_line = match pr.mergeable {
        Some(true) => format!("**Mergeable:** yes ({})  \n", pr.mergeable_state.as_deref().unwrap_or("clean")),
        Some(false) => format!(
            "**Mergeable:** no ({})  \n",
            pr.mergeable_state.as_deref().unwrap_or("blocked")
        ),
        None => "**Mergeable:** unknown  \n".into(),
    };

    format!(
        "# {} (#{})\n\n\
         **State:** {}  \n\
         **Author:** @{}  \n\
         **Head:** `{}`  \n\
         **Base:** `{}`  \n\
         {merged_line}\
         {mergeable_line}\
         **Draft:** {}  \n\
         **URL:** {}\n\n\
         ---\n\n\
         {body}\n",
        pr.title,
        pr.number,
        pr.state,
        pr.user.login,
        pr.head.r#ref,
        pr.base.r#ref,
        if pr.draft.unwrap_or(false) { "yes" } else { "no" },
        pr.html_url,
    )
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

async fn github_put_json<T, B>(
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
    let mut request = HttpRequest::new(HttpMethod::Put, url)
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

#[derive(Debug, Serialize)]
struct MergePullRequestBody {
    merge_method: String,
}

#[derive(Debug, Deserialize)]
struct MergePullResponse {
    merged: bool,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GithubBranchRef {
    #[serde(rename = "ref")]
    r#ref: String,
}

#[derive(Debug, Deserialize)]
struct GithubPullSummary {
    number: u64,
    title: String,
    html_url: String,
    user: GithubUser,
    head: GithubBranchRef,
    base: GithubBranchRef,
    draft: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GithubPullDetail {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    user: GithubUser,
    body: Option<String>,
    head: GithubBranchRef,
    base: GithubBranchRef,
    draft: Option<bool>,
    merged_at: Option<String>,
    mergeable: Option<bool>,
    mergeable_state: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_pull_request_markdown() {
        let pr = GithubPullDetail {
            number: 9,
            title: "Add feature".into(),
            state: "open".into(),
            html_url: "https://github.com/o/r/pull/9".into(),
            user: GithubUser {
                login: "dev".into(),
            },
            body: Some("Summary".into()),
            head: GithubBranchRef {
                r#ref: "feature/x".into(),
            },
            base: GithubBranchRef {
                r#ref: "main".into(),
            },
            draft: Some(false),
            merged_at: None,
            mergeable: Some(true),
            mergeable_state: Some("clean".into()),
        };
        let text = format_pull_request(&pr);
        assert!(text.contains("# Add feature (#9)"));
        assert!(text.contains("`feature/x`"));
        assert!(text.contains("`main`"));
    }
}
