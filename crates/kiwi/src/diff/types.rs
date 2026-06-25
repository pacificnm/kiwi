#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffSource {
    #[default]
    Unstaged,
    #[allow(dead_code)] // wired by diff staged toggle (#52)
    Staged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Header,
    Context,
    Addition,
    Deletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiffLoadResult {
    pub path: String,
    pub lines: Vec<DiffLine>,
    pub is_binary: bool,
    pub error: Option<String>,
}

impl FileDiffLoadResult {
    #[must_use]
    pub fn error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            lines: Vec::new(),
            is_binary: false,
            error: Some(message.into()),
        }
    }
}
