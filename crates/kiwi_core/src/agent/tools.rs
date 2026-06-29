//! Tool definitions for the native-chat agent: registry schemas sent to Claude and parser for tool_use blocks.

use serde::Serialize;
use serde_json::{json, Value};

/// Provider-agnostic tool definition in the dotted namespace (`file.read`, `git.diff`, …).
#[derive(Debug, Clone)]
pub struct KiwiToolDef {
    pub id: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

/// Central registry of all compiled-in agent tools.
pub struct ToolRegistry;

/// A locally-executable tool that Claude can invoke via a `tool_use` block.
#[derive(Debug, Clone)]
pub enum KiwiTool {
    FileRead { path: String },
    FileWrite { path: String, content: String },
    FileList { path: String, depth: u8 },
    ShellRun { command: String },
    GitStatus,
    GitDiff { path: Option<String> },
    FileSearch { query: String },
    FileGrep { query: String, path: Option<String> },
}

/// JSON schema descriptor for a single tool — Claude Messages API wire format.
#[derive(Debug, Clone, Serialize)]
pub struct ToolSchema {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

/// Error returned when a `tool_use` block cannot be parsed into a `KiwiTool`.
#[derive(Debug)]
pub struct ToolParseError(pub String);

impl std::fmt::Display for ToolParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&KiwiToolDef> for ToolSchema {
    fn from(def: &KiwiToolDef) -> Self {
        Self {
            name: def.id,
            description: def.description,
            input_schema: def.input_schema.clone(),
        }
    }
}

static TOOLS: std::sync::OnceLock<Vec<KiwiToolDef>> = std::sync::OnceLock::new();

fn init_tools() -> Vec<KiwiToolDef> {
    vec![
        KiwiToolDef {
            id: "file.read",
            description: "Read the full contents of a file in the repository.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path relative to the repository root."}
                },
                "required": ["path"]
            }),
        },
        KiwiToolDef {
            id: "file.write",
            description: "Write (create or overwrite) a file in the repository.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path relative to the repository root."},
                    "content": {"type": "string", "description": "Full file content to write."}
                },
                "required": ["path", "content"]
            }),
        },
        KiwiToolDef {
            id: "file.list",
            description: "List files and directories under a path (up to a given depth).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Directory path relative to the repository root."},
                    "depth": {"type": "integer", "description": "Max recursion depth (default 2, max 5)."}
                },
                "required": ["path"]
            }),
        },
        KiwiToolDef {
            id: "shell.run",
            description: "Run a shell command in kiwi's Terminal panel. Output appears in the Terminal tab, not inline in chat.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command to execute."}
                },
                "required": ["command"]
            }),
        },
        KiwiToolDef {
            id: "git.status",
            description: "Show the current git status: staged, modified, and untracked files.",
            input_schema: json!({"type": "object", "properties": {}}),
        },
        KiwiToolDef {
            id: "git.diff",
            description: "Show the unified diff of uncommitted changes.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Limit diff to this file (optional)."}
                }
            }),
        },
        KiwiToolDef {
            id: "file.search",
            description: "Find files whose names contain a given substring.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Filename substring to search for."}
                },
                "required": ["query"]
            }),
        },
        KiwiToolDef {
            id: "file.grep",
            description: "Search for text or a regex pattern within file contents (uses ripgrep when available).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Text or regex pattern."},
                    "path": {"type": "string", "description": "Restrict search to this path (optional)."}
                },
                "required": ["query"]
            }),
        },
    ]
}

impl ToolRegistry {
    /// Every registered tool definition.
    pub fn all() -> &'static [KiwiToolDef] {
        TOOLS.get_or_init(init_tools).as_slice()
    }

    /// Convert registry entries to Claude Messages API tool schemas.
    pub fn claude_schemas() -> Vec<ToolSchema> {
        Self::all().iter().map(ToolSchema::from).collect()
    }
}

impl KiwiTool {
    /// Parse a `tool_use` block from the API into a `KiwiTool`.
    pub fn from_tool_use(name: &str, input: &Value) -> Result<Self, ToolParseError> {
        let str_field = |field: &str| -> Result<String, ToolParseError> {
            input[field]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| ToolParseError(format!("missing required field '{field}'")))
        };

