#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitHubLeftPane {
    #[default]
    Issues,
    Prs,
}

impl GitHubLeftPane {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Issues => 0,
            Self::Prs => 1,
        }
    }
}
