//! Default directory entry names hidden from the file tree per SPEC-005 / ADR-008.

pub const DEFAULT_IGNORED_NAMES: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".venv",
];

#[must_use]
pub fn is_default_ignored(name: &str) -> bool {
    DEFAULT_IGNORED_NAMES.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ignore_list_matches_spec() {
        assert_eq!(
            DEFAULT_IGNORED_NAMES,
            &[
                ".git",
                "node_modules",
                "target",
                "dist",
                "build",
                ".next",
                ".nuxt",
                ".venv",
            ]
        );
    }

    #[test]
    fn is_default_ignored_matches_exact_names_only() {
        assert!(is_default_ignored("node_modules"));
        assert!(is_default_ignored(".git"));
        assert!(!is_default_ignored("Node_modules"));
        assert!(!is_default_ignored("src"));
        assert!(!is_default_ignored(".github"));
    }
}
