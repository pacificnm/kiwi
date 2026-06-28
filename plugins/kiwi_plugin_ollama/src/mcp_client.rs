use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct McpProcess {
    _child: Child,
    writer: BufWriter<ChildStdin>,
    reader: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpProcess {
    /// Spawn a binary as an MCP server and perform the initialize handshake.
    /// Passes DATABASE_URL, EMBED_BACKEND, OLLAMA_URL, and OLLAMA_EMBED_MODEL
    /// from the current process environment.
    pub fn spawn(binary: &str) -> Result<Self> {
        let mut cmd = Command::new(binary);
        for var in &[
            "DATABASE_URL",
            "EMBED_BACKEND",
            "OLLAMA_URL",
            "OLLAMA_EMBED_MODEL",
            "OPENAI_API_KEY",
            "OPENAI_EMBED_MODEL",
            "GITHUB_TOKEN",
            "GITHUB_REPO",
            "GITHUB_API_URL",
            "GIT_REPO_PATH",
        ] {
            if let Ok(val) = std::env::var(var) {
                cmd.env(var, val);
            }
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("failed to spawn MCP server '{binary}'"))?;

        let writer = BufWriter::new(child.stdin.take().context("no stdin")?);
        let reader = BufReader::new(child.stdout.take().context("no stdout")?);

        let mut proc = Self { _child: child, writer, reader, next_id: 1 };
        proc.initialize()?;
        Ok(proc)
    }

    fn initialize(&mut self) -> Result<()> {
        self.send(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "kiwi-ollama", "version": "0.1.0" }
            }),
        )?;
        // Notification: no id, no response expected
        let notif = serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }))?;
        writeln!(self.writer, "{notif}").context("mcp write error")?;
        self.writer.flush().context("mcp flush error")?;
        Ok(())
    }

    /// Fetch the server's tool list and convert to Ollama tool format.
    pub fn list_tools(&mut self) -> Result<Vec<crate::ollama::OllamaTool>> {
        let resp = self.send("tools/list", serde_json::json!({}))?;
        let tools = resp
            .pointer("/result/tools")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(tools
            .into_iter()
            .filter_map(|t| {
                let name = t.get("name")?.as_str()?.to_string();
                let description = t
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                let parameters = t
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({"type":"object","properties":{}}));
                Some(crate::ollama::OllamaTool {
                    kind: "function".to_string(),
                    function: crate::ollama::OllamaToolFunction {
                        name,
                        description,
                        parameters,
                    },
                })
            })
            .collect())
    }

    /// Call an MCP tool and return the text content of the first content block.
    pub fn call_tool(&mut self, name: &str, args: Value) -> Result<String> {
        let result = self.send(
            "tools/call",
            json!({ "name": name, "arguments": args }),
        )?;

        if let Some(err) = result.get("error") {
            bail!("MCP error: {}", err);
        }

        let text = result
            .pointer("/result/content/0/text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(text)
    }

    fn send(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let req = serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))?;

        writeln!(self.writer, "{req}").context("mcp write error")?;
        self.writer.flush().context("mcp flush error")?;

        let mut line = String::new();
        self.reader.read_line(&mut line).context("mcp read error")?;

        let resp: Value = serde_json::from_str(line.trim())
            .with_context(|| format!("mcp parse error: {line}"))?;

        Ok(resp)
    }
}
