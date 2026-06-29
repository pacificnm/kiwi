//! Runtime PTY handles for multiple concurrent agent sessions (ADR-017).

use std::collections::HashMap;
use std::io::Read;

use kiwi_core::agent::{AgentId, AgentOutputReader, AgentSession, StreamCancelHandle};
use kiwi_core::events::EventSender;

pub struct AgentRuntime {
    sessions: HashMap<AgentId, AgentSession>,
    readers: HashMap<AgentId, AgentOutputReader>,
    /// Cancel handles for in-progress native-chat API streams.
    stream_cancels: HashMap<AgentId, StreamCancelHandle>,
}

impl AgentRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            readers: HashMap::new(),
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

    fn cancel_all_streams(&mut self) {
        for (_, cancel) in self.stream_cancels.drain() {
            cancel.cancel();
        }
    }

    #[must_use]
    pub fn has_session(&self, id: AgentId) -> bool {
        self.sessions.contains_key(&id)
    }

    pub fn attach_session(&mut self, id: AgentId, session: AgentSession) {
        self.sessions.insert(id, session);
    }

    pub fn attach_reader(
        &mut self,
        id: AgentId,
        reader: Box<dyn Read + Send>,
        sender: EventSender,
    ) {
        self.readers
            .insert(id, AgentOutputReader::spawn(reader, id, sender));
    }

    pub fn write(&mut self, id: AgentId, data: &[u8]) -> bool {
        let Some(session) = self.sessions.get_mut(&id) else {
            return false;
        };
        session.write(data).is_ok()
    }

    pub fn resize(&mut self, id: AgentId, cols: u16, rows: u16) -> bool {
        let Some(session) = self.sessions.get_mut(&id) else {
            return false;
        };
        session.resize(cols, rows).is_ok()
    }

    #[must_use]
    pub fn poll_exits(&mut self) -> Vec<(AgentId, i32)> {
        let ids: Vec<AgentId> = self.sessions.keys().copied().collect();
        let mut exits = Vec::new();

        for id in ids {
            let Some(session) = self.sessions.get_mut(&id) else {
                continue;
            };
            let Some(code) = session.poll_exit() else {
                continue;
            };
            if let Some(reader) = self.readers.remove(&id) {
                reader.abandon();
            }
            self.sessions.remove(&id);
            exits.push((id, code));
        }

        exits
    }

    pub fn shutdown(&mut self, id: AgentId) {
        if let Some(reader) = self.readers.remove(&id) {
            reader.abandon();
        }
        if let Some(mut session) = self.sessions.remove(&id) {
            session.shutdown();
        }
    }

    pub fn shutdown_all(&mut self) {
        self.cancel_all_streams();
        let ids: Vec<AgentId> = self.sessions.keys().copied().collect();
        for id in ids {
            self.shutdown(id);
        }
    }
}

impl Default for AgentRuntime {
    fn default() -> Self {
        Self::new()
    }
}
