mod generate;
mod io;

pub use generate::load_file_diff;
pub use io::spawn_file_diff_load;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffSource {
    #[default]
    Unstaged,
    Staged,
}

impl DiffSource {
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Unstaged => Self::Staged,
            Self::Staged => Self::Unstaged,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_source_toggle_alternates() {
        assert_eq!(DiffSource::Unstaged.toggle(), DiffSource::Staged);
        assert_eq!(DiffSource::Staged.toggle(), DiffSource::Unstaged);
    }
}
