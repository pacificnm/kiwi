//! One-shot MCP client for kiwi-memory tools used by the agent executor.
//!
//! Spawns `kiwi-mcp-memory` as a subprocess per call so `tool_executor` stays
//! decoupled from MCP server internals (db, embed, indexer).

use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

const DEFAULT_MEMORY_BINARY: &str = "kiwi-mcp-memory";

/// Resolve the kiwi-mcp-memory binary path (`KIWI_MCP_MEMORY_BIN` overrides).
pub fn memory_binary_path() -> String {
    std::env::var("KIWI_MCP_MEMORY_BIN").unwrap_or_else(|_| DEFAULT_MEMORY_BINARY.to_string())
}

/// Search indexed project documentation via the `search_project_memory` MCP tool.
pub fn search_project_memory(query: &str, limit: u32) -> Result<String, String> {
    call_memory_tool(
        &memory_binary_path(),
        "search_project_memory",
        json!({ "query": query, "limit": limit }),
    )
}

/// Call a single MCP tool on a freshly spawned memory server process.
pub fn call_memory_tool(binary: &str, tool_name: &str, arguments: Value) -> Result<String, String> {
    let mut cmd = Command::new(binary);
    for var in &[
        "DATABASE_URL",
        "EMBED_BACKEND",
        "OLLAMA_URL",
        "OLLAMA_EMBED_MODEL",
        "OPENAI_API_KEY",
        "OPENAI_EMBED_MODEL",
        "OPENAI_EMBED_DIMENSIONS",
    ] {
        if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| spawn_error(binary, err))?;

    let mut writer = BufWriter::new(
        child
            .stdin
            .take()
            .ok_or_else(|| "MCP server stdin unavailable".to_string())?,
    );
    let mut reader = BufReader::new(
        child
            .stdout
            .take()
            .ok_or_else(|| "MCP server stdout unavailable".to_string())?,
    );

    send_request(
        &mut writer,
        &mut reader,
        1,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "kiwi-agent", "version": "0.1.0" }
        }),
    )?;

    let notif = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    }))
    .map_err(|err| format!("MCP notification encode failed: {err}"))?;
    writer
        .write_all(format!("{notif}\n").as_bytes())
        .map_err(|err| format!("MCP write failed: {err}"))?;
    writer
        .flush()
        .map_err(|err| format!("MCP flush failed: {err}"))?;

    let response = send_request(
        &mut writer,
        &mut reader,
        2,
        "tools/call",
        json!({ "name": tool_name, "arguments": arguments }),
    )?;

    let _ = child.wait();

    if let Some(err) = response.get("error") {
        return Err(format!("MCP error: {err}"));
    }

    response
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "MCP tool returned no text content".to_string())
}

fn send_request(
    writer: &mut BufWriter<impl Write>,
    reader: &mut BufReader<impl Read>,
    id: u64,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    let req = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    }))
    .map_err(|err| format!("MCP request encode failed: {err}"))?;

    writer
        .write_all(format!("{req}\n").as_bytes())
        .map_err(|err| format!("MCP write failed: {err}"))?;
    writer
        .flush()
        .map_err(|err| format!("MCP flush failed: {err}"))?;

    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|err| format!("MCP read failed: {err}"))?;
    if bytes == 0 || line.trim().is_empty() {
        return Err(
            "MCP server exited during startup. Check DATABASE_URL, PostgreSQL, and embedding \
             backend configuration (see tools/MCP-SETUP.md)."
                .to_string(),
        );
    }

    serde_json::from_str(line.trim())
        .map_err(|err| format!("MCP response parse failed: {err}; line={line}"))
}

fn spawn_error(binary: &str, err: std::io::Error) -> String {
    if err.kind() == std::io::ErrorKind::NotFound {
        format!(
            "kiwi-memory MCP server `{binary}` not found on PATH. Install with \
             `cargo install --path plugins/kiwi_mcp_memory` or set KIWI_MCP_MEMORY_BIN."
        )
    } else {
        format!("Failed to spawn kiwi-memory MCP server `{binary}`: {err}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_memory_tool_missing_binary_returns_error() {
        let err = call_memory_tool(
            "kiwi-missing-memory-mcp-binary-for-test",
            "search_project_memory",
            json!({ "query": "layout engine", "limit": 3 }),
        )
        .expect_err("missing binary");
        assert!(err.contains("not found on PATH") || err.contains("Failed to spawn"));
    }
}
