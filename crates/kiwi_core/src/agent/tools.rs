//! Tool definitions for the native-chat agent: registry schemas sent to Claude and parser for tool_use blocks.

use serde::Serialize;
use serde_json::{json, Map, Value};

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
    FileRead {
        path: String,
    },
    FileWrite {
        path: String,
        content: String,
    },
    FilePatch {
        path: String,
        old_str: String,
        new_str: String,
    },
    FileReadRange {
        path: String,
        start_line: u32,
        end_line: Option<u32>,
    },
    FileDelete {
        path: String,
    },
    FileMove {
        src: String,
        dest: String,
    },
    FileList {
        path: String,
        depth: u8,
    },
    ShellRun {
        command: String,
    },
    ShellCapture {
        command: String,
        timeout_secs: u32,
    },
    GitStatus,
    GitDiff {
        path: Option<String>,
    },
    GitCommit {
        message: String,
        stage_all: bool,
    },
    GitBranch {
        action: GitBranchAction,
        name: Option<String>,
    },
    CargoCheck {
        package: Option<String>,
    },
    CargoBuild {
        package: Option<String>,
        release: bool,
    },
    CargoTest {
        filter: Option<String>,
        package: Option<String>,
    },
    GitHubIssues {
        limit: u32,
        label: Option<String>,
        milestone: Option<String>,
    },
    GitHubPrs {
        limit: u32,
        base: Option<String>,
    },
    MemorySearch {
        query: String,
        limit: u32,
    },
    ProjectContext,
    FileSearch {
        query: String,
    },
    FileGrep {
        query: String,
        path: Option<String>,
    },
}