        match name {
            "file.read" => Ok(Self::FileRead { path: str_field("path")? }),
            "file.write" => Ok(Self::FileWrite {
                path: str_field("path")?,
                content: str_field("content")?,
            }),
            "file.list" => Ok(Self::FileList {
                path: str_field("path")?,
                depth: input["depth"].as_u64().unwrap_or(2).min(5) as u8,
            }),
            "shell.run" => Ok(Self::ShellRun { command: str_field("command")? }),
            "git.status" => Ok(Self::GitStatus),
            "git.diff" => Ok(Self::GitDiff {
                path: input["path"].as_str().map(str::to_owned),
            }),
            "file.search" => Ok(Self::FileSearch { query: str_field("query")? }),
            "file.grep" => Ok(Self::FileGrep {
                query: str_field("query")?,
                path: input["path"].as_str().map(str::to_owned),
            }),
            other => Err(ToolParseError(format!("unknown tool '{other}'"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file_read() {
        let tool = KiwiTool::from_tool_use("file.read", &json!({"path": "src/main.rs"})).unwrap();
        assert!(matches!(tool, KiwiTool::FileRead { path } if path == "src/main.rs"));
    }

    #[test]
    fn parse_file_write() {
        let tool = KiwiTool::from_tool_use(
            "file.write",
            &json!({"path": "out.txt", "content": "hello"}),
        )
        .unwrap();
        assert!(matches!(tool, KiwiTool::FileWrite { path, content } if path == "out.txt" && content == "hello"));
    }

    #[test]
    fn parse_file_list_default_depth() {
        let tool = KiwiTool::from_tool_use("file.list", &json!({"path": "src"})).unwrap();
        assert!(matches!(tool, KiwiTool::FileList { depth: 2, .. }));
    }

    #[test]
    fn parse_file_list_depth_capped_at_5() {
        let tool = KiwiTool::from_tool_use("file.list", &json!({"path": ".", "depth": 99})).unwrap();
        assert!(matches!(tool, KiwiTool::FileList { depth: 5, .. }));
    }

    #[test]
    fn parse_shell_run() {
        let tool =
            KiwiTool::from_tool_use("shell.run", &json!({"command": "cargo test"})).unwrap();
        assert!(matches!(tool, KiwiTool::ShellRun { command } if command == "cargo test"));
    }

    #[test]
    fn parse_git_status() {
        let tool = KiwiTool::from_tool_use("git.status", &json!({})).unwrap();
        assert!(matches!(tool, KiwiTool::GitStatus));
    }

    #[test]
    fn parse_git_diff_optional_path() {
        let with_path =
            KiwiTool::from_tool_use("git.diff", &json!({"path": "src/main.rs"})).unwrap();
        assert!(matches!(with_path, KiwiTool::GitDiff { path: Some(_) }));

        let no_path = KiwiTool::from_tool_use("git.diff", &json!({})).unwrap();
        assert!(matches!(no_path, KiwiTool::GitDiff { path: None }));
    }

    #[test]
    fn parse_file_grep_optional_path() {
        let tool = KiwiTool::from_tool_use(
            "file.grep",
            &json!({"query": "fn main", "path": "src"}),
        )
        .unwrap();
        assert!(matches!(tool, KiwiTool::FileGrep { path: Some(_), .. }));
    }

    #[test]
    fn unknown_tool_returns_error() {
        let err = KiwiTool::from_tool_use("delete_everything", &json!({}));
        assert!(err.is_err());
        assert!(err.unwrap_err().0.contains("delete_everything"));
    }

    #[test]
    fn missing_required_field_returns_error() {
        let err = KiwiTool::from_tool_use("file.read", &json!({}));
        assert!(err.is_err());
    }

    #[test]
    fn registry_returns_eight_tools() {
        assert_eq!(ToolRegistry::all().len(), 8);
    }

    #[test]
    fn registry_schemas_are_serialisable() {
        for schema in ToolRegistry::claude_schemas() {
            let v = serde_json::to_value(&schema).unwrap();
            assert!(v["name"].is_string());
            assert!(v["input_schema"].is_object());
        }
    }

    #[test]
    fn registry_ids_use_dotted_namespace() {
        let ids: Vec<_> = ToolRegistry::all().iter().map(|tool| tool.id).collect();
        assert!(ids.iter().all(|id| id.contains('.')));
        assert!(ids.contains(&"file.read"));
        assert!(ids.contains(&"shell.run"));
    }
}
