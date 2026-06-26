use std::fmt;

/// Stable identifier for a managed agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(u32);

impl AgentId {
    pub const FIRST: Self = Self(1);

    #[must_use]
    pub fn from_u32(id: u32) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
