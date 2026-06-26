//! TUI-facing agent API; domain logic lives in [`kiwi_core::agent`].

mod runtime;

#[allow(unused_imports)]
pub use kiwi_core::agent::{
    agent_launch_spec, AgentError, AgentId, AgentLaunchSpec, AgentManager, AgentOutputReader,
    AgentSession, AgentStatus,
};
pub use runtime::AgentRuntime;
