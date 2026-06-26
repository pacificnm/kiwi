//! Multi-agent session state per ADR-017 (#71).
//!
//! Runtime PTY handles remain in [`super::session::AgentSession`] and
//! [`super::runtime::AgentRuntime`]; this module holds managed per-agent PTY
//! state and metadata (`label`, `linked_issue`).

#![allow(dead_code)] // remove_agent / link_active_issue for future session management.

use std::collections::HashMap;
use std::fmt;

use crate::state::AgentState;

use super::id::AgentId;

/// GitHub issue number linked to an agent session (ADR-017).
pub type IssueNumber = u64;

/// Per-agent session metadata and PTY state (ADR-017 `AgentSession`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedAgentSession {
    pub id: AgentId,
    pub label: String,
    pub linked_issue: Option<IssueNumber>,
    pub pty: AgentState,
}

impl ManagedAgentSession {
    #[must_use]
    pub fn new(id: AgentId, label: String, pty: AgentState) -> Self {
        Self {
            id,
            label,
            linked_issue: None,
            pty,
        }
    }

    #[must_use]
    pub fn with_linked_issue(mut self, issue: IssueNumber) -> Self {
        self.linked_issue = Some(issue);
        self
    }
}

/// Default maximum concurrent agent sessions (ADR-017).
pub const DEFAULT_MAX_AGENTS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentManagerError {
    AtCapacity,
    UnknownAgent(AgentId),
    CannotRemoveLast,
}

impl fmt::Display for AgentManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AtCapacity => write!(f, "agent session limit reached"),
            Self::UnknownAgent(id) => write!(f, "unknown agent id {id}"),
            Self::CannotRemoveLast => write!(f, "cannot remove the last agent session"),
        }
    }
}

/// Owns all managed agent sessions and tracks the active one (ADR-017).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentManager {
    agents: HashMap<AgentId, ManagedAgentSession>,
    active_agent: AgentId,
    next_id: u32,
    max_agents: usize,
}

impl AgentManager {
    #[must_use]
    pub fn with_initial_agent(pty: AgentState) -> Self {
        let id = AgentId::FIRST;
        let label = default_label_for_index(1, &pty);
        let mut agents = HashMap::new();
        agents.insert(id, ManagedAgentSession::new(id, label, pty));
        Self {
            agents,
            active_agent: id,
            next_id: id.as_u32() + 1,
            max_agents: DEFAULT_MAX_AGENTS,
        }
    }

    #[must_use]
    pub fn with_max_agents(mut self, max_agents: usize) -> Self {
        self.max_agents = max_agents.max(1);
        self
    }

    #[must_use]
    pub fn active_id(&self) -> AgentId {
        self.active_agent
    }

    #[must_use]
    pub fn session_count(&self) -> usize {
        self.agents.len()
    }

    #[must_use]
    pub fn running_count(&self) -> usize {
        self.agents
            .values()
            .filter(|session| session.pty.running)
            .count()
    }

    #[must_use]
    pub fn max_agents(&self) -> usize {
        self.max_agents
    }

    pub fn sessions(&self) -> impl Iterator<Item = &ManagedAgentSession> {
        self.session_ids().filter_map(|id| self.agents.get(&id))
    }

    pub fn session_ids(&self) -> impl Iterator<Item = AgentId> + '_ {
        self.ordered_ids()
    }

    #[must_use]
    pub fn pty(&self, id: AgentId) -> Option<&AgentState> {
        self.agents.get(&id).map(|session| &session.pty)
    }

    pub fn pty_mut(&mut self, id: AgentId) -> Option<&mut AgentState> {
        self.agents.get_mut(&id).map(|session| &mut session.pty)
    }

    pub fn cycle_active(&mut self, delta: i32) -> AgentId {
        let ids: Vec<AgentId> = self.ordered_ids().collect();
        if ids.is_empty() {
            return self.active_agent;
        }

        let current = ids
            .iter()
            .position(|&id| id == self.active_agent)
            .unwrap_or(0);
        let len = ids.len() as i32;
        let next = (current as i32 + delta).rem_euclid(len) as usize;
        self.active_agent = ids[next];
        self.active_agent
    }

    #[must_use]
    pub fn active_session(&self) -> &ManagedAgentSession {
        self.agents
            .get(&self.active_agent)
            .expect("active agent id must exist")
    }

    #[must_use]
    pub fn active_session_mut(&mut self) -> &mut ManagedAgentSession {
        let active = self.active_agent;
        self.agents
            .get_mut(&active)
            .expect("active agent id must exist")
    }

    #[must_use]
    pub fn active_pty(&self) -> &AgentState {
        &self.active_session().pty
    }

    pub fn active_pty_mut(&mut self) -> &mut AgentState {
        &mut self.active_session_mut().pty
    }

    pub fn session_id_at_index(&self, index: usize) -> Option<AgentId> {
        self.session_ids().nth(index)
    }

    pub fn set_active(&mut self, id: AgentId) -> Result<(), AgentManagerError> {
        if !self.agents.contains_key(&id) {
            return Err(AgentManagerError::UnknownAgent(id));
        }
        self.active_agent = id;
        Ok(())
    }

    pub fn create_agent(
        &mut self,
        label: Option<String>,
        linked_issue: Option<IssueNumber>,
    ) -> Result<AgentId, AgentManagerError> {
        if self.agents.len() >= self.max_agents {
            return Err(AgentManagerError::AtCapacity);
        }

        let id = AgentId::from_u32(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        let index = self.agents.len() + 1;
        let template = self.active_pty();
        let mut pty = AgentState {
            command: template.command.clone(),
            agent_name: template.agent_name.clone(),
            ..AgentState::default()
        };
        let session_label = label.unwrap_or_else(|| default_label_for_index(index, &pty));
        if let Some(name) = session_label.strip_prefix("Agent ") {
            pty.agent_name = name.to_string();
        } else {
            pty.agent_name = session_label.clone();
        }

        let session = ManagedAgentSession {
            id,
            label: session_label,
            linked_issue,
            pty,
        };
        self.agents.insert(id, session);
        self.active_agent = id;
        Ok(id)
    }

    pub fn remove_agent(&mut self, id: AgentId) -> Result<(), AgentManagerError> {
        if self.agents.len() <= 1 {
            return Err(AgentManagerError::CannotRemoveLast);
        }
        if !self.agents.contains_key(&id) {
            return Err(AgentManagerError::UnknownAgent(id));
        }

        let reassign_active = self.active_agent == id;
        let next_active = reassign_active.then(|| {
            self.ordered_ids()
                .find(|&candidate| candidate != id)
                .expect("another agent must exist")
        });
        self.agents.remove(&id);
        if let Some(next) = next_active {
            self.active_agent = next;
        }
        Ok(())
    }

    pub fn link_active_issue(&mut self, issue: IssueNumber) {
        self.active_session_mut().linked_issue = Some(issue);
    }

    /// Status-bar label for the agent segment (ADR-017 future UX).
    #[must_use]
    pub fn status_bar_label(&self) -> String {
        let active = self.active_pty();
        if self.session_count() <= 1 {
            return active.status.status_bar_label(active.running).to_string();
        }

        let total = self.session_count();
        let running = self.running_count();
        format!("{total} Agents ({running} Running)")
    }

    fn ordered_ids(&self) -> impl Iterator<Item = AgentId> + '_ {
        let mut ids: Vec<AgentId> = self.agents.keys().copied().collect();
        ids.sort_by_key(|id| id.as_u32());
        ids.into_iter()
    }
}

