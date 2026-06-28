use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

pub struct GiteaClient {
    token: String,
    api_base: String, // e.g. "http://gitea.local/api/v1"
    owner: String,
    repo: String,
}

impl GiteaClient {
    /// Construct from env vars: GITEA_TOKEN, GITEA_URL, GITEA_REPO (owner/repo).
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("GITEA_TOKEN")
            .context("GITEA_TOKEN is required for Gitea operations")?;
        let url = std::env::var("GITEA_URL")
            .context("GITEA_URL is required (e.g. http://gitea.local)")?;
        let repo_full = std::env::var("GITEA_REPO")
            .context("GITEA_REPO is required (format: owner/repo)")?;

        let (owner, repo) = repo_full
            .split_once('/')
            .context("GITEA_REPO must be in owner/repo format")?;

        let api_base = format!("{}/api/v1", url.trim_end_matches('/'));

        Ok(Self {
            token,
            api_base,
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    // ── public API ────────────────────────────────────────────────────────────

    pub fn list_issues(&self, state: &str) -> Result<String> {
        let resp = self.get(&format!(
            "/repos/{}/{}/issues?type=issues&state={state}&limit=50",
            self.owner, self.repo
        ))?;
        let items = resp.as_array().cloned().unwrap_or_default();
        if items.is_empty() {
            return Ok(format!("no {state} issues"));
        }
        let mut out = format!("[{} {state} issue(s)]\n", items.len());
        for i in &items {
            let n = i["number"].as_u64().unwrap_or(0);
            let title = i["title"].as_str().unwrap_or("(no title)");
            let labels: Vec<&str> = i["labels"]
                .as_array()
                .map(|a| a.iter().filter_map(|l| l["name"].as_str()).collect())
                .unwrap_or_default();
            let label_str = if labels.is_empty() {
                String::new()
            } else {
                format!("  [{}]", labels.join(", "))
            };
            out.push_str(&format!("  #{n}  {title}{label_str}\n"));
        }
        Ok(out.trim_end().to_string())
    }

    pub fn get_issue(&self, number: u64) -> Result<String> {
        let resp = self.get(&format!(
            "/repos/{}/{}/issues/{number}",
            self.owner, self.repo
        ))?;
        let title = resp["title"].as_str().unwrap_or("(no title)");
        let body = resp["body"].as_str().unwrap_or("(no body)");
        let state = resp["state"].as_str().unwrap_or("unknown");
        Ok(format!("Issue #{number} [{state}]: {title}\n\n{body}"))
    }

    pub fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[String],
        milestone_id: Option<u64>,
    ) -> Result<String> {
        let mut payload = json!({
            "title": title,
            "body": body,
        });
        if !labels.is_empty() {
            // Gitea expects label IDs — names would need a lookup; pass names and note limitation
            payload["labels"] = json!(labels);
        }
        if let Some(id) = milestone_id {
            payload["milestone"] = json!(id);
        }
        let resp = self.post(
            &format!("/repos/{}/{}/issues", self.owner, self.repo),
            &payload,
        )?;
        let number = resp["number"].as_u64().unwrap_or(0);
        let url = resp["html_url"].as_str().unwrap_or("").to_string();
        Ok(format!("created issue #{number}: {url}"))
    }

    pub fn close_issue(&self, number: u64) -> Result<String> {
        self.patch(
            &format!("/repos/{}/{}/issues/{number}", self.owner, self.repo),
            &json!({ "state": "closed" }),
        )?;
        Ok(format!("issue #{number} closed"))
    }

    pub fn add_issue_comment(&self, number: u64, body: &str) -> Result<String> {
        let resp = self.post(
            &format!(
                "/repos/{}/{}/issues/{number}/comments",
                self.owner, self.repo
            ),
            &json!({ "body": body }),
        )?;
        let id = resp["id"].as_u64().unwrap_or(0);
        Ok(format!("added comment #{id} to issue #{number}"))
    }

    pub fn list_milestones(&self) -> Result<String> {
        let resp = self.get(&format!(
            "/repos/{}/{}/milestones?state=open&limit=50",
            self.owner, self.repo
        ))?;
        let items = resp.as_array().cloned().unwrap_or_default();
        if items.is_empty() {
            return Ok("no open milestones".to_string());
        }
        let mut out = format!("[{} open milestone(s)]\n", items.len());
        for m in &items {
            let id = m["id"].as_u64().unwrap_or(0);
            let title = m["title"].as_str().unwrap_or("");
            let due = m["due_on"].as_str().unwrap_or("no due date");
            let open = m["open_issues"].as_u64().unwrap_or(0);
            out.push_str(&format!("  #{id}  {title}  (due: {due}, open: {open})\n"));
        }
        Ok(out.trim_end().to_string())
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
        let resp = self.post(
            &format!("/repos/{}/{}/milestones", self.owner, self.repo),
            &payload,
        )?;
        let id = resp["id"].as_u64().unwrap_or(0);
        Ok(format!("created milestone #{id}: {title}"))
    }

    pub fn list_branches(&self) -> Result<String> {
        let resp = self.get(&format!(
            "/repos/{}/{}/branches?limit=50",
            self.owner, self.repo
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

    /// Create a branch via the Gitea API.
    /// `from` is the source branch name (defaults to default branch if omitted).
    pub fn create_branch(&self, name: &str, from: Option<&str>) -> Result<String> {
        let mut payload = json!({ "new_branch_name": name });
        if let Some(src) = from {
            payload["old_branch_name"] = json!(src);
        }
        self.post(
            &format!("/repos/{}/{}/branches", self.owner, self.repo),
            &payload,
        )?;
        Ok(format!("created branch '{name}'"))
    }

    pub fn list_prs(&self, state: &str) -> Result<String> {
        let resp = self.get(&format!(
            "/repos/{}/{}/pulls?state={state}&limit=50",
            self.owner, self.repo
        ))?;
        let items = resp.as_array().cloned().unwrap_or_default();
        if items.is_empty() {
            return Ok(format!("no {state} pull requests"));
        }
        let mut out = format!("[{} {state} PR(s)]\n", items.len());
        for p in &items {
            let n = p["number"].as_u64().unwrap_or(0);
            let title = p["title"].as_str().unwrap_or("(no title)");
            let head = p["head"]["label"].as_str().unwrap_or("?");
            let base = p["base"]["label"].as_str().unwrap_or("?");
            out.push_str(&format!("  #{n}  {title}  ({head} → {base})\n"));
        }
        Ok(out.trim_end().to_string())
    }

    pub fn create_pr(
        &self,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<String> {
        let payload = json!({
            "title": title,
            "body": body,
            "head": head,
            "base": base,
        });
        let resp = self.post(
            &format!("/repos/{}/{}/pulls", self.owner, self.repo),
            &payload,
        )?;
        let number = resp["number"].as_u64().unwrap_or(0);
        let url = resp["html_url"].as_str().unwrap_or("").to_string();
        Ok(format!("created PR #{number}: {url}"))
    }

    // ── internal helpers ─────────────────────────────────────────────────────

    fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{path}", self.api_base);
        let resp = ureq::get(&url)
            .set("Authorization", &format!("token {}", self.token))
            .set("Accept", "application/json")
            .call()
            .with_context(|| format!("Gitea GET {url} failed"))?;
        let val: Value = resp
            .into_json()
            .context("failed to parse Gitea response")?;
        if let Some(msg) = val.get("message").and_then(|v| v.as_str()) {
            bail!("Gitea API error: {msg}");
        }
        Ok(val)
    }

    fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{path}", self.api_base);
        let resp = ureq::post(&url)
            .set("Authorization", &format!("token {}", self.token))
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .send_json(body)
            .with_context(|| format!("Gitea POST {url} failed"))?;
        let val: Value = resp
            .into_json()
            .context("failed to parse Gitea response")?;
        if let Some(msg) = val.get("message").and_then(|v| v.as_str()) {
            bail!("Gitea API error: {msg}");
        }
        Ok(val)
    }

    fn patch(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{path}", self.api_base);
        let resp = ureq::request("PATCH", &url)
            .set("Authorization", &format!("token {}", self.token))
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .send_json(body)
            .with_context(|| format!("Gitea PATCH {url} failed"))?;
        let val: Value = resp
            .into_json()
            .context("failed to parse Gitea response")?;
        if let Some(msg) = val.get("message").and_then(|v| v.as_str()) {
            bail!("Gitea API error: {msg}");
        }
        Ok(val)
    }
}
