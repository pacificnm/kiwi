//! GitHub repository labels, milestones, and issue metadata API.

use std::sync::mpsc;
use std::thread;

use nest_error::{NestError, NestResult};
use nest_http_client::HttpClientService;
use nest_http::HttpMethod;
use nest_http_client::HttpRequest;
use serde::{Deserialize, Serialize};

use super::github::{map_github_api_error, GITHUB_API};

/// A repository label from the GitHub API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubLabel {
    /// Label name (unique per repository).
    pub name: String,
    /// Six-digit hex color without `#`.
    pub color: String,
    /// Optional description.
    pub description: String,
}

/// A repository milestone from the GitHub API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubMilestone {
    /// Milestone number.
    pub number: u64,
    /// Title.
    pub title: String,
    /// Optional description.
    pub description: String,
    /// `open` or `closed`.
    pub state: String,
    /// Optional due date (`YYYY-MM-DDTHH:MM:SSZ`).
    pub due_on: Option<String>,
}

/// Background metadata operation result.
#[derive(Debug)]
pub enum MetadataEvent {
    /// Repository labels loaded.
    LabelsListed(Vec<GitHubLabel>),
    /// Label created or updated.
    LabelSaved(GitHubLabel),
    /// Label deleted.
    LabelDeleted {
        /// Deleted label name.
        name: String,
    },
    /// Milestones loaded.
    MilestonesListed(Vec<GitHubMilestone>),
    /// Milestone created or updated.
    MilestoneSaved(GitHubMilestone),
    /// Milestone deleted.
    MilestoneDeleted {
        /// Deleted milestone number.
        number: u64,
    },
    /// Issue metadata loaded for the assign modal.
    IssueMetadataLoaded {
        /// Issue number.
        issue_number: u64,
        /// All repository labels.
        labels: Vec<GitHubLabel>,
        /// Open milestones.
        milestones: Vec<GitHubMilestone>,
        /// Label names currently on the issue.
        selected_labels: Vec<String>,
        /// Current milestone number, if any.
        milestone: Option<u64>,
    },
    /// Issue labels/milestone updated.
    IssueUpdated {
        /// Updated issue number.
        issue_number: u64,
    },
    /// Operation failed.
    Failed {
        /// Short context for the UI.
        context: &'static str,
        /// Error message.
        error: String,
    },
}

