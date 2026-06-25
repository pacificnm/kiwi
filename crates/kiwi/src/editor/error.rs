#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorLaunchError {
    FileNotFound {
        path: String,
    },
    CommandNotFound {
        command: String,
        resolution_hint: String,
    },
    SpawnFailed {
        message: String,
    },
}

impl EditorLaunchError {
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::FileNotFound { path } => format!("File not found: {path}"),
            Self::CommandNotFound {
                command,
                resolution_hint,
            } => format!("Editor command not found: {command}. {resolution_hint}"),
            Self::SpawnFailed { message } => format!("Failed to launch editor: {message}"),
        }
    }

    #[must_use]
    pub fn is_command_not_found(&self) -> bool {
        matches!(self, Self::CommandNotFound { .. })
    }
}

impl std::fmt::Display for EditorLaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message())
    }
}
