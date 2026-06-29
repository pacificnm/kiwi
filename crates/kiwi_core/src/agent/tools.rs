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

/// Action for [`KiwiTool::GitBranch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitBranchAction {
    List,
    Create,
    Checkout,
}

/// A locally-executable tool that Claude can invoke via a `tool_use` block.
#[derive(Debug, Clone)]
pub enum KiwiTool {
    FileRead { path: String },
    FileWrite { path: String, content: String },
    FileList { path: String, depth: u8 },
    ShellRun { command: String },
    GitStatus,
    GitDiff { path: Option<String> },
    GitCommit {
        message: String,
        stage_all: bool,
    },
    GitBranch {
        action: GitBranchAction,
        name: Option<String>,
    },
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
            id: "git.commit",
            description: "Stage changes (optional) and create a git commit with the given message.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string", "description": "Commit message."},
                    "stage_all": {"type": "boolean", "description": "If true, run git add -A before committing (default true)."}
                },
                "required": ["message"]
            }),
        },
        KiwiToolDef {
            id: "git.branch",
            description: "List local branches, create a new branch, or checkout an existing branch.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list", "create", "checkout"],
                        "description": "Action to perform (default: list)."
                    },
                    "name": {"type": "string", "description": "Branch name (required for create/checkout)."}
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

/// Named subset of tool IDs exposed to a model for a given agent type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolProfile {
    pub name: &'static str,
    pub allowed: &'static [&'static str],
}

const TOOL_PROFILES: &[ToolProfile] = &[
    ToolProfile {
        name: "all",
        allowed: &[],
    },
    ToolProfile {
        name: "coding",
        allowed: &[
            "file.read",
            "file.write",
            "file.list",
            "file.search",
            "file.grep",
            "shell.run",
            "git.status",
            "git.diff",
            "git.branch",
            "git.commit",
            "cargo.check",
            "cargo.test",
        ],
    },
    ToolProfile {
        name: "code_review",
        allowed: &[
            "file.read",
            "file.search",
            "file.grep",
            "git.diff",
            "cargo.check",
        ],
    },
    ToolProfile {
        name: "github",
        allowed: &[
            "github.issues",
            "github.prs",
            "git.branch",
            "git.commit",
            "git.status",
        ],
    },
    ToolProfile {
        name: "planner",
        allowed: &[
            "project.context",
            "memory.search",
            "file.search",
            "file.grep",
        ],
    },
];

/// Look up a pre-defined profile by name.
pub fn tool_profile_by_name(name: &str) -> Option<&'static ToolProfile> {
    TOOL_PROFILES.iter().find(|profile| profile.name == name)
}

/// Provider-level profile wins over the agent default when set.
pub fn resolve_tool_profile<'a>(
    agent_profile: &'a str,
    provider_profile: Option<&'a str>,
) -> &'a str {
    provider_profile.unwrap_or(agent_profile)
}

/// OpenAI Chat Completions tool definition (`type: "function"`).
#[derive(Debug, Clone, Serialize)]
pub struct OpenAiToolSchema {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub function: OpenAiFunctionSchema,
}

/// Function payload inside an OpenAI tool definition.
#[derive(Debug, Clone, Serialize)]
pub struct OpenAiFunctionSchema {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

/// Convert registry entries to Claude Messages API tool schemas.
pub fn tools_for_claude(tools: &[KiwiToolDef]) -> Vec<ToolSchema> {
    tools.iter().map(ToolSchema::from).collect()
}

/// Convert registry entries to OpenAI Chat Completions tool schemas.
pub fn tools_for_openai(tools: &[KiwiToolDef]) -> Vec<OpenAiToolSchema> {
    tools
        .iter()
        .map(|tool| OpenAiToolSchema {
            kind: "function",
            function: OpenAiFunctionSchema {
                name: tool.id,
                description: tool.description,
                parameters: tool.input_schema.clone(),
            },
        })
        .collect()
}

/// Return true when the Ollama model is known to support OpenAI-style tool calling.
pub fn ollama_supports_tools(model: &str) -> bool {
    let base = model
        .split(':')
        .next()
        .unwrap_or(model)
        .to_ascii_lowercase();
    base.starts_with("qwen2.5-coder")
        || base.starts_with("llama3")
        || base.starts_with("mistral")
        || base.starts_with("mixtral")
}

impl ToolRegistry {
    /// Every registered tool definition.
    pub fn all() -> &'static [KiwiToolDef] {
        TOOLS.get_or_init(init_tools).as_slice()
    }

