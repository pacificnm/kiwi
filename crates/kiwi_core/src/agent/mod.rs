pub mod command;
mod id;
mod manager;
mod status;

pub use command::{agent_display_name, agent_launch_spec, AgentLaunchSpec};
pub use id::AgentId;
pub use manager::{
    AgentManager, AgentManagerError, IssueNumber, ManagedAgentSession, DEFAULT_MAX_AGENTS,
};
pub use status::{infer_status_from_scrollback, infer_status_from_text, AgentStatus};
