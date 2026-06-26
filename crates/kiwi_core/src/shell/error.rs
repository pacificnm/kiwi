use std::fmt;

#[derive(Debug)]
pub enum ShellError {
    Spawn { message: String },
    Write { message: String },
    Resize { message: String },
}

impl ShellError {
    pub fn spawn(message: impl Into<String>) -> Self {
        Self::Spawn {
            message: message.into(),
        }
    }

    pub fn write(message: impl Into<String>) -> Self {
        Self::Write {
            message: message.into(),
        }
    }

    pub fn resize(message: impl Into<String>) -> Self {
        Self::Resize {
            message: message.into(),
        }
    }
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { message } => write!(f, "failed to spawn shell: {message}"),
            Self::Write { message } => write!(f, "failed to write to shell: {message}"),
            Self::Resize { message } => write!(f, "failed to resize shell pty: {message}"),
        }
    }
}

impl std::error::Error for ShellError {}
