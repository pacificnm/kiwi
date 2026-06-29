pub mod api_client;
pub mod chat;
pub mod command;
mod error;
mod id;
mod io;
mod manager;
mod session;
mod status;
mod stream_event;
pub mod tool_executor;
pub mod tools;

pub use api_client::{spawn_claude_stream, spawn_ollama_stream, StreamCancelHandle};
pub use chat::{
    AgentProvider, ChatMessage, ChatSession, ContentBlock, MessageRole, ToolResult, ToolUse,
};
pub use tool_executor::{execute_tool, ExecutionResult};
pub use tools::{KiwiTool, ToolParseError, ToolSchema};
pub use command::{agent_display_name, agent_launch_spec, AgentLaunchSpec};
pub use error::AgentError;
pub use id::AgentId;
pub use io::AgentOutputReader;
pub use manager::{
    AgentManager, AgentManagerError, IssueNumber, ManagedAgentSession, DEFAULT_MAX_AGENTS,
};
pub use session::AgentSession;
pub use status::{infer_status_from_scrollback, infer_status_from_text, AgentStatus};
