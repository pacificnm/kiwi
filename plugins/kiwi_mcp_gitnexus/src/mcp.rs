use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::gitea::GiteaClient;

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

pub struct Ctx {
    pub gitea: Option<GiteaClient>,
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
                "serverInfo": { "name": "kiwi-mcp-gitnexus", "version": "0.1.0" }
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

    let result: Result<String> = call_gitea(&ctx.gitea, |g| match name {
        "gitea_list_issues" => {
            let state = args.get("state").and_then(|v| v.as_str()).unwrap_or("open");
            g.list_issues(state)
        }
        "gitea_get_issue" => {
            let number = match args.get("number").and_then(|v| v.as_u64()) {
                Some(n) => n,
                None => anyhow::bail!("missing argument: number"),
            };
            g.get_issue(number)
        }
        "gitea_create_issue" => {
            let title = match args.get("title").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => anyhow::bail!("missing argument: title"),
            };
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let labels: Vec<String> = args
                .get("labels")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let milestone = args.get("milestone_id").and_then(|v| v.as_u64());
            g.create_issue(title, body, &labels, milestone)
        }
        "gitea_close_issue" => {
            let number = match args.get("number").and_then(|v| v.as_u64()) {
                Some(n) => n,
                None => anyhow::bail!("missing argument: number"),
            };
            g.close_issue(number)
        }
        "gitea_add_issue_comment" => {
            let number = match args.get("number").and_then(|v| v.as_u64()) {
                Some(n) => n,
                None => anyhow::bail!("missing argument: number"),
            };
            let body = match args.get("body").and_then(|v| v.as_str()) {
                Some(b) => b,
                None => anyhow::bail!("missing argument: body"),
            };
            g.add_issue_comment(number, body)
        }
        "gitea_list_milestones" => g.list_milestones(),
        "gitea_create_milestone" => {
            let title = match args.get("title").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => anyhow::bail!("missing argument: title"),
            };
            let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let due_on = args.get("due_on").and_then(|v| v.as_str());
            g.create_milestone(title, description, due_on)
        }
        "gitea_list_branches" => g.list_branches(),
        "gitea_create_branch" => {
            let name = match args.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => anyhow::bail!("missing argument: name"),
            };
            let from = args.get("from").and_then(|v| v.as_str());
            g.create_branch(name, from)
        }
        "gitea_list_prs" => {
            let state = args.get("state").and_then(|v| v.as_str()).unwrap_or("open");
            g.list_prs(state)
        }
        "gitea_create_pr" => {
            let title = match args.get("title").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => anyhow::bail!("missing argument: title"),
            };
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let head = match args.get("head").and_then(|v| v.as_str()) {
                Some(h) => h,
                None => anyhow::bail!("missing argument: head"),
            };
            let base = args.get("base").and_then(|v| v.as_str()).unwrap_or("main");
            g.create_pr(title, body, head, base)
        }
        other => anyhow::bail!("unknown tool: {other}"),
    });

    match result {
        Ok(text) => ok_response(id, tool_result(text)),
        Err(e) => error_response(id, -32603, &e.to_string()),
    }
}

fn call_gitea<F>(gitea: &Option<GiteaClient>, f: F) -> Result<String>
where
    F: FnOnce(&GiteaClient) -> Result<String>,
{
    match gitea {
        Some(client) => f(client),
        None => anyhow::bail!(
            "Gitea tools are unavailable: set GITEA_TOKEN, GITEA_URL, and GITEA_REPO env vars"
        ),
    }
}

fn tool_list() -> Value {
    json!([
        {
            "name": "gitea_list_issues",
            "description": "List issues on the Gitea repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "state": { "type": "string", "description": "Issue state: open or closed (default: open)", "default": "open" }
                },
                "required": []
            }
        },
        {
            "name": "gitea_get_issue",
            "description": "Fetch details for a specific Gitea issue by number.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "number": { "type": "integer", "description": "Issue number" }
                },
                "required": ["number"]
            }
        },
        {
            "name": "gitea_create_issue",
            "description": "Create a new issue on the Gitea repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title":        { "type": "string" },
                    "body":         { "type": "string", "description": "Issue body (markdown)" },
                    "labels":       { "type": "array", "items": { "type": "string" }, "description": "Label names" },
                    "milestone_id": { "type": "integer", "description": "Milestone ID (from gitea_list_milestones)" }
                },
                "required": ["title"]
            }
        },
        {
            "name": "gitea_close_issue",
            "description": "Close a Gitea issue by number.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "number": { "type": "integer", "description": "Issue number to close" }
                },
                "required": ["number"]
            }
        },
        {
            "name": "gitea_add_issue_comment",
            "description": "Add a comment to a Gitea issue.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "number": { "type": "integer", "description": "Issue number" },
                    "body":   { "type": "string", "description": "Comment text (markdown)" }
                },
                "required": ["number", "body"]
            }
        },
        {
            "name": "gitea_list_milestones",
            "description": "List open milestones on the Gitea repository (includes IDs for use in gitea_create_issue).",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "gitea_create_milestone",
            "description": "Create a milestone on the Gitea repository.",
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
            "name": "gitea_list_branches",
            "description": "List branches on the Gitea repository.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "gitea_create_branch",
            "description": "Create a new branch on the Gitea repository via the API.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "New branch name" },
                    "from": { "type": "string", "description": "Source branch name (defaults to default branch)" }
                },
                "required": ["name"]
            }
        },
        {
            "name": "gitea_list_prs",
            "description": "List pull requests on the Gitea repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "state": { "type": "string", "description": "PR state: open or closed (default: open)", "default": "open" }
                },
                "required": []
            }
        },
        {
            "name": "gitea_create_pr",
            "description": "Create a pull request on the Gitea repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "body":  { "type": "string", "description": "PR description (markdown)" },
                    "head":  { "type": "string", "description": "Source branch" },
                    "base":  { "type": "string", "description": "Target branch (default: main)", "default": "main" }
                },
                "required": ["title", "head"]
            }
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