    /// Registry entries allowed by `profile_name` (unknown names fall back to `all`).
    pub fn for_profile(profile_name: &str) -> Vec<KiwiToolDef> {
        let profile = tool_profile_by_name(profile_name)
            .unwrap_or_else(|| tool_profile_by_name("all").expect("all profile must exist"));
        if profile.name == "all" {
            return Self::all().to_vec();
        }
        Self::all()
            .iter()
            .filter(|tool| profile.allowed.contains(&tool.id))
            .cloned()
            .collect()
    }

    /// Convert registry entries to Claude Messages API tool schemas.
    pub fn claude_schemas() -> Vec<ToolSchema> {
        tools_for_claude(Self::all())
    }

    /// Convert registry entries to OpenAI Chat Completions tool schemas.
    pub fn openai_schemas() -> Vec<OpenAiToolSchema> {
        tools_for_openai(Self::all())
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
            "file.read" => Ok(Self::FileRead {
                path: str_field("path")?,
            }),
            "file.write" => Ok(Self::FileWrite {
                path: str_field("path")?,
                content: str_field("content")?,
            }),
            "file.list" => Ok(Self::FileList {
                path: str_field("path")?,
                depth: input["depth"].as_u64().unwrap_or(2).min(5) as u8,
            }),
            "shell.run" => Ok(Self::ShellRun {
                command: str_field("command")?,
            }),
            "git.status" => Ok(Self::GitStatus),
            "git.diff" => Ok(Self::GitDiff {
                path: input["path"].as_str().map(str::to_owned),
            }),
            "git.commit" => Ok(Self::GitCommit {
                message: str_field("message")?,
                stage_all: input["stage_all"].as_bool().unwrap_or(true),
            }),
            "git.branch" => {
                let action = parse_git_branch_action(input)?;
                let name = input["name"].as_str().map(str::to_owned);
                match action {
                    GitBranchAction::List => Ok(Self::GitBranch { action, name: None }),
                    GitBranchAction::Create | GitBranchAction::Checkout => {
                        let name = name.ok_or_else(|| {
                            ToolParseError("missing required field 'name'".to_string())
                        })?;
                        if name.trim().is_empty() {
                            return Err(ToolParseError(
                                "branch name must not be empty".to_string(),
                            ));
                        }
                        Ok(Self::GitBranch {
                            action,
                            name: Some(name),
                        })
                    }
                }
            }
            "file.search" => Ok(Self::FileSearch {
                query: str_field("query")?,
            }),
            "file.grep" => Ok(Self::FileGrep {
                query: str_field("query")?,
                path: input["path"].as_str().map(str::to_owned),
            }),
            other => Err(ToolParseError(format!("unknown tool '{other}'"))),
        }
    }
}