/// JSON schema descriptor for a single tool — Claude Messages API wire format.
#[derive(Debug, Clone, Serialize)]
pub struct ToolSchema {
    pub name: String,
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
            name: openai_tool_name(def.id),
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
            id: "file.patch",
            description: "Surgical edit: replace a unique old_str with new_str in a file.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path relative to repo root."},
                    "old_str": {"type": "string", "description": "Exact string to find (must be unique in the file)."},
                    "new_str": {"type": "string", "description": "Replacement string."}
                },
                "required": ["path", "old_str", "new_str"]
            }),
        },
        KiwiToolDef {
            id: "file.read_range",
            description: "Read a specific line range from a file with line numbers.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path relative to repo root."},
                    "start_line": {"type": "integer", "description": "First line to return (1-indexed)."},
                    "end_line": {"type": "integer", "description": "Last line to return (inclusive). Omit to read to end of file."}
                },
                "required": ["path", "start_line"]
            }),
        },
        KiwiToolDef {
            id: "file.delete",
            description: "Delete a file from the repository.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path relative to repo root."}
                },
                "required": ["path"]
            }),
        },
        KiwiToolDef {
            id: "file.move",
            description: "Rename or move a file within the repository.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "src": {"type": "string", "description": "Current file path relative to repo root."},
                    "dest": {"type": "string", "description": "Destination path relative to repo root."}
                },
                "required": ["src", "dest"]
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
            id: "shell.capture",
            description: "Run a shell command and return captured stdout/stderr to the agent.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command to run (executed via sh -c)."},
                    "timeout_secs": {"type": "integer", "description": "Max seconds to wait (default 30, max 120)."}
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
            id: "cargo.check",
            description: "Run cargo check in the repository root and return compiler output.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "package": {
                        "type": "string",
                        "description": "Limit check to a specific package (optional, runs workspace check by default)."
                    }
                }
            }),
        },
        KiwiToolDef {
            id: "cargo.build",
            description: "Run cargo build in the repository root and return build output.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "package": {"type": "string", "description": "Limit to a specific package (optional)."},
                    "release": {"type": "boolean", "description": "Build in release mode (default false)."}
                }
            }),
        },
        KiwiToolDef {
            id: "cargo.test",
            description: "Run cargo test in the repository root and return test output.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "string",
                        "description": "Test name substring filter (optional)."
                    },
                    "package": {
                        "type": "string",
                        "description": "Limit to a specific package (optional)."
                    }
                }
            }),
        },
        KiwiToolDef {
            id: "github.issues",
            description: "List open GitHub issues for the current repository using the gh CLI.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Max issues to return (default 20, max 100)."
                    },
                    "label": {
                        "type": "string",
                        "description": "Filter by label (optional)."
                    },
                    "milestone": {
                        "type": "string",
                        "description": "Filter by milestone title (optional)."
                    }
                }
            }),
        },
        KiwiToolDef {
            id: "github.prs",
            description: "List open pull requests for the current repository using the gh CLI.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Max PRs to return (default 20, max 100)."
                    },
                    "base": {
                        "type": "string",
                        "description": "Filter by base branch (optional)."
                    }
                }
            }),
        },
        KiwiToolDef {
            id: "memory.search",
            description: "Search indexed project documentation (SPECs, ADRs, architecture notes) via semantic similarity.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language search query."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results to return (default 5)."
                    }
                },
                "required": ["query"]
            }),
        },
        KiwiToolDef {
            id: "project.context",
            description: "Return a structured overview of the repository: branch, recent commits, directory layout, and key config files.",
            input_schema: json!({
                "type": "object",
                "properties": {}
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

impl ToolProfile {
    /// Every pre-defined tool permission profile.
    pub fn all_profiles() -> &'static [ToolProfile] {
        TOOL_PROFILES
    }
}

/// Tools always exposed to the agent regardless of the active profile.
const MANDATORY_TOOL_IDS: &[&str] = &["memory.search"];

const TOOL_PROFILES: &[ToolProfile] = &[
    ToolProfile {
        name: "all",
        allowed: &[],
    },
    ToolProfile {
        name: "coding",
        allowed: &[
            "file.read",
            "file.read_range",
            "file.write",
            "file.patch",
            "file.list",
            "file.search",
            "file.grep",
            "file.delete",
            "file.move",
            "shell.run",
            "shell.capture",
            "git.status",
            "git.diff",
            "git.commit",
            "cargo.check",
            "cargo.build",
            "cargo.test",
        ],
    },
    ToolProfile {
        name: "code_review",
        allowed: &[
            "file.read",
            "file.read_range",
            "file.list",
            "file.search",
            "file.grep",
            "git.diff",
            "cargo.check",
        ],
    },
    ToolProfile {
        name: "github",
        allowed: &[
            "git.status",
            "git.branch",
            "git.commit",
            "github.issues",
            "github.prs",
        ],
    },
    ToolProfile {
        name: "planner",
        allowed: &[
            "project.context",
            "memory.search",
            "file.search",
            "file.grep",
            "file.read",
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
    pub name: String,
    pub description: &'static str,
    pub parameters: Value,
}

/// Convert a registry dotted id (`file.read`) to a provider wire name (`file_read`).
///
/// Claude and OpenAI require `^[a-zA-Z0-9_-]+$` for tool/function names — dots are rejected.
pub fn openai_tool_name(kiwi_id: &str) -> String {
    kiwi_id.replace('.', "_")
}

/// Map an OpenAI/Ollama wire tool name back to the registry dotted id.
pub fn kiwi_tool_id_from_openai(wire_name: &str) -> Option<&'static str> {
    ToolRegistry::all()
        .iter()
        .find(|tool| openai_tool_name(tool.id) == wire_name || tool.id == wire_name)
        .map(|tool| tool.id)
}

/// Tool call parsed from Ollama `message.content` when models emit JSON instead of `tool_calls`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaContentToolCall {
    pub wire_name: String,
    pub arguments: Value,
}

/// Parse tool calls embedded in assistant text (common with `qwen2.5-coder` on Ollama).
pub fn parse_ollama_content_tool_calls(content: &str) -> Vec<OllamaContentToolCall> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if let Some(call) = value_to_ollama_content_tool_call(&value) {
            return vec![call];
        }
        if let Some(items) = value.as_array() {
            return items
                .iter()
                .filter_map(value_to_ollama_content_tool_call)
                .collect();
        }
    }

    extract_json_objects(trimmed)
        .into_iter()
        .filter_map(|value| value_to_ollama_content_tool_call(&value))
        .collect()
}

fn value_to_ollama_content_tool_call(value: &Value) -> Option<OllamaContentToolCall> {
    let wire_name = value.get("name")?.as_str()?.trim();
    if wire_name.is_empty() {
        return None;
    }
    let arguments = value
        .get("arguments")
        .or_else(|| value.get("parameters"))
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    Some(OllamaContentToolCall {
        wire_name: wire_name.to_string(),
        arguments,
    })
}

fn extract_json_objects(text: &str) -> Vec<Value> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'{' {
            index += 1;
            continue;
        }
        let mut depth = 0usize;
        let start = index;
        for (offset, byte) in bytes[index..].iter().enumerate() {
            match byte {
                b'{' => depth += 1,
                b'}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let end = index + offset + 1;
                        if let Ok(value) = serde_json::from_slice(&bytes[start..end]) {
                            out.push(value);
                        }
                        index = end;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            break;
        }
    }
    out
}

