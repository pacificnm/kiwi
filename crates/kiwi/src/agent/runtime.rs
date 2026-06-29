//! Runtime PTY handles for multiple concurrent agent sessions (ADR-017, #72).

use std::collections::HashMap;
use std::io::Read;

use crate::state::EventSender;

use kiwi_core::agent::{AgentId, AgentOutputReader, AgentSession};

pub struct AgentRuntime {
    sessions: HashMap<AgentId, AgentSession>,
    readers: HashMap<AgentId, AgentOutputReader>,
}

impl AgentRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            readers: HashMap::new(),
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
        let ids: Vec<AgentId> = self.sessions.keys().copied().collect();
        for id in ids {
            self.shutdown(id);
        }
    }

    #[cfg(test)]
    #[must_use]
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for AgentRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::state::AgentState;
    use kiwi_core::agent::AgentSession;
    use kiwi_core::config::AgentSettings;

    use super::*;

    #[test]
    fn attach_session_tracks_runtime_handle() {
        if !Path::new("/bin/bash").exists() && !Path::new("/usr/bin/bash").exists() {
            return;
        }

        let repo = std::env::temp_dir().join("kiwi-agent-runtime-test");
        std::fs::create_dir_all(&repo).expect("temp dir");
        let settings = AgentSettings {
            command: "bash".to_string(),
            args: Vec::new(),
            env: Default::default(),
            ..AgentSettings::default()
        };
        let session = AgentSession::spawn(&repo, &settings, 80, 24).expect("spawn");
        let mut runtime = AgentRuntime::new();
        let id = AgentId::FIRST;

        runtime.attach_session(id, session);
        assert_eq!(runtime.session_count(), 1);

        let mut pty = AgentState::default();
        pty.apply_spawn("bash", "bash", Some(1), 80, 24);
        assert!(pty.spawned);

        runtime.shutdown_all();
    }
}
