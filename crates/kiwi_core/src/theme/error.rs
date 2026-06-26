use std::fmt;
use std::path::PathBuf;

use super::roles::SemanticRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeError {
    UnknownTheme(String),
    Parse {
        path: Option<PathBuf>,
        message: String,
    },
    InvalidColor {
        role: SemanticRole,
        value: String,
    },
    MissingRole(SemanticRole),
}

impl ThemeError {
    #[must_use]
    pub fn invalid_color(role: SemanticRole, value: &str) -> Self {
        Self::InvalidColor {
            role,
            value: value.to_string(),
        }
    }
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTheme(name) => write!(f, "unknown theme: {name}"),
            Self::Parse { path, message } => match path {
                Some(path) => write!(f, "{}: {message}", path.display()),
                None => write!(f, "{message}"),
            },
            Self::InvalidColor { role, value } => {
                write!(f, "invalid color for {role}: {value}")
            }
            Self::MissingRole(role) => write!(f, "missing theme role: {role}"),
        }
    }
}

impl std::error::Error for ThemeError {}