fn default_label_for_index(index: usize, pty: &AgentState) -> String {
    if !pty.agent_name.is_empty() {
        return pty.agent_name.clone();
    }
    format!("Agent {index}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentStatus;

    fn sample_pty(name: &str) -> AgentState {
        AgentState {
            agent_name: name.to_string(),
            ..AgentState::default()
        }
    }

    #[test]
    fn starts_with_one_default_session() {
        let manager = AgentManager::with_initial_agent(sample_pty("cursor"));
        assert_eq!(manager.session_count(), 1);
        assert_eq!(manager.active_id(), AgentId::FIRST);
        assert_eq!(manager.active_session().label, "cursor");
    }

    #[test]
    fn create_agent_adds_session_and_activates_it() {
        let mut manager = AgentManager::with_initial_agent(sample_pty("cursor"));
        let id = manager
            .create_agent(Some("tests".to_string()), Some(42))
            .expect("create");

        assert_eq!(manager.session_count(), 2);
        assert_eq!(manager.active_id(), id);
        assert_eq!(manager.active_session().label, "tests");
        assert_eq!(manager.active_session().linked_issue, Some(42));
    }

    #[test]
    fn create_agent_respects_capacity() {
        let mut manager = AgentManager::with_initial_agent(sample_pty("cursor")).with_max_agents(2);
        manager.create_agent(None, None).expect("second");
        let err = manager.create_agent(None, None).expect_err("full");
        assert_eq!(err, AgentManagerError::AtCapacity);
    }

    #[test]
    fn set_active_switches_pty_target() {
        let mut manager = AgentManager::with_initial_agent(sample_pty("first"));
        let second = manager
            .create_agent(Some("second".to_string()), None)
            .expect("create");
        manager.set_active(AgentId::FIRST).expect("switch");
        assert_eq!(manager.active_session().label, "first");
        manager.set_active(second).expect("switch back");
        assert_eq!(manager.active_session().label, "second");
    }

    #[test]
    fn remove_agent_reassigns_active_when_needed() {
        let mut manager = AgentManager::with_initial_agent(sample_pty("first"));
        let second = manager
            .create_agent(Some("second".to_string()), None)
            .expect("create");
        manager.remove_agent(second).expect("remove");
        assert_eq!(manager.session_count(), 1);
        assert_eq!(manager.active_id(), AgentId::FIRST);
    }

    #[test]
    fn cannot_remove_last_agent() {
        let mut manager = AgentManager::with_initial_agent(sample_pty("only"));
        let err = manager.remove_agent(AgentId::FIRST).expect_err("last");
        assert_eq!(err, AgentManagerError::CannotRemoveLast);
    }

    #[test]
    fn cycle_active_wraps_sessions() {
        let mut manager = AgentManager::with_initial_agent(sample_pty("first"));
        let second = manager
            .create_agent(Some("second".to_string()), None)
            .expect("create");
        assert_eq!(manager.active_id(), second);
        assert_eq!(manager.cycle_active(1), AgentId::FIRST);
        assert_eq!(manager.cycle_active(1), second);
        assert_eq!(manager.cycle_active(-1), AgentId::FIRST);
    }

    #[test]
    fn status_bar_label_single_agent_uses_pty_status() {
        let mut manager = AgentManager::with_initial_agent(sample_pty("cursor"));
        manager.active_pty_mut().running = true;
        manager.active_pty_mut().status = AgentStatus::Idle;
        assert_eq!(manager.status_bar_label(), "Agent Running");
    }

    #[test]
    fn status_bar_label_multi_agent_shows_counts() {
        let mut manager = AgentManager::with_initial_agent(sample_pty("first"));
        manager.active_pty_mut().running = true;
        manager.create_agent(None, None).expect("second");
        assert_eq!(manager.status_bar_label(), "2 Agents (1 Running)");
    }
}
