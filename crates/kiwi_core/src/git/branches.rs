#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchEntry {
    pub name: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchDetail {
    pub name: String,
    pub is_current: bool,
    pub tip_sha: String,
    pub tip_subject: String,
    pub tip_author: String,
    pub tip_date: String,
}

impl BranchDetail {
    #[must_use]
    pub fn display_lines(&self, ahead: u32, behind: u32) -> Vec<String> {
        let mut lines = vec![
            format!("Branch: {}", self.name),
            if self.is_current {
                "Status: current branch".to_string()
            } else {
                "Status: local branch".to_string()
            },
        ];
        if self.is_current && (ahead > 0 || behind > 0) {
            lines.push(format!("Upstream: ↑{ahead} ↓{behind}"));
        }
        lines.push(String::new());
        lines.push(format!("Tip: {}", self.tip_sha));
        lines.push(self.tip_subject.clone());
        lines.push(format!("Author: {}", self.tip_author));
        lines.push(format!("Date: {}", self.tip_date));
        lines
    }
}
