use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::ContextDb;
use crate::embed::EmbedClient;

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

pub fn handle_line(
    line: &str,
    db: &mut ContextDb,
    embed: &EmbedClient,
) -> Option<String> {
    let req: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            let resp = error_response(Value::Null, -32700, &format!("parse error: {e}"));
            return Some(resp);
        }
    };

    let id = req.id.clone()?;

    let response = match req.method.as_str() {
        "initialize" => handle_initialize(id),
        "ping" => ok_response(id, json!({})),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tools_call(id, req.params.as_ref(), db, embed),
        other => error_response(id, -32601, &format!("method not found: {other}")),
    };

    Some(response)
}

fn handle_initialize(id: Value) -> String {
    ok_response(
        id,
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "kiwi-mcp-context",
                "version": "0.1.0"
            }
        }),
    )
}

fn handle_tools_list(id: Value) -> String {
    ok_response(
        id,
        json!({
            "tools": [
                {
                    "name": "save_context_memory",
                    "description": "Save a context memory entry for the current agent session.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "content": { "type": "string", "description": "The content to save" },
                            "title": { "type": "string", "description": "Short descriptive title" },
                            "session_key": { "type": "string", "description": "Session identifier (e.g. 'main:abc123')" },
                            "tags": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Optional tags for categorization"
                            }
                        },
                        "required": ["content", "title", "session_key"]
                    }
                },
                {
                    "name": "search_context_memory",
                    "description": "Search context memory using semantic similarity.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": { "type": "string", "description": "The search query" },
                            "limit": { "type": "integer", "description": "Max results (default 8)", "default": 8 },
                            "session_key": { "type": "string", "description": "Filter by session (empty = all sessions)" }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "list_context_memory",
                    "description": "List recent context memory entries.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "limit": { "type": "integer", "description": "Max entries (default 20)", "default": 20 },
                            "session_key": { "type": "string", "description": "Filter by session (empty = all)" }
                        }
                    }
                },
                {
                    "name": "get_context_memory",
                    "description": "Retrieve a single context memory entry by ID.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "entry_id": { "type": "integer", "description": "The entry ID" }
                        },
                        "required": ["entry_id"]
                    }
                }
            ]
        }),
    )
}

fn handle_tools_call(
    id: Value,
    params: Option<&Value>,
    db: &mut ContextDb,
    embed: &EmbedClient,
) -> String {
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
        "save_context_memory" => dispatch_save(args, db, embed),
        "search_context_memory" => dispatch_search(args, db, embed),
        "list_context_memory" => dispatch_list(args, db),
        "get_context_memory" => dispatch_get(args, db),
        other => return error_response(id, -32602, &format!("unknown tool: {other}")),
    };

    match result {
        Ok(text) => ok_response(id, tool_result(text)),
        Err(e) => error_response(id, -32603, &format!("tool error: {e}")),
    }
}

fn dispatch_save(
    args: &Value,
    db: &mut ContextDb,
    embed: &EmbedClient,
) -> anyhow::Result<String> {
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing argument: content"))?;
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let session_key = args
        .get("session_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let tags: Vec<String> = args
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();

    let embedding = embed.embed(content)?;
    let id = db.save(content, title, session_key, &tags, &embedding)?;
    Ok(format!("Saved context memory entry #{id}"))
}

fn dispatch_search(
    args: &Value,
    db: &mut ContextDb,
    embed: &EmbedClient,
) -> anyhow::Result<String> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing argument: query"))?;
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(8);
    let session_key = args
        .get("session_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let embedding = embed.embed(query)?;
    let results = db.search(&embedding, limit, session_key)?;

    if results.is_empty() {
        return Ok("No results found.".to_string());
    }

    let mut out = format!("[{} results for \"{query}\"]\n", results.len());
    for r in &results {
        let preview: String = r.content.chars().take(2000).collect();
        let tags_str = r.tags.join(", ");
        out.push_str(&format!(
            "[{}] {} | {} | session: {} | tags: {}\n{}\n---\n",
            r.id, r.created_at, r.title, r.session_key, tags_str, preview
        ));
    }
    Ok(out.trim_end().to_string())
}

fn dispatch_list(args: &Value, db: &mut ContextDb) -> anyhow::Result<String> {
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(20);
    let session_key = args
        .get("session_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let entries = db.list(limit, session_key)?;

    if entries.is_empty() {
        return Ok("No entries found.".to_string());
    }

    let mut out = format!("[{} recent entries]\n", entries.len());
    for e in &entries {
        let preview: String = e.content.chars().take(500).collect();
        out.push_str(&format!(
            "[{}] {} | {}\n{}\n---\n",
            e.id, e.created_at, e.title, preview
        ));
    }
    Ok(out.trim_end().to_string())
}

fn dispatch_get(args: &Value, db: &mut ContextDb) -> anyhow::Result<String> {
    let entry_id = args
        .get("entry_id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow::anyhow!("missing argument: entry_id"))?;

    match db.get(entry_id)? {
        None => Ok(format!("No entry found with id {entry_id}")),
        Some(e) => {
            let content: String = e.content.chars().take(10_000).collect();
            let tags_str = e.tags.join(", ");
            Ok(format!(
                "[{}] {} | {}\ntags: {}\n\n{}",
                e.id, e.created_at, e.title, tags_str, content
            ))
        }
    }
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
    json!({
        "content": [{ "type": "text", "text": text }]
    })
}
