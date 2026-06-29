//! Tool definitions for the native-chat agent: schemas sent to Claude and parser for tool_use blocks.

use serde::Serialize;
use serde_json::{json, Value};

/// A locally-executable tool that Claude can invoke via a `tool_use` block.
#[derive(Debug, Clone)]
pub enum KiwiTool {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    ListDirectory { path: String, depth: u8 },
    RunBash { command: String },
    GitStatus,
    GitDiff { path: Option<String> },
    SearchFiles { query: String },
    SearchContent { query: String, path: Option<String> },
}

/// JSON schema descriptor for a single tool — sent in the API request `tools` array.
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

impl KiwiTool {
    /// Returns schemas for all supported tools, included in every API request.
    pub fn all_schemas() -> Vec<ToolSchema> {
        vec![
            ToolSchema {
                name: "read_file",
                description: "Read the full contents of a file in the repository.",
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Path relative to the repository root."}
                    },
                    "required": ["path"]
                }),
            },
            ToolSchema {
                name: "write_file",
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
            ToolSchema {
                name: "list_directory",
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
            ToolSchema {
                name: "run_bash",
                description: "Run a shell command in kiwi's Terminal panel. Output appears in the Terminal tab, not inline in chat.",
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {"type": "string", "description": "Shell command to execute."}
                    },
                    "required": ["command"]
                }),
            },
            ToolSchema {
                name: "git_status",
                description: "Show the current git status: staged, modified, and untracked files.",
                input_schema: json!({"type": "object", "properties": {}}),
            },
            ToolSchema {
                name: "git_diff",
                description: "Show the unified diff of uncommitted changes.",
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Limit diff to this file (optional)."}
                    }
                }),
            },
            ToolSchema {
                name: "search_files",
                description: "Find files whose names contain a given substring.",
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Filename substring to search for."}
                    },
                    "required": ["query"]
                }),
            },
            ToolSchema {
                name: "search_content",
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

    /// Parse a `tool_use` block from the API into a `KiwiTool`.
    pub fn from_tool_use(name: &str, input: &Value) -> Result<Self, ToolParseError> {
        let str_field = |field: &str| -> Result<String, ToolParseError> {
            input[field]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| ToolParseError(format!("missing required field '{field}'")))
        };

        match name {
            "read_file" => Ok(Self::ReadFile { path: str_field("path")? }),
            "write_file" => Ok(Self::WriteFile {
                path: str_field("path")?,
                content: str_field("content")?,
            }),
            "list_directory" => Ok(Self::ListDirectory {
                path: str_field("path")?,
                depth: input["depth"].as_u64().unwrap_or(2).min(5) as u8,
            }),
            "run_bash" => Ok(Self::RunBash { command: str_field("command")? }),
            "git_status" => Ok(Self::GitStatus),
            "git_diff" => Ok(Self::GitDiff {
                path: input["path"].as_str().map(str::to_owned),
            }),
            "search_files" => Ok(Self::SearchFiles { query: str_field("query")? }),
            "search_content" => Ok(Self::SearchContent {
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
    fn parse_read_file() {
        let tool = KiwiTool::from_tool_use("read_file", &json!({"path": "src/main.rs"})).unwrap();
        assert!(matches!(tool, KiwiTool::ReadFile { path } if path == "src/main.rs"));
    }

    #[test]
    fn parse_write_file() {
        let tool = KiwiTool::from_tool_use(
            "write_file",
            &json!({"path": "out.txt", "content": "hello"}),
        )
        .unwrap();
        assert!(matches!(tool, KiwiTool::WriteFile { path, content } if path == "out.txt" && content == "hello"));
    }

    #[test]
    fn parse_list_directory_default_depth() {
        let tool =
            KiwiTool::from_tool_use("list_directory", &json!({"path": "src"})).unwrap();
        assert!(matches!(tool, KiwiTool::ListDirectory { depth: 2, .. }));
    }

    #[test]
    fn parse_list_directory_depth_capped_at_5() {
        let tool =
            KiwiTool::from_tool_use("list_directory", &json!({"path": ".", "depth": 99})).unwrap();
        assert!(matches!(tool, KiwiTool::ListDirectory { depth: 5, .. }));
    }

    #[test]
    fn parse_run_bash() {
        let tool =
            KiwiTool::from_tool_use("run_bash", &json!({"command": "cargo test"})).unwrap();
        assert!(matches!(tool, KiwiTool::RunBash { command } if command == "cargo test"));
    }

    #[test]
    fn parse_git_status() {
        let tool = KiwiTool::from_tool_use("git_status", &json!({})).unwrap();
        assert!(matches!(tool, KiwiTool::GitStatus));
    }

    #[test]
    fn parse_git_diff_optional_path() {
        let with_path =
            KiwiTool::from_tool_use("git_diff", &json!({"path": "src/main.rs"})).unwrap();
        assert!(matches!(with_path, KiwiTool::GitDiff { path: Some(_) }));

        let no_path = KiwiTool::from_tool_use("git_diff", &json!({})).unwrap();
        assert!(matches!(no_path, KiwiTool::GitDiff { path: None }));
    }

    #[test]
    fn parse_search_content_optional_path() {
        let tool = KiwiTool::from_tool_use(
            "search_content",
            &json!({"query": "fn main", "path": "src"}),
        )
        .unwrap();
        assert!(matches!(tool, KiwiTool::SearchContent { path: Some(_), .. }));
    }

    #[test]
    fn unknown_tool_returns_error() {
        let err = KiwiTool::from_tool_use("delete_everything", &json!({}));
        assert!(err.is_err());
        assert!(err.unwrap_err().0.contains("delete_everything"));
    }

    #[test]
    fn missing_required_field_returns_error() {
        let err = KiwiTool::from_tool_use("read_file", &json!({}));
        assert!(err.is_err());
    }

    #[test]
    fn all_schemas_returns_eight_tools() {
        assert_eq!(KiwiTool::all_schemas().len(), 8);
    }

    #[test]
    fn all_schemas_are_serialisable() {
        for schema in KiwiTool::all_schemas() {
            let v = serde_json::to_value(&schema).unwrap();
            assert!(v["name"].is_string());
            assert!(v["input_schema"].is_object());
        }
    }
}
