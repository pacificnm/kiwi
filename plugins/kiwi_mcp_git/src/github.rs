use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

pub struct GitHubClient {
    token: String,
    repo: String,   // "owner/repo"
    api_url: String,
}

impl GitHubClient {
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN")
            .context("GITHUB_TOKEN is required for GitHub operations")?;
        let repo = std::env::var("GITHUB_REPO")
            .context("GITHUB_REPO is required (format: owner/repo)")?;
        let api_url = std::env::var("GITHUB_API_URL")
            .unwrap_or_else(|_| "https://api.github.com".into());
        Ok(Self { token, repo, api_url })
    }

    pub fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[String],
        milestone_number: Option<u64>,
    ) -> Result<String> {
        let mut payload = json!({
            "title": title,
            "body": body,
            "labels": labels,
        });
        if let Some(n) = milestone_number {
            payload["milestone"] = json!(n);
        }
        let resp = self.post(&format!("repos/{}/issues", self.repo), &payload)?;
        let number = resp["number"].as_u64().unwrap_or(0);
        let url = resp["html_url"].as_str().unwrap_or("").to_string();
        Ok(format!("created issue #{number}: {url}"))
    }

    pub fn create_milestone(
        &self,
        title: &str,
        description: &str,
        due_on: Option<&str>,
    ) -> Result<String> {
        let mut payload = json!({
            "title": title,
            "description": description,
        });
        if let Some(d) = due_on {
            payload["due_on"] = json!(d);
        }
        let resp = self.post(&format!("repos/{}/milestones", self.repo), &payload)?;
        let number = resp["number"].as_u64().unwrap_or(0);
        let url = resp["html_url"].as_str().unwrap_or("").to_string();
        Ok(format!("created milestone #{number}: {url}"))
    }

    pub fn list_milestones(&self) -> Result<String> {
        let resp = self.get(&format!(
            "repos/{}/milestones?state=open&per_page=50",
            self.repo
        ))?;
        let items = resp.as_array().cloned().unwrap_or_default();
        if items.is_empty() {
            return Ok("no open milestones".to_string());
        }
        let mut out = format!("[{} open milestone(s)]\n", items.len());
        for m in &items {
            let n = m["number"].as_u64().unwrap_or(0);
            let title = m["title"].as_str().unwrap_or("");
            let due = m["due_on"].as_str().unwrap_or("no due date");
            let open = m["open_issues"].as_u64().unwrap_or(0);
            out.push_str(&format!("  #{n}  {title}  (due: {due}, open issues: {open})\n"));
        }
        Ok(out.trim_end().to_string())
    }

    /// Create a remote branch via the Git Data API.
    /// `from` defaults to the default branch if omitted.
    pub fn create_branch(&self, name: &str, from: Option<&str>) -> Result<String> {
        // Resolve the SHA of the source ref
        let source = from.unwrap_or("HEAD");
        let sha = self.resolve_ref(source)?;
        let payload = json!({
            "ref": format!("refs/heads/{name}"),
            "sha": sha,
        });
        let resp = self.post(&format!("repos/{}/git/refs", self.repo), &payload)?;
        let url = resp["url"].as_str().unwrap_or("").to_string();
        Ok(format!("created remote branch '{name}' at {sha:.8}: {url}"))
    }

    pub fn create_pr(
        &self,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
        draft: bool,
    ) -> Result<String> {
        let payload = json!({
            "title": title,
            "body": body,
            "head": head,
            "base": base,
            "draft": draft,
        });
        let resp = self.post(&format!("repos/{}/pulls", self.repo), &payload)?;
        let number = resp["number"].as_u64().unwrap_or(0);
        let url = resp["html_url"].as_str().unwrap_or("").to_string();
        Ok(format!("created PR #{number}: {url}"))
    }

    pub fn list_branches(&self) -> Result<String> {
        let resp = self.get(&format!(
            "repos/{}/branches?per_page=50",
            self.repo
        ))?;
        let items = resp.as_array().cloned().unwrap_or_default();
        if items.is_empty() {
            return Ok("no branches found".to_string());
        }
        let names: Vec<&str> = items
            .iter()
            .filter_map(|b| b["name"].as_str())
            .collect();
        Ok(names.join("\n"))
    }

    // ── internal helpers ─────────────────────────────────────────────────────

    fn resolve_ref(&self, refname: &str) -> Result<String> {
        // Try as a branch first
        if let Ok(resp) = self.get(&format!(
            "repos/{}/git/ref/heads/{refname}",
            self.repo
        )) {
            if let Some(sha) = resp.pointer("/object/sha").and_then(|v| v.as_str()) {
                return Ok(sha.to_string());
            }
        }
        // Try as a tag
        if let Ok(resp) = self.get(&format!(
            "repos/{}/git/ref/tags/{refname}",
            self.repo
        )) {
            if let Some(sha) = resp.pointer("/object/sha").and_then(|v| v.as_str()) {
                return Ok(sha.to_string());
            }
        }
        // Fall back to commit SHA directly
        if refname.len() >= 7 && refname.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(refname.to_string());
        }
        bail!("could not resolve ref '{refname}' to a SHA")
    }

    fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}/{path}", self.api_url);
        let resp = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(body)
            .with_context(|| format!("GitHub API POST {url} failed"))?;

        let val: Value = resp
            .into_json()
            .context("failed to parse GitHub API response")?;

        if let Some(msg) = val.get("message").and_then(|v| v.as_str()) {
            bail!("GitHub API error: {msg}");
        }
        Ok(val)
    }

    fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}/{path}", self.api_url);
        let resp = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .call()
            .with_context(|| format!("GitHub API GET {url} failed"))?;

        let val: Value = resp
            .into_json()
            .context("failed to parse GitHub API response")?;

        if let Some(msg) = val.get("message").and_then(|v| v.as_str()) {
            bail!("GitHub API error: {msg}");
        }
        Ok(val)
    }
}