/// True when assistant text is only a tool-call JSON blob (not user-facing prose).
pub fn streaming_text_is_ollama_tool_json(text: &str) -> bool {
    !parse_ollama_content_tool_calls(text).is_empty()
}

/// Strip null entries from tool argument objects.
///
/// Ollama and some OpenAI models emit `"package": null` for optional fields; treat those
/// as absent so parsers see the same shape as `{}`.
pub fn normalize_tool_arguments(input: Value) -> Value {
    match input {
        Value::Object(map) => {
            let cleaned: Map<String, Value> = map
                .into_iter()
                .filter(|(_, value)| {
                    !value.is_null()
                        && !(value.is_string() && value.as_str().unwrap_or("").trim().is_empty())
                })
                .map(|(key, value)| (key, normalize_tool_arguments(value)))
                .collect();
            Value::Object(cleaned)
        }
        Value::Array(items) => {
            Value::Array(items.into_iter().map(normalize_tool_arguments).collect())
        }
        other => other,
    }
}

/// Parse tool argument JSON, drop null optional fields, and re-serialize.
pub fn normalize_tool_arguments_json(raw: &str) -> String {
    let Ok(value) = serde_json::from_str(raw) else {
        return raw.to_string();
    };
    serde_json::to_string(&normalize_tool_arguments(value)).unwrap_or_else(|_| raw.to_string())
}

fn optional_str_field(input: &Value, field: &str) -> Option<String> {
    match input.get(field) {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) if text.trim().is_empty() => None,
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
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
                name: openai_tool_name(tool.id),
                description: tool.description,
                parameters: tool.input_schema.clone(),
            },
        })
        .collect()
}

/// Maximum tool-call round trips per user message (prevents runaway Ollama loops).
pub const MAX_TOOL_ROUNDS_PER_TURN: u8 = 8;

/// Tools exposed to Ollama. Excludes `shell.run` — output goes to the Terminal panel,
/// which Ollama chat models cannot read, causing retry loops.
pub fn tools_for_ollama(profile_name: &str) -> Vec<KiwiToolDef> {
    ToolRegistry::for_profile(profile_name)
        .into_iter()
        .filter(|tool| tool.id != "shell.run")
        .collect()
}

/// Ollama models that return structured `tool_calls` (not JSON blobs in content).
pub fn ollama_uses_native_tool_calls(model: &str) -> bool {
    let base = model
        .split(':')
        .next()
        .unwrap_or(model)
        .to_ascii_lowercase();
    base.starts_with("llama3")
        || base.starts_with("mistral")
        || base.starts_with("mixtral")
        || base.starts_with("gpt-oss")
}

/// True when split `tool_model` / `code_model` fields are configured.
pub fn ollama_split_models(settings: &crate::config::ProviderSettings) -> bool {
    settings.tool_model.is_some() || settings.code_model.is_some()
}

/// Return true when the Ollama model is known to support OpenAI-style tool calling.
pub fn ollama_supports_tools(model: &str) -> bool {
    ollama_uses_native_tool_calls(model)
        || model
            .split(':')
            .next()
            .unwrap_or(model)
            .to_ascii_lowercase()
            .starts_with("qwen2.5-coder")
}

