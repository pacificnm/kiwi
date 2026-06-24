use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    pub path: Option<PathBuf>,
    pub message: String,
}

impl ConfigError {
    #[must_use]
    pub fn parse(path: PathBuf, content: &str, source: toml::de::Error) -> Self {
        let message = format_toml_parse_error(content, &source);
        Self {
            path: Some(path),
            message,
        }
    }

    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            path: None,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn io(path: PathBuf, source: std::io::Error) -> Self {
        Self {
            path: Some(path),
            message: source.to_string(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(f, "{}: {}", path.display(), self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for ConfigError {}

fn format_toml_parse_error(content: &str, err: &toml::de::Error) -> String {
    if let Some(span) = err.span() {
        let line = content[..span.start]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1;
        return format!("{} (line {line})", err.message());
    }
    err.message().to_string()
}
