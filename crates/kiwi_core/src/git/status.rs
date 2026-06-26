use crate::theme::SemanticRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum GitFileStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
}

impl GitFileStatus {
    #[must_use]
    pub const fn badge(self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Untracked => "U",
        }
    }

    #[must_use]
    pub const fn semantic_role(self) -> SemanticRole {
        match self {
            Self::Modified => SemanticRole::GitModified,
            Self::Added => SemanticRole::GitAdded,
            Self::Deleted => SemanticRole::GitDeleted,
            Self::Untracked => SemanticRole::GitUntracked,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitFileEntry {
    pub path: String,
    pub status: GitFileStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_chars_match_spec_roles() {
        assert_eq!(GitFileStatus::Modified.badge(), "M");
        assert_eq!(GitFileStatus::Added.semantic_role(), SemanticRole::GitAdded);
        assert_eq!(GitFileStatus::Deleted.badge(), "D");
        assert_eq!(
            GitFileStatus::Deleted.semantic_role(),
            SemanticRole::GitDeleted
        );
        assert_eq!(
            GitFileStatus::Untracked.semantic_role(),
            SemanticRole::GitUntracked
        );
    }
}
