use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::MemoryDb;
use crate::embed::EmbedClient;

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

pub fn handle_line(
    line: &str,
    db: &mut MemoryDb,
    embed: &EmbedClient,
) -> Option<String> {
    let req: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            let resp = error_response(Value::Null, -32700, &format!("parse error: {e}"));
            return Some(resp);
        }
    };

    // Notifications have no id — no response required
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
                "name": "kiwi-mcp-memory",
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
                    "name": "search_project_memory",
                    "description": "Search indexed project documentation using semantic similarity.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "The search query"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results to return (default 8)",
                                "default": 8
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "search_knowledge_base",
                    "description": "Search reference documentation (Rust book, eGUI, React, etc.) indexed into the knowledge base.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "The search query"
                            },
                            "collection": {
                                "type": "string",
                                "description": "Filter by collection name (e.g. 'rust-book', 'egui'). Omit to search all collections."
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results to return (default 8)",
                                "default": 8
                            }
                        },
                        "required": ["query"]
                    }
                }
            ]
        }),
    )
}

fn handle_tools_call(
    id: Value,
    params: Option<&Value>,
    db: &mut MemoryDb,
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
        "search_project_memory" => {
            let query = match args.get("query").and_then(|v| v.as_str()) {
                Some(q) => q,
                None => return error_response(id, -32602, "missing argument: query"),
            };
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(8) as i32;
            search_project(query, limit, db, embed)
        }
        "search_knowledge_base" => {
            let query = match args.get("query").and_then(|v| v.as_str()) {
                Some(q) => q,
                None => return error_response(id, -32602, "missing argument: query"),
            };
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(8) as i32;
            let collection = args.get("collection").and_then(|v| v.as_str());
            search_knowledge(query, limit, collection, db, embed)
        }
        other => return error_response(id, -32602, &format!("unknown tool: {other}")),
    };

    match result {
        Ok(text) => ok_response(id, tool_result(text)),
        Err(e) => error_response(id, -32603, &format!("search failed: {e}")),
    }
}

fn search_project(query: &str, limit: i32, db: &mut MemoryDb, embed: &EmbedClient) -> Result<String> {
    let embedding = embed.embed(query)?;
    let results = db.search(&embedding, limit)?;

    if results.is_empty() {
        return Ok("No results found.".to_string());
    }

    let mut out = format!("[{} results for \"{query}\"]\n\n", results.len());
    for r in &results {
        out.push_str(&format!(
            "--- {} (score: {:.3}) ---\n{}\n\n",
            r.source_path,
            r.score,
            r.content.trim()
        ));
    }
    Ok(out.trim_end().to_string())
}

fn search_knowledge(
    query: &str,
    limit: i32,
    collection: Option<&str>,
    db: &mut MemoryDb,
    embed: &EmbedClient,
) -> Result<String> {
    let embedding = embed.embed(query)?;
    let results = db.search_knowledge(&embedding, limit, collection)?;

    if results.is_empty() {
        return Ok("No results found.".to_string());
    }

    let header = match collection {
        Some(c) => format!("[{} results for \"{query}\" in '{c}']\n\n", results.len()),
        None => format!("[{} results for \"{query}\"]\n\n", results.len()),
    };
    let mut out = header;
    for r in &results {
        out.push_str(&format!(
            "--- {}/{} (score: {:.3}) ---\n{}\n\n",
            r.collection,
            r.source_path,
            r.score,
            r.content.trim()
        ));
    }
    Ok(out.trim_end().to_string())
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
