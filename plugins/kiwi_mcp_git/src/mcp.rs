use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::git::GitRepo;
use crate::github::GitHubClient;

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

pub struct Ctx {
    pub git: GitRepo,
    pub github: Option<GitHubClient>,
}

pub fn handle_line(line: &str, ctx: &mut Ctx) -> Option<String> {
    let req: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return Some(error_response(
                Value::Null,
                -32700,
                &format!("parse error: {e}"),
            ));
        }
    };

    let id = req.id.clone()?; // notifications have no id

    let response = match req.method.as_str() {
        "initialize" => ok_response(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "kiwi-mcp-git", "version": "0.1.0" }
            }),
        ),
        "ping" => ok_response(id, json!({})),
        "tools/list" => ok_response(id, json!({ "tools": tool_list() })),
        "tools/call" => dispatch(id, req.params.as_ref(), ctx),
        other => error_response(id, -32601, &format!("method not found: {other}")),
    };

    Some(response)
}

fn dispatch(id: Value, params: Option<&Value>, ctx: &mut Ctx) -> String {
    let params = match params {
        Some(p) => p,
        None => return error_response(id, -32602, "missing params"),
    };
    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return error_response(id, -32602, "missing tool name"),
    };
    let args = params.get("arguments").unwrap_or(&Value::Null);

    let result = match name {
        // ── git tools ──────────────────────────────────────────────────────
        "git_status" => ctx.git.status(),
        "git_diff" => {
            let staged = args.get("staged").and_then(|v| v.as_bool()).unwrap_or(false);
            ctx.git.diff(staged)
        }
        "git_add" => {
            let all = args.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
            let files: Vec<String> = args
                .get("files")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let file_refs: Vec<&str> = files.iter().map(String::as_str).collect();
            ctx.git.add(&file_refs, all)
        }
        "git_commit" => {
            let message = match args.get("message").and_then(|v| v.as_str()) {
                Some(m) => m,
                None => return error_response(id, -32602, "missing argument: message"),
            };
            let files: Vec<String> = args
                .get("files")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let file_refs: Vec<&str> = files.iter().map(String::as_str).collect();
            ctx.git.commit(message, &file_refs)
        }
        "git_log" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            ctx.git.log(limit)
        }
        "git_current_branch" => ctx.git.current_branch(),
        "git_create_branch" => {
            let name = match args.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => return error_response(id, -32602, "missing argument: name"),
            };
            let from = args.get("from").and_then(|v| v.as_str());
            ctx.git.create_branch(name, from)
        }
        "git_checkout" => {
            let branch = match args.get("branch").and_then(|v| v.as_str()) {
                Some(b) => b,
                None => return error_response(id, -32602, "missing argument: branch"),
            };
            ctx.git.checkout(branch)
        }

        // ── github tools ───────────────────────────────────────────────────
        "github_create_issue" => call_github(
            &ctx.github,
            |gh| -> Result<String> {
                let title = args["title"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing argument: title"))?;
                let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
                let labels: Vec<String> = args
                    .get("labels")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let milestone = args.get("milestone_number").and_then(|v| v.as_u64());
                gh.create_issue(title, body, &labels, milestone)
            },
        ),
        "github_create_milestone" => call_github(
            &ctx.github,
            |gh| -> Result<String> {
                let title = args["title"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing argument: title"))?;
                let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let due_on = args.get("due_on").and_then(|v| v.as_str());
                gh.create_milestone(title, description, due_on)
            },
        ),
        "github_list_milestones" => {
            call_github(&ctx.github, |gh| gh.list_milestones())
        }
        "github_create_branch" => call_github(
            &ctx.github,
            |gh| -> Result<String> {
                let name = args["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing argument: name"))?;
                let from = args.get("from").and_then(|v| v.as_str());
                gh.create_branch(name, from)
            },
        ),
        "github_create_pr" => call_github(
            &ctx.github,
            |gh| -> Result<String> {
                let title = args["title"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing argument: title"))?;
                let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
                let head = args["head"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing argument: head"))?;
                let base = args.get("base").and_then(|v| v.as_str()).unwrap_or("main");
                let draft = args.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
                gh.create_pr(title, body, head, base, draft)
            },
        ),
        "github_list_branches" => {
            call_github(&ctx.github, |gh| gh.list_branches())
        }

        other => return error_response(id, -32602, &format!("unknown tool: {other}")),
    };

    match result {
        Ok(text) => ok_response(id, tool_result(text)),
        Err(e) => error_response(id, -32603, &e.to_string()),
    }
}

fn call_github<F>(gh: &Option<GitHubClient>, f: F) -> Result<String>
where
    F: FnOnce(&GitHubClient) -> Result<String>,
{
    match gh {
        Some(client) => f(client),
        None => anyhow::bail!(
            "GitHub tools are unavailable: set GITHUB_TOKEN and GITHUB_REPO env vars"
        ),
    }
}

fn tool_list() -> Value {
    json!([
        {
            "name": "git_status",
            "description": "Show the working tree status (branch, modified, staged, untracked files).",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "git_diff",
            "description": "Show uncommitted changes. Set staged=true to see staged (index) diff.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "staged": { "type": "boolean", "description": "Show staged diff instead of unstaged", "default": false }
                },
                "required": []
            }
        },
        {
            "name": "git_add",
            "description": "Stage files for the next commit.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "files": { "type": "array", "items": { "type": "string" }, "description": "Paths to stage" },
                    "all":   { "type": "boolean", "description": "Stage all changes (git add -A)", "default": false }
                },
                "required": []
            }
        },
        {
            "name": "git_commit",
            "description": "Stage optional files then create a commit.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "message": { "type": "string", "description": "Commit message" },
                    "files":   { "type": "array", "items": { "type": "string" }, "description": "Files to stage before committing (optional)" }
                },
                "required": ["message"]
            }
        },
        {
            "name": "git_log",
            "description": "Show recent commit history (one-line format).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "description": "Number of commits to show", "default": 10 }
                },
                "required": []
            }
        },
        {
            "name": "git_current_branch",
            "description": "Return the name of the currently checked-out branch.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "git_create_branch",
            "description": "Create a new local branch and switch to it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "New branch name" },
                    "from": { "type": "string", "description": "Starting branch or SHA (defaults to current HEAD)" }
                },
                "required": ["name"]
            }
        },
        {
            "name": "git_checkout",
            "description": "Switch to an existing local branch.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "branch": { "type": "string", "description": "Branch to check out" }
                },
                "required": ["branch"]
            }
        },
        {
            "name": "github_create_issue",
            "description": "Create a GitHub issue.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title":            { "type": "string" },
                    "body":             { "type": "string", "description": "Issue body (markdown)" },
                    "labels":           { "type": "array", "items": { "type": "string" } },
                    "milestone_number": { "type": "integer", "description": "Milestone number (from github_list_milestones)" }
                },
                "required": ["title"]
            }
        },
        {
            "name": "github_create_milestone",
            "description": "Create a GitHub milestone.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title":       { "type": "string" },
                    "description": { "type": "string" },
                    "due_on":      { "type": "string", "description": "ISO 8601 date-time, e.g. 2025-12-31T00:00:00Z" }
                },
                "required": ["title"]
            }
        },
        {
            "name": "github_list_milestones",
            "description": "List open GitHub milestones (includes their numbers for use in github_create_issue).",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "github_create_branch",
            "description": "Create a remote branch on GitHub via the API (does not require a local push).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "New branch name" },
                    "from": { "type": "string", "description": "Source branch name or SHA (defaults to the default branch HEAD)" }
                },
                "required": ["name"]
            }
        },
        {
            "name": "github_create_pr",
            "description": "Open a GitHub pull request.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "body":  { "type": "string", "description": "PR description (markdown)" },
                    "head":  { "type": "string", "description": "Branch containing the changes" },
                    "base":  { "type": "string", "description": "Target branch (default: main)", "default": "main" },
                    "draft": { "type": "boolean", "description": "Open as draft PR", "default": false }
                },
                "required": ["title", "head"]
            }
        },
        {
            "name": "github_list_branches",
            "description": "List remote branches on GitHub.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        }
    ])
}

fn ok_response(id: Value, result: Value) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
    .unwrap_or_default()
}

fn error_response(id: Value, code: i32, message: &str) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    }))
    .unwrap_or_default()
}

fn tool_result(text: String) -> Value {
    json!({ "content": [{ "type": "text", "text": text }] })
}
