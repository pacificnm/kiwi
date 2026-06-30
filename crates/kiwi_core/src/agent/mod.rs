pub mod api_client;
pub mod chat;
pub mod command;
mod error;
mod id;
mod io;
mod manager;
mod memory_client;
mod session;
mod status;
mod stream_event;
pub mod tool_executor;
pub mod tools;

pub use api_client::{
    resolve_ollama_stream, spawn_claude_stream, spawn_cursor_stream, spawn_ollama_stream,
    spawn_openai_stream, OllamaHistoryFormat, OllamaStreamPlan, StreamCancelHandle,
};
pub use chat::{
    AgentProvider, ChatMessage, ChatSession, ContentBlock, MessageRole, ToolResult, ToolUse,
};
pub use command::{agent_display_name, agent_launch_spec, AgentLaunchSpec};
pub use error::AgentError;
pub use id::AgentId;
pub use io::AgentOutputReader;
pub use manager::{
    AgentManager, AgentManagerError, IssueNumber, ManagedAgentSession, DEFAULT_MAX_AGENTS,
};
pub use session::AgentSession;
pub use status::{infer_status_from_scrollback, infer_status_from_text, AgentStatus};
pub use tool_executor::{execute_tool, ExecutionResult};
pub use tools::{
    kiwi_tool_id_from_openai, normalize_tool_arguments, normalize_tool_arguments_json,
    ollama_split_models, ollama_supports_tools, ollama_uses_native_tool_calls,
    openai_tool_name, parse_ollama_content_tool_calls, resolve_tool_profile,
    tools_for_ollama, MAX_TOOL_ROUNDS_PER_TURN, streaming_text_is_ollama_tool_json,
    tool_profile_by_name,
    tools_for_claude, tools_for_openai, KiwiTool, KiwiToolDef, OpenAiToolSchema, ToolParseError,
    ToolProfile, ToolRegistry, ToolSchema,
};