fn parse_git_branch_action(input: &Value) -> Result<GitBranchAction, ToolParseError> {
    match input["action"].as_str().unwrap_or("list") {
        "list" => Ok(GitBranchAction::List),
        "create" => Ok(GitBranchAction::Create),
        "checkout" => Ok(GitBranchAction::Checkout),
        other => Err(ToolParseError(format!(
            "invalid git.branch action '{other}'"
        ))),
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
        assert!(
            matches!(tool, KiwiTool::FileWrite { path, content } if path == "out.txt" && content == "hello")
        );
    }

    #[test]
    fn parse_file_list_default_depth() {
        let tool = KiwiTool::from_tool_use("file.list", &json!({"path": "src"})).unwrap();
        assert!(matches!(tool, KiwiTool::FileList { depth: 2, .. }));
    }

    #[test]
    fn parse_file_list_depth_capped_at_5() {
        let tool =
            KiwiTool::from_tool_use("file.list", &json!({"path": ".", "depth": 99})).unwrap();
        assert!(matches!(tool, KiwiTool::FileList { depth: 5, .. }));
    }

    #[test]
    fn parse_shell_run() {
        let tool = KiwiTool::from_tool_use("shell.run", &json!({"command": "cargo test"})).unwrap();
        assert!(matches!(tool, KiwiTool::ShellRun { command } if command == "cargo test"));
    }

    #[test]
    fn parse_git_status() {
        let tool = KiwiTool::from_tool_use("git.status", &json!({})).unwrap();
        assert!(matches!(tool, KiwiTool::GitStatus));
    }

    #[test]
    fn parse_git_branch_defaults_to_list() {
        let tool = KiwiTool::from_tool_use("git.branch", &json!({})).unwrap();
        assert!(matches!(
            tool,
            KiwiTool::GitBranch {
                action: GitBranchAction::List,
                name: None
            }
        ));
    }

    #[test]
    fn parse_git_branch_create_requires_name() {
        let err = KiwiTool::from_tool_use("git.branch", &json!({"action": "create"}));
        assert!(err.is_err());
        let tool = KiwiTool::from_tool_use(
            "git.branch",
            &json!({"action": "create", "name": "feature-x"}),
        )
        .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::GitBranch {
                action: GitBranchAction::Create,
                name: Some(name)
            } if name == "feature-x"
        ));
    }

    #[test]
    fn parse_git_branch_checkout() {
        let tool = KiwiTool::from_tool_use(
            "git.branch",
            &json!({"action": "checkout", "name": "main"}),
        )
        .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::GitBranch {
                action: GitBranchAction::Checkout,
                ..
            }
        ));
    }

    #[test]
    fn parse_git_commit_defaults_stage_all() {
        let tool = KiwiTool::from_tool_use("git.commit", &json!({"message": "fix bug"})).unwrap();
        assert!(
            matches!(tool, KiwiTool::GitCommit { message, stage_all: true } if message == "fix bug")
        );
    }

    #[test]
    fn parse_git_commit_stage_all_false() {
        let tool = KiwiTool::from_tool_use(
            "git.commit",
            &json!({"message": "wip", "stage_all": false}),
        )
        .unwrap();
        assert!(matches!(tool, KiwiTool::GitCommit { stage_all: false, .. }));
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
        let tool =
            KiwiTool::from_tool_use("file.grep", &json!({"query": "fn main", "path": "src"}))
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
    fn registry_returns_ten_tools() {
        assert_eq!(ToolRegistry::all().len(), 10);
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
    fn coding_profile_includes_registered_file_and_git_tools() {
        let tools = ToolRegistry::for_profile("coding");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert!(ids.contains(&"file.read"));
        assert!(ids.contains(&"shell.run"));
        assert!(ids.contains(&"git.status"));
        assert!(ids.contains(&"git.commit"));
        assert!(ids.contains(&"git.branch"));
    }

    #[test]
    fn github_profile_includes_registered_git_tools() {
        let tools = ToolRegistry::for_profile("github");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert_eq!(ids, vec!["git.status", "git.commit", "git.branch"]);
    }

    #[test]
    fn unknown_profile_falls_back_to_all() {
        assert_eq!(
            ToolRegistry::for_profile("nonexistent").len(),
            ToolRegistry::all().len()
        );
    }

    #[test]
    fn resolve_tool_profile_prefers_provider_override() {
        assert_eq!(resolve_tool_profile("coding", Some("planner")), "planner");
        assert_eq!(resolve_tool_profile("coding", None), "coding");
    }

    #[test]
    fn openai_adapter_uses_function_type() {
        let schemas = tools_for_openai(ToolRegistry::all());
        assert_eq!(schemas.len(), 10);
        let first = serde_json::to_value(&schemas[0]).unwrap();
        assert_eq!(first["type"], "function");
        assert_eq!(first["function"]["name"], "file.read");
        assert!(first["function"]["parameters"].is_object());
    }

    #[test]
    fn ollama_supports_known_tool_models() {
        assert!(ollama_supports_tools("qwen2.5-coder:7b"));
        assert!(ollama_supports_tools("llama3.1:8b"));
        assert!(ollama_supports_tools("mistral:latest"));
        assert!(!ollama_supports_tools("nomic-embed-text"));
    }

    #[test]
    fn registry_ids_use_dotted_namespace() {
        let ids: Vec<_> = ToolRegistry::all().iter().map(|tool| tool.id).collect();
        assert!(ids.iter().all(|id| id.contains('.')));
        assert!(ids.contains(&"file.read"));
        assert!(ids.contains(&"shell.run"));
    }
}
