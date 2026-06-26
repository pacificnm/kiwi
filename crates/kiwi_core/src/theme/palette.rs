use std::collections::HashMap;

use super::color::ResolvedColor;
use super::roles::SemanticRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RoleStyle {
    pub fg: Option<ResolvedColor>,
    pub bg: Option<ResolvedColor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePalette {
    pub name: String,
    roles: HashMap<SemanticRole, RoleStyle>,
}

impl ThemePalette {
    pub(crate) fn new(name: String, roles: HashMap<SemanticRole, RoleStyle>) -> Self {
        Self { name, roles }
    }

    #[must_use]
    pub fn get(&self, role: SemanticRole) -> RoleStyle {
        self.roles.get(&role).copied().unwrap_or_default()
    }

    #[must_use]
    pub fn git_modified_style(&self) -> RoleStyle {
        self.get(SemanticRole::GitModified)
    }
}