impl ToolRegistry {
    /// Every registered tool definition.
    pub fn all() -> &'static [KiwiToolDef] {
        TOOLS.get_or_init(init_tools).as_slice()
    }

    /// Registry entries allowed by `profile_name`, plus mandatory tools (unknown names fall back to `all`).
    pub fn for_profile(profile_name: &str) -> Vec<KiwiToolDef> {
        let profile = tool_profile_by_name(profile_name)
            .unwrap_or_else(|| tool_profile_by_name("all").expect("all profile must exist"));
        if profile.name == "all" {
            return Self::all().to_vec();
        }
        Self::all()
            .iter()
            .filter(|tool| {
                profile.allowed.contains(&tool.id) || MANDATORY_TOOL_IDS.contains(&tool.id)
            })
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
            "file.patch" => Ok(Self::FilePatch {
                path: str_field("path")?,
                old_str: str_field("old_str")?,
                new_str: str_field("new_str")?,
            }),
            "file.read_range" => Ok(Self::FileReadRange {
                path: str_field("path")?,
                start_line: parse_positive_line_number(input, "start_line")?,
                end_line: optional_line_number(input, "end_line"),
            }),
            "file.delete" => Ok(Self::FileDelete {
                path: str_field("path")?,
            }),
            "file.move" => Ok(Self::FileMove {
                src: str_field("src")?,
                dest: str_field("dest")?,
            }),
            "file.list" => Ok(Self::FileList {
                path: str_field("path")?,
                depth: input["depth"].as_u64().unwrap_or(2).min(5) as u8,
            }),
            "shell.run" => Ok(Self::ShellRun {
                command: str_field("command")?,
            }),
            "shell.capture" => Ok(Self::ShellCapture {
                command: str_field("command")?,
                timeout_secs: parse_shell_capture_timeout(input),
            }),
            "git.status" => Ok(Self::GitStatus),
            "git.diff" => Ok(Self::GitDiff {
                path: optional_str_field(input, "path"),
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
            "cargo.check" => Ok(Self::CargoCheck {
                package: optional_str_field(input, "package"),
            }),
            "cargo.build" => Ok(Self::CargoBuild {
                package: optional_str_field(input, "package"),
                release: input["release"].as_bool().unwrap_or(false),
            }),
            "cargo.test" => Ok(Self::CargoTest {
                filter: optional_str_field(input, "filter"),
                package: optional_str_field(input, "package"),
            }),
            "github.issues" => Ok(Self::GitHubIssues {
                limit: parse_github_issues_limit(input),
                label: optional_str_field(input, "label"),
                milestone: optional_str_field(input, "milestone"),
            }),
            "github.prs" => Ok(Self::GitHubPrs {
                limit: parse_github_prs_limit(input),
                base: optional_str_field(input, "base"),
            }),
            "memory.search" => Ok(Self::MemorySearch {
                query: str_field("query")?,
                limit: parse_memory_search_limit(input),
            }),
            "project.context" => Ok(Self::ProjectContext),
            "file.search" => Ok(Self::FileSearch {
                query: str_field("query")?,
            }),
            "file.grep" => Ok(Self::FileGrep {
                query: str_field("query")?,
                path: optional_str_field(input, "path"),
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

fn parse_github_issues_limit(input: &Value) -> u32 {
    input["limit"].as_u64().unwrap_or(20).clamp(1, 100) as u32
}

fn parse_github_prs_limit(input: &Value) -> u32 {
    parse_github_issues_limit(input)
}

fn parse_memory_search_limit(input: &Value) -> u32 {
    input["limit"].as_u64().unwrap_or(5).clamp(1, 20) as u32
}

fn parse_positive_line_number(input: &Value, field: &str) -> Result<u32, ToolParseError> {
    let line = input[field]
        .as_u64()
        .ok_or_else(|| ToolParseError(format!("missing required field '{field}'")))?;
    if line == 0 {
        return Err(ToolParseError(format!("'{field}' must be >= 1")));
    }
    Ok(line as u32)
}

fn optional_line_number(input: &Value, field: &str) -> Option<u32> {
    input[field]
        .as_u64()
        .and_then(|line| if line == 0 { None } else { Some(line as u32) })
}

fn parse_shell_capture_timeout(input: &Value) -> u32 {
    input["timeout_secs"].as_u64().unwrap_or(30).clamp(1, 120) as u32
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
        let tool =
            KiwiTool::from_tool_use("git.branch", &json!({"action": "checkout", "name": "main"}))
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
        let tool =
            KiwiTool::from_tool_use("git.commit", &json!({"message": "wip", "stage_all": false}))
                .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::GitCommit {
                stage_all: false,
                ..
            }
        ));
    }

    #[test]
    fn parse_cargo_test_optional_filter_and_package() {
        let tool = KiwiTool::from_tool_use("cargo.test", &json!({})).unwrap();
        assert!(matches!(
            tool,
            KiwiTool::CargoTest {
                filter: None,
                package: None
            }
        ));

        let tool = KiwiTool::from_tool_use(
            "cargo.test",
            &json!({"filter": "integration", "package": "kiwi_core"}),
        )
        .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::CargoTest {
                filter: Some(f),
                package: Some(p)
            } if f == "integration" && p == "kiwi_core"
        ));
    }

    #[test]
    fn parse_github_issues_defaults_and_clamps_limit() {
        let tool = KiwiTool::from_tool_use("github.issues", &json!({})).unwrap();
        assert!(matches!(
            tool,
            KiwiTool::GitHubIssues {
                limit: 20,
                label: None,
                milestone: None
            }
        ));

        let tool = KiwiTool::from_tool_use("github.issues", &json!({"limit": 500})).unwrap();
        assert!(matches!(tool, KiwiTool::GitHubIssues { limit: 100, .. }));

        let tool = KiwiTool::from_tool_use(
            "github.issues",
            &json!({"limit": 5, "label": "bug", "milestone": "M1"}),
        )
        .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::GitHubIssues {
                limit: 5,
                label: Some(l),
                milestone: Some(m)
            } if l == "bug" && m == "M1"
        ));
    }

    #[test]
    fn parse_github_prs_defaults_and_clamps_limit() {
        let tool = KiwiTool::from_tool_use("github.prs", &json!({})).unwrap();
        assert!(matches!(
            tool,
            KiwiTool::GitHubPrs {
                limit: 20,
                base: None
            }
        ));

        let tool = KiwiTool::from_tool_use("github.prs", &json!({"limit": 500})).unwrap();
        assert!(matches!(tool, KiwiTool::GitHubPrs { limit: 100, .. }));

        let tool =
            KiwiTool::from_tool_use("github.prs", &json!({"limit": 10, "base": "main"})).unwrap();
        assert!(matches!(
            tool,
            KiwiTool::GitHubPrs {
                limit: 10,
                base: Some(b)
            } if b == "main"
        ));
    }

    #[test]
    fn parse_memory_search_requires_query_and_clamps_limit() {
        let err = KiwiTool::from_tool_use("memory.search", &json!({}));
        assert!(err.is_err());

        let tool = KiwiTool::from_tool_use(
            "memory.search",
            &json!({"query": "layout engine", "limit": 99}),
        )
        .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::MemorySearch {
                query,
                limit: 20
            } if query == "layout engine"
        ));

        let tool = KiwiTool::from_tool_use("memory.search", &json!({"query": "ADR-003"})).unwrap();
        assert!(matches!(
            tool,
            KiwiTool::MemorySearch {
                query,
                limit: 5
            } if query == "ADR-003"
        ));
    }

    #[test]
    fn parse_cargo_check_optional_package() {
        let tool = KiwiTool::from_tool_use("cargo.check", &json!({})).unwrap();
        assert!(matches!(tool, KiwiTool::CargoCheck { package: None }));

        let tool =
            KiwiTool::from_tool_use("cargo.check", &json!({"package": "kiwi_core"})).unwrap();
        assert!(matches!(tool, KiwiTool::CargoCheck { package: Some(pkg) } if pkg == "kiwi_core"));
    }

    #[test]
    fn parse_cargo_check_treats_null_package_as_absent() {
        let tool = KiwiTool::from_tool_use("cargo.check", &json!({"package": null})).unwrap();
        assert!(matches!(tool, KiwiTool::CargoCheck { package: None }));

        let normalized = normalize_tool_arguments(json!({"package": null}));
        let tool = KiwiTool::from_tool_use("cargo.check", &normalized).unwrap();
        assert!(matches!(tool, KiwiTool::CargoCheck { package: None }));
    }

    #[test]
    fn normalize_tool_arguments_json_strips_nulls() {
        assert_eq!(
            normalize_tool_arguments_json(r#"{"package":null,"other":"x"}"#),
            r#"{"other":"x"}"#
        );
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
    fn registry_returns_twenty_two_tools() {
        assert_eq!(ToolRegistry::all().len(), 22);
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
    fn coding_profile_matches_spec_tool_set() {
        let tools = ToolRegistry::for_profile("coding");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert_eq!(
            ids,
            vec![
                "file.read",
                "file.write",
                "file.patch",
                "file.read_range",
                "file.delete",
                "file.move",
                "file.list",
                "shell.run",
                "shell.capture",
                "git.status",
                "git.diff",
                "git.commit",
                "cargo.check",
                "cargo.build",
                "cargo.test",
                "memory.search",
                "file.search",
                "file.grep",
            ]
        );
    }

    #[test]
    fn all_profiles_lists_every_defined_profile() {
        assert_eq!(ToolProfile::all_profiles().len(), 5);
        let names: Vec<_> = ToolProfile::all_profiles()
            .iter()
            .map(|profile| profile.name)
            .collect();
        assert!(names.contains(&"coding"));
        assert!(names.contains(&"code_review"));
        assert!(names.contains(&"github"));
        assert!(names.contains(&"planner"));
        assert!(names.contains(&"all"));
    }

    #[test]
    fn planner_profile_matches_spec_tool_set() {
        let tools = ToolRegistry::for_profile("planner");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert_eq!(
            ids,
            vec![
                "file.read",
                "memory.search",
                "project.context",
                "file.search",
                "file.grep",
            ]
        );
    }

    #[test]
    fn planner_profile_excludes_write_shell_and_git_tools() {
        let tools = ToolRegistry::for_profile("planner");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert!(!ids.contains(&"file.write"));
        assert!(!ids.contains(&"file.patch"));
        assert!(!ids.contains(&"shell.run"));
        assert!(!ids.contains(&"git.commit"));
        assert!(!ids.contains(&"git.status"));
        assert!(!ids.contains(&"cargo.check"));
    }

    #[test]
    fn github_profile_matches_spec_tool_set() {
        let tools = ToolRegistry::for_profile("github");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert_eq!(
            ids,
            vec![
                "git.status",
                "git.commit",
                "git.branch",
                "github.issues",
                "github.prs",
                "memory.search",
            ]
        );
    }

    #[test]
    fn github_profile_excludes_write_and_shell_tools() {
        let tools = ToolRegistry::for_profile("github");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert!(!ids.contains(&"file.write"));
        assert!(!ids.contains(&"file.patch"));
        assert!(!ids.contains(&"shell.run"));
        assert!(!ids.contains(&"shell.capture"));
        assert!(!ids.contains(&"cargo.check"));
    }

    #[test]
    fn memory_search_is_mandatory_in_github_profile() {
        let tools = ToolRegistry::for_profile("github");
        assert!(tools.iter().any(|tool| tool.id == "memory.search"));
    }

    #[test]
    fn code_review_profile_matches_spec_tool_set() {
        let tools = ToolRegistry::for_profile("code_review");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert_eq!(
            ids,
            vec![
                "file.read",
                "file.read_range",
                "file.list",
                "git.diff",
                "cargo.check",
                "memory.search",
                "file.search",
                "file.grep",
            ]
        );
    }

    #[test]
    fn file_read_range_schema_matches_spec() {
        let def = ToolRegistry::all()
            .iter()
            .find(|tool| tool.id == "file.read_range")
            .expect("file.read_range must be registered");
        assert_eq!(
            def.description,
            "Read a specific line range from a file with line numbers."
        );
        let schema = &def.input_schema;
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"], json!(["path", "start_line"]));
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["start_line"].is_object());
        assert!(schema["properties"]["end_line"].is_object());
    }

    #[test]
    fn code_review_profile_includes_file_read_range() {
        let tools = ToolRegistry::for_profile("code_review");
        assert!(tools.iter().any(|tool| tool.id == "file.read_range"));
    }

    #[test]
    fn code_review_profile_excludes_write_tools() {
        let tools = ToolRegistry::for_profile("code_review");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert!(!ids.contains(&"file.write"));
        assert!(!ids.contains(&"file.patch"));
        assert!(!ids.contains(&"git.commit"));
        assert!(!ids.contains(&"shell.run"));
    }

    #[test]
    fn code_review_profile_includes_cargo_check() {
        let tools = ToolRegistry::for_profile("code_review");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert!(ids.contains(&"cargo.check"));
        assert!(!ids.contains(&"cargo.test"));
    }

    #[test]
    fn github_profile_includes_github_and_git_tools() {
        let tools = ToolRegistry::for_profile("github");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert!(ids.contains(&"git.status"));
        assert!(ids.contains(&"git.branch"));
        assert!(ids.contains(&"git.commit"));
        assert!(ids.contains(&"github.issues"));
        assert!(ids.contains(&"github.prs"));
    }

    #[test]
    fn memory_search_is_mandatory_in_code_review_profile() {
        let tools = ToolRegistry::for_profile("code_review");
        assert!(tools.iter().any(|tool| tool.id == "memory.search"));
    }

    #[test]
    fn memory_search_is_mandatory_in_planner_profile() {
        let tools = ToolRegistry::for_profile("planner");
        assert!(tools.iter().any(|tool| tool.id == "memory.search"));
    }

    #[test]
    fn planner_profile_includes_read_and_search_tools() {
        let tools = ToolRegistry::for_profile("planner");
        let ids: Vec<_> = tools.iter().map(|tool| tool.id).collect();
        assert!(ids.contains(&"project.context"));
        assert!(ids.contains(&"file.read"));
        assert!(ids.contains(&"file.search"));
        assert!(ids.contains(&"file.grep"));
    }

    #[test]
    fn file_patch_schema_matches_spec() {
        let def = ToolRegistry::all()
            .iter()
            .find(|tool| tool.id == "file.patch")
            .expect("file.patch must be registered");
        assert_eq!(
            def.description,
            "Surgical edit: replace a unique old_str with new_str in a file."
        );
        let schema = &def.input_schema;
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"], json!(["path", "old_str", "new_str"]));
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["old_str"].is_object());
        assert!(schema["properties"]["new_str"].is_object());
    }

    #[test]
    fn parse_file_patch_requires_unique_fields() {
        let tool = KiwiTool::from_tool_use(
            "file.patch",
            &json!({"path": "src/main.rs", "old_str": "foo", "new_str": "bar"}),
        )
        .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::FilePatch {
                path,
                old_str,
                new_str
            } if path == "src/main.rs" && old_str == "foo" && new_str == "bar"
        ));
    }

    #[test]
    fn file_move_schema_matches_spec() {
        let def = ToolRegistry::all()
            .iter()
            .find(|tool| tool.id == "file.move")
            .expect("file.move must be registered");
        assert_eq!(
            def.description,
            "Rename or move a file within the repository."
        );
        let schema = &def.input_schema;
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"], json!(["src", "dest"]));
        assert!(schema["properties"]["src"].is_object());
        assert!(schema["properties"]["dest"].is_object());
    }

    #[test]
    fn parse_file_move_requires_src_and_dest() {
        let tool =
            KiwiTool::from_tool_use("file.move", &json!({"src": "old.rs", "dest": "new.rs"}))
                .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::FileMove { src, dest } if src == "old.rs" && dest == "new.rs"
        ));
        assert!(KiwiTool::from_tool_use("file.move", &json!({"src": "old.rs"})).is_err());
    }

    #[test]
    fn coding_profile_includes_file_move() {
        let tools = ToolRegistry::for_profile("coding");
        assert!(tools.iter().any(|tool| tool.id == "file.move"));
    }

    #[test]
    fn file_delete_schema_matches_spec() {
        let def = ToolRegistry::all()
            .iter()
            .find(|tool| tool.id == "file.delete")
            .expect("file.delete must be registered");
        assert_eq!(def.description, "Delete a file from the repository.");
        let schema = &def.input_schema;
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"], json!(["path"]));
        assert!(schema["properties"]["path"].is_object());
    }

    #[test]
    fn parse_file_delete_requires_path() {
        let tool = KiwiTool::from_tool_use("file.delete", &json!({"path": "src/old.rs"})).unwrap();
        assert!(matches!(tool, KiwiTool::FileDelete { path } if path == "src/old.rs"));
        assert!(KiwiTool::from_tool_use("file.delete", &json!({})).is_err());
    }

    #[test]
    fn coding_profile_includes_file_delete() {
        let tools = ToolRegistry::for_profile("coding");
        assert!(tools.iter().any(|tool| tool.id == "file.delete"));
    }

    #[test]
    fn parse_file_read_range_optional_end_line() {
        let tool = KiwiTool::from_tool_use(
            "file.read_range",
            &json!({"path": "src/lib.rs", "start_line": 10, "end_line": 20}),
        )
        .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::FileReadRange {
                start_line: 10,
                end_line: Some(20),
                ..
            }
        ));

        let tool = KiwiTool::from_tool_use(
            "file.read_range",
            &json!({"path": "src/lib.rs", "start_line": 1}),
        )
        .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::FileReadRange {
                start_line: 1,
                end_line: None,
                ..
            }
        ));
    }

    #[test]
    fn shell_capture_schema_matches_spec() {
        let def = ToolRegistry::all()
            .iter()
            .find(|tool| tool.id == "shell.capture")
            .expect("shell.capture must be registered");
        assert_eq!(
            def.description,
            "Run a shell command and return captured stdout/stderr to the agent."
        );
        let schema = &def.input_schema;
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"], json!(["command"]));
        assert!(schema["properties"]["command"].is_object());
        assert!(schema["properties"]["timeout_secs"].is_object());
    }

    #[test]
    fn parse_shell_capture_defaults_timeout() {
        let tool =
            KiwiTool::from_tool_use("shell.capture", &json!({"command": "echo hi"})).unwrap();
        assert!(matches!(
            tool,
            KiwiTool::ShellCapture {
                command,
                timeout_secs: 30
            } if command == "echo hi"
        ));
    }

    #[test]
    fn parse_shell_capture_clamps_timeout() {
        let tool = KiwiTool::from_tool_use(
            "shell.capture",
            &json!({"command": "echo hi", "timeout_secs": 999}),
        )
        .unwrap();
        assert!(matches!(
            tool,
            KiwiTool::ShellCapture {
                timeout_secs: 120,
                ..
            }
        ));
    }

    #[test]
    fn cargo_build_schema_matches_spec() {
        let def = ToolRegistry::all()
            .iter()
            .find(|tool| tool.id == "cargo.build")
            .expect("cargo.build must be registered");
        assert_eq!(
            def.description,
            "Run cargo build in the repository root and return build output."
        );
        let schema = &def.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["package"].is_object());
        assert!(schema["properties"]["release"].is_object());
    }

    #[test]
    fn coding_profile_includes_cargo_build() {
        let tools = ToolRegistry::for_profile("coding");
        assert!(tools.iter().any(|tool| tool.id == "cargo.build"));
    }

    #[test]
    fn parse_cargo_build_optional_release() {
        let tool = KiwiTool::from_tool_use("cargo.build", &json!({"release": true})).unwrap();
        assert!(matches!(
            tool,
            KiwiTool::CargoBuild {
                release: true,
                package: None
            }
        ));
    }

    #[test]
    fn parse_project_context_accepts_empty_input() {
        let tool = KiwiTool::from_tool_use("project.context", &json!({})).unwrap();
        assert!(matches!(tool, KiwiTool::ProjectContext));
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
        assert_eq!(schemas.len(), 22);
        let first = serde_json::to_value(&schemas[0]).unwrap();
        assert_eq!(first["type"], "function");
        assert_eq!(first["function"]["name"], "file_read");
        assert!(first["function"]["parameters"].is_object());
    }

    #[test]
    fn claude_adapter_uses_wire_tool_names() {
        let schemas = tools_for_claude(ToolRegistry::all());
        assert_eq!(schemas[0].name, "file_read");
    }

    #[test]
    fn openai_tool_names_use_underscores_not_dots() {
        for tool in ToolRegistry::all() {
            let wire = openai_tool_name(tool.id);
            assert!(
                wire.chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'),
                "invalid OpenAI tool name '{wire}' for id '{}'",
                tool.id
            );
            assert_eq!(kiwi_tool_id_from_openai(&wire), Some(tool.id));
        }
    }

    #[test]
    fn kiwi_tool_id_from_openai_accepts_legacy_dotted_names() {
        assert_eq!(kiwi_tool_id_from_openai("file.read"), Some("file.read"));
        assert_eq!(kiwi_tool_id_from_openai("cargo_check"), Some("cargo.check"));
    }

    #[test]
    fn ollama_supports_known_tool_models() {
        assert!(ollama_supports_tools("gpt-oss:20b"));
        assert!(ollama_supports_tools("qwen2.5-coder:7b"));
        assert!(ollama_supports_tools("llama3.1:8b"));
        assert!(ollama_supports_tools("mistral:latest"));
        assert!(!ollama_supports_tools("nomic-embed-text"));
    }

    #[test]
    fn parse_ollama_content_tool_calls_reads_qwen_json_blob() {
        let calls = parse_ollama_content_tool_calls(
            "{\n  \"name\": \"cargo_check\",\n  \"arguments\": {}\n}",
        );
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].wire_name, "cargo_check");
        assert_eq!(calls[0].arguments, json!({}));
    }

    #[test]
    fn parse_ollama_content_tool_calls_strips_null_optional_fields() {
        let calls = parse_ollama_content_tool_calls(
            r#"{"name":"cargo_check","arguments":{"package":null}}"#,
        );
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments, json!({"package": null}));
    }

    #[test]
    fn normalize_tool_arguments_drops_empty_strings() {
        let normalized = normalize_tool_arguments(json!({"package": ""}));
        assert_eq!(normalized, json!({}));
    }

    #[test]
    fn ollama_uses_native_tool_calls_for_llama3() {
        assert!(ollama_uses_native_tool_calls("gpt-oss:20b"));
        assert!(ollama_uses_native_tool_calls("llama3.1:8b"));
        assert!(!ollama_uses_native_tool_calls("qwen2.5-coder:7b"));
    }

    #[test]
    fn streaming_text_is_ollama_tool_json_detects_tool_blob() {
        assert!(streaming_text_is_ollama_tool_json(
            r#"{"name":"cargo_check","arguments":{}}"#
        ));
        assert!(!streaming_text_is_ollama_tool_json(
            "I'll run cargo check now."
        ));
    }

    #[test]
    fn registry_ids_use_dotted_namespace() {
        let ids: Vec<_> = ToolRegistry::all().iter().map(|tool| tool.id).collect();
        assert!(ids.iter().all(|id| id.contains('.')));
        assert!(ids.contains(&"file.read"));
        assert!(ids.contains(&"shell.run"));
    }
}
