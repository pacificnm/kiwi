use std::fmt;

#[derive(Debug)]
pub enum AgentError {
    Spawn { message: String },
}

impl AgentError {
    pub fn spawn(message: impl Into<String>) -> Self {
        Self::Spawn {
            message: message.into(),
        }
    }
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { message } => write!(f, "failed to spawn agent: {message}"),
        }
    }
}

impl std::error::Error for AgentError {}