/// Loads repository labels on a background thread.
pub fn spawn_list_labels(
    http: HttpClientService,
    owner: String,
    repo: String,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match fetch_labels(http, &owner, &repo, token.as_deref()).await {
                Ok(labels) => MetadataEvent::LabelsListed(labels),
                Err(error) => MetadataEvent::Failed {
                    context: "list labels",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

/// Creates a repository label on a background thread.
pub fn spawn_create_label(
    http: HttpClientService,
    owner: String,
    repo: String,
    name: String,
    color: String,
    description: String,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match create_label(
                http,
                &owner,
                &repo,
                &name,
                &color,
                &description,
                token.as_deref(),
            )
            .await
            {
                Ok(label) => MetadataEvent::LabelSaved(label),
                Err(error) => MetadataEvent::Failed {
                    context: "create label",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

/// Updates a repository label on a background thread.
pub fn spawn_update_label(
    http: HttpClientService,
    owner: String,
    repo: String,
    original_name: String,
    name: String,
    color: String,
    description: String,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match update_label(
                http,
                &owner,
                &repo,
                &original_name,
                &name,
                &color,
                &description,
                token.as_deref(),
            )
            .await
            {
                Ok(label) => MetadataEvent::LabelSaved(label),
                Err(error) => MetadataEvent::Failed {
                    context: "update label",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

/// Deletes a repository label on a background thread.
pub fn spawn_delete_label(
    http: HttpClientService,
    owner: String,
    repo: String,
    name: String,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match delete_label(http, &owner, &repo, &name, token.as_deref()).await {
                Ok(()) => MetadataEvent::LabelDeleted { name },
                Err(error) => MetadataEvent::Failed {
                    context: "delete label",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

/// Loads repository milestones on a background thread.
pub fn spawn_list_milestones(
    http: HttpClientService,
    owner: String,
    repo: String,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match fetch_milestones(http, &owner, &repo, token.as_deref()).await {
                Ok(milestones) => MetadataEvent::MilestonesListed(milestones),
                Err(error) => MetadataEvent::Failed {
                    context: "list milestones",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

/// Creates a milestone on a background thread.
pub fn spawn_create_milestone(
    http: HttpClientService,
    owner: String,
    repo: String,
    title: String,
    description: String,
    due_on: Option<String>,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match create_milestone(
                http,
                &owner,
                &repo,
                &title,
                &description,
                due_on.as_deref(),
                token.as_deref(),
            )
            .await
            {
                Ok(milestone) => MetadataEvent::MilestoneSaved(milestone),
                Err(error) => MetadataEvent::Failed {
                    context: "create milestone",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

/// Updates a milestone on a background thread.
pub fn spawn_update_milestone(
    http: HttpClientService,
    owner: String,
    repo: String,
    number: u64,
    title: String,
    description: String,
    due_on: Option<String>,
    state: String,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match update_milestone(
                http,
                &owner,
                &repo,
                number,
                &title,
                &description,
                due_on.as_deref(),
                &state,
                token.as_deref(),
            )
            .await
            {
                Ok(milestone) => MetadataEvent::MilestoneSaved(milestone),
                Err(error) => MetadataEvent::Failed {
                    context: "update milestone",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

/// Deletes a milestone on a background thread.
pub fn spawn_delete_milestone(
    http: HttpClientService,
    owner: String,
    repo: String,
    number: u64,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match delete_milestone(http, &owner, &repo, number, token.as_deref()).await {
                Ok(()) => MetadataEvent::MilestoneDeleted { number },
                Err(error) => MetadataEvent::Failed {
                    context: "delete milestone",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

/// Loads issue metadata for the assign modal on a background thread.
pub fn spawn_load_issue_metadata(
    http: HttpClientService,
    owner: String,
    repo: String,
    issue_number: u64,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match load_issue_metadata(
                http,
                &owner,
                &repo,
                issue_number,
                token.as_deref(),
            )
            .await
            {
                Ok(data) => MetadataEvent::IssueMetadataLoaded {
                    issue_number,
                    labels: data.labels,
                    milestones: data.milestones,
                    selected_labels: data.selected_labels,
                    milestone: data.milestone,
                },
                Err(error) => MetadataEvent::Failed {
                    context: "load issue metadata",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

/// Updates issue labels and milestone on a background thread.
pub fn spawn_update_issue_metadata(
    http: HttpClientService,
    owner: String,
    repo: String,
    issue_number: u64,
    labels: Vec<String>,
    milestone: Option<u64>,
    token: Option<String>,
) -> mpsc::Receiver<MetadataEvent> {
    spawn_metadata(http, move |http, tx| {
        let event = block_on(async {
            match update_issue_metadata(
                http,
                &owner,
                &repo,
                issue_number,
                &labels,
                milestone,
                token.as_deref(),
            )
            .await
            {
                Ok(()) => MetadataEvent::IssueUpdated { issue_number },
                Err(error) => MetadataEvent::Failed {
                    context: "update issue",
                    error: error.to_string(),
                },
            }
        });
        let _ = tx.send(event);
    })
}

struct IssueMetadataLoad {
    labels: Vec<GitHubLabel>,
    milestones: Vec<GitHubMilestone>,
    selected_labels: Vec<String>,
    milestone: Option<u64>,
}

fn spawn_metadata(
    http: HttpClientService,
    run: impl FnOnce(&HttpClientService, mpsc::Sender<MetadataEvent>) + Send + 'static,
) -> mpsc::Receiver<MetadataEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || run(&http, tx));
    rx
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(future)
}

async fn fetch_labels(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    token: Option<&str>,
) -> NestResult<Vec<GitHubLabel>> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/labels?per_page=100");
    let raw: Vec<GithubLabelJson> = github_get_json(http, &url, token).await?;
    Ok(raw.into_iter().map(GithubLabelJson::into_label).collect())
}

async fn create_label(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    name: &str,
    color: &str,
    description: &str,
    token: Option<&str>,
) -> NestResult<GitHubLabel> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/labels");
    let payload = CreateLabelBody {
        name,
        color: normalize_color(color),
        description,
    };
    let raw: GithubLabelJson = github_post_json(http, &url, &payload, token).await?;
    Ok(raw.into_label())
}

async fn update_label(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    original_name: &str,
    name: &str,
    color: &str,
    description: &str,
    token: Option<&str>,
) -> NestResult<GitHubLabel> {
    let encoded = urlencoding::encode(original_name);
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/labels/{encoded}");
    let payload = UpdateLabelBody {
        new_name: if name == original_name {
            None
        } else {
            Some(name)
        },
        color: Some(normalize_color(color)),
        description: Some(description),
    };
    let raw: GithubLabelJson = github_patch_json(http, &url, &payload, token).await?;
    Ok(raw.into_label())
}

async fn delete_label(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    name: &str,
    token: Option<&str>,
) -> NestResult<()> {
    let encoded = urlencoding::encode(name);
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/labels/{encoded}");
    github_delete(http, &url, token).await
}

async fn fetch_milestones(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    token: Option<&str>,
) -> NestResult<Vec<GitHubMilestone>> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/milestones?state=all&per_page=100");
    let raw: Vec<GithubMilestoneJson> = github_get_json(http, &url, token).await?;
    Ok(raw
        .into_iter()
        .map(GithubMilestoneJson::into_milestone)
        .collect())
}

async fn create_milestone(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    title: &str,
    description: &str,
    due_on: Option<&str>,
    token: Option<&str>,
) -> NestResult<GitHubMilestone> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/milestones");
    let payload = CreateMilestoneBody {
        title,
        description,
        due_on,
        state: "open",
    };
    let raw: GithubMilestoneJson = github_post_json(http, &url, &payload, token).await?;
    Ok(raw.into_milestone())
}

async fn update_milestone(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    number: u64,
    title: &str,
    description: &str,
    due_on: Option<&str>,
    state: &str,
    token: Option<&str>,
) -> NestResult<GitHubMilestone> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/milestones/{number}");
    let payload = UpdateMilestoneBody {
        title,
        description,
        due_on,
        state,
    };
    let raw: GithubMilestoneJson = github_patch_json(http, &url, &payload, token).await?;
    Ok(raw.into_milestone())
}

async fn delete_milestone(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    number: u64,
    token: Option<&str>,
) -> NestResult<()> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/milestones/{number}");
    github_delete(http, &url, token).await
}

async fn load_issue_metadata(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    issue_number: u64,
    token: Option<&str>,
) -> NestResult<IssueMetadataLoad> {
    let issue_url = format!("{GITHUB_API}/repos/{owner}/{repo}/issues/{issue_number}");
    let issue: GithubIssueMetadataJson = github_get_json(http, &issue_url, token).await?;
    let labels = fetch_labels(http, owner, repo, token).await?;
    let milestones = fetch_milestones(http, owner, repo, token).await?;
    Ok(IssueMetadataLoad {
        selected_labels: issue
            .labels
            .into_iter()
            .map(|label| label.name)
            .collect(),
        milestone: issue.milestone.map(|m| m.number),
        labels,
        milestones,
    })
}

async fn update_issue_metadata(
    http: &HttpClientService,
    owner: &str,
    repo: &str,
    issue_number: u64,
    labels: &[String],
    milestone: Option<u64>,
    token: Option<&str>,
) -> NestResult<()> {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo}/issues/{issue_number}");
    let payload = UpdateIssueMetadataBody {
        labels,
        milestone,
    };
    let _: GithubIssueMetadataJson = github_patch_json(http, &url, &payload, token).await?;
    Ok(())
}

fn normalize_color(color: &str) -> String {
    color.trim().trim_start_matches('#').to_string()
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
    github_json_body(http, HttpMethod::Post, url, body, token).await
}

async fn github_patch_json<T, B>(
    http: &HttpClientService,
    url: &str,
    body: &B,
    token: Option<&str>,
) -> NestResult<T>
where
    T: for<'de> Deserialize<'de>,
    B: Serialize + ?Sized,
{
    github_json_body(http, HttpMethod::Patch, url, body, token).await
}

async fn github_json_body<T, B>(
    http: &HttpClientService,
    method: HttpMethod,
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
    let mut request = HttpRequest::new(method, url)
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

async fn github_delete(
    http: &HttpClientService,
    url: &str,
    token: Option<&str>,
) -> NestResult<()> {
    let mut request = HttpRequest::new(HttpMethod::Delete, url)
        .with_header("Accept", "application/vnd.github+json")
        .with_header("X-GitHub-Api-Version", "2022-11-28");
    if let Some(token) = token.filter(|value| !value.is_empty()) {
        request = request.with_header("Authorization", format!("Bearer {token}"));
    }

    let response = http.send(request).await?;
    if !response.status.is_success() {
        return Err(map_github_api_error(response.status.code(), &response.body));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct CreateLabelBody<'a> {
    name: &'a str,
    color: String,
    description: &'a str,
}

#[derive(Debug, Serialize)]
struct UpdateLabelBody<'a> {
    #[serde(rename = "new_name", skip_serializing_if = "Option::is_none")]
    new_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct CreateMilestoneBody<'a> {
    title: &'a str,
    description: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_on: Option<&'a str>,
    state: &'a str,
}

#[derive(Debug, Serialize)]
struct UpdateMilestoneBody<'a> {
    title: &'a str,
    description: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_on: Option<&'a str>,
    state: &'a str,
}

#[derive(Debug, Serialize)]
struct UpdateIssueMetadataBody<'a> {
    labels: &'a [String],
    milestone: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct GithubLabelJson {
    name: String,
    color: String,
    description: Option<String>,
}

impl GithubLabelJson {
    fn into_label(self) -> GitHubLabel {
        GitHubLabel {
            name: self.name,
            color: self.color,
            description: self.description.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GithubMilestoneJson {
    number: u64,
    title: String,
    description: Option<String>,
    state: String,
    due_on: Option<String>,
}

impl GithubMilestoneJson {
    fn into_milestone(self) -> GitHubMilestone {
        GitHubMilestone {
            number: self.number,
            title: self.title,
            description: self.description.unwrap_or_default(),
            state: self.state,
            due_on: self.due_on,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GithubIssueMetadataJson {
    labels: Vec<GithubLabelJson>,
    milestone: Option<GithubMilestoneRefJson>,
}

#[derive(Debug, Deserialize)]
struct GithubMilestoneRefJson {
    number: u64,
}
