pub mod command;
mod error;
mod id;
mod io;
mod manager;
mod session;
mod status;

pub use command::{agent_display_name, agent_launch_spec, AgentLaunchSpec};
pub use error::AgentError;
pub use id::AgentId;
pub use io::AgentOutputReader;
pub use manager::{
    AgentManager, AgentManagerError, IssueNumber, ManagedAgentSession, DEFAULT_MAX_AGENTS,
};
pub use session::AgentSession;
pub use status::{infer_status_from_scrollback, infer_status_from_text, AgentStatus};
