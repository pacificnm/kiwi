//! Stream-cancel handle registry for concurrent native-chat API streams (ADR-017).
//!
//! PTY session management was removed in Phase 6 (#334). Only cancel handles remain.

use std::collections::HashMap;

use kiwi_core::agent::{AgentId, StreamCancelHandle};

pub struct AgentRuntime {
    stream_cancels: HashMap<AgentId, StreamCancelHandle>,
}

impl AgentRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stream_cancels: HashMap::new(),
        }
    }

    /// Register a cancel handle for a native-chat stream, cancelling any prior stream for this agent.
    pub fn register_stream(&mut self, id: AgentId, cancel: StreamCancelHandle) {
        if let Some(prev) = self.stream_cancels.remove(&id) {
            prev.cancel();
        }
        self.stream_cancels.insert(id, cancel);
    }

    /// Cancel the active stream for the given agent, if any.
    pub fn cancel_stream(&mut self, id: AgentId) {
        if let Some(cancel) = self.stream_cancels.remove(&id) {
            cancel.cancel();
        }
    }

    /// Cancel all active streams (called on shutdown).
    pub fn cancel_all_streams(&mut self) {
        for (_, cancel) in self.stream_cancels.drain() {
            cancel.cancel();
        }
    }
}

impl Default for AgentRuntime {
    fn default() -> Self {
        Self::new()
    }
}
