#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhContextMenuAction {
    View,
    CreateBranch,
    Comment,
    AddLabels,
    Merge,
    OpenInBrowser,
    SendToAgent,
}

impl GhContextMenuAction {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::View => "View",
            Self::CreateBranch => "Create Branch",
            Self::Comment => "Comment",
            Self::AddLabels => "Add Labels",
            Self::Merge => "Merge into main",
            Self::OpenInBrowser => "Open in Browser",
            Self::SendToAgent => "Send To Agent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhContextTarget {
    Issue { list_index: usize },
    PullRequest { list_index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhContextMenuState {
    pub target: GhContextTarget,
    pub anchor_x: u16,
    pub anchor_y: u16,
    pub items: Vec<GhContextMenuAction>,
    pub cursor: usize,
}

impl GhContextMenuState {
    #[must_use]
    pub fn new(
        target: GhContextTarget,
        anchor_x: u16,
        anchor_y: u16,
        merge_available: bool,
    ) -> Self {
        let items = match target {
            GhContextTarget::Issue { .. } => vec![
                GhContextMenuAction::View,
                GhContextMenuAction::CreateBranch,
                GhContextMenuAction::Comment,
                GhContextMenuAction::AddLabels,
                GhContextMenuAction::OpenInBrowser,
                GhContextMenuAction::SendToAgent,
            ],
            GhContextTarget::PullRequest { .. } => {
                let mut items = vec![GhContextMenuAction::View];
                if merge_available {
                    items.push(GhContextMenuAction::Merge);
                }
                items.push(GhContextMenuAction::OpenInBrowser);
                items.push(GhContextMenuAction::SendToAgent);
                items
            }
        };

        Self {
            target,
            anchor_x,
            anchor_y,
            items,
            cursor: 0,
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }

        let len = i32::try_from(self.items.len()).unwrap_or(1);
        let current = i32::try_from(self.cursor).unwrap_or(0);
        let next = (current + delta).rem_euclid(len);
        self.cursor = usize::try_from(next).unwrap_or(0);
    }

    #[must_use]
    pub fn selected_action(&self) -> Option<GhContextMenuAction> {
        self.items.get(self.cursor).copied()
    }

    #[must_use]
    pub fn menu_width(&self) -> u16 {
        self.items
            .iter()
            .map(|action| u16::try_from(action.label().len()).unwrap_or(0))
            .max()
            .unwrap_or(12)
            .saturating_add(4)
            .max(16)
    }

    #[must_use]
    pub fn menu_height(&self) -> u16 {
        u16::try_from(self.items.len())
            .unwrap_or(0)
            .saturating_add(2)
    }
}

/// GUI GitHub list context menu (#194): issue row actions.
pub const GUI_ISSUE_LIST_ACTIONS: [GhContextMenuAction; 3] = [
    GhContextMenuAction::View,
    GhContextMenuAction::CreateBranch,
    GhContextMenuAction::SendToAgent,
];

/// GUI GitHub list context menu (#194): PR row actions.
pub const GUI_PR_LIST_ACTIONS: [GhContextMenuAction; 2] = [
    GhContextMenuAction::View,
    GhContextMenuAction::SendToAgent,
];

#[must_use]
pub fn format_issue_agent_prompt(number: u32, title: &str, body_excerpt: Option<&str>) -> String {
    let mut prompt = format!(
        "Please help me work on GitHub issue #{number}: {title}\n\n\
         Review the issue context and suggest an implementation approach.\n"
    );

    if let Some(body) = body_excerpt.filter(|text| !text.trim().is_empty()) {
        prompt.push_str("\nIssue description:\n");
        prompt.push_str(body.trim());
        prompt.push('\n');
    }

    prompt
}

const BODY_EXCERPT_MAX_CHARS: usize = 400;

/// Extract a truncated issue body from loaded detail display lines, if present.
#[must_use]
pub fn issue_body_excerpt_from_detail(detail: &crate::github::IssueDetail) -> Option<String> {
    let body_start = detail
        .display_lines
        .iter()
        .position(|line| line.as_str() == "— Body —")?
        .saturating_add(2);
    let body_end = detail.display_lines[body_start..]
        .iter()
        .position(|line| line.starts_with("— Comments"))?;
    let text = detail.display_lines[body_start..body_start.saturating_add(body_end)]
        .join("\n")
        .trim()
        .to_string();

    if text.is_empty() || text == "(empty)" {
        return None;
    }

    Some(truncate_excerpt(&text, BODY_EXCERPT_MAX_CHARS))
}

fn truncate_excerpt(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let truncated: String = text.chars().take(max_chars).collect();
    format!("{truncated}…")
}

#[must_use]
pub fn format_pr_agent_prompt(number: u32, title: &str) -> String {
    format!(
        "Please help me review GitHub PR #{number}: {title}\n\n\
         Summarize the changes and suggest improvements.\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_menu_includes_github_actions() {
        let menu = GhContextMenuState::new(GhContextTarget::Issue { list_index: 0 }, 10, 5, false);
        assert_eq!(menu.items.len(), 6);
        assert!(menu.items.contains(&GhContextMenuAction::CreateBranch));
        assert!(menu.items.contains(&GhContextMenuAction::Comment));
        assert!(menu.items.contains(&GhContextMenuAction::AddLabels));
        assert!(menu.items.contains(&GhContextMenuAction::OpenInBrowser));
    }

    #[test]
    fn pr_menu_includes_merge_when_available() {
        let menu =
            GhContextMenuState::new(GhContextTarget::PullRequest { list_index: 0 }, 10, 5, true);
        assert_eq!(menu.items.len(), 4);
        assert!(menu.items.contains(&GhContextMenuAction::Merge));
        assert!(menu.items.contains(&GhContextMenuAction::OpenInBrowser));
    }

    #[test]
    fn pr_menu_omits_merge_when_unavailable() {
        let menu =
            GhContextMenuState::new(GhContextTarget::PullRequest { list_index: 0 }, 10, 5, false);
        assert_eq!(menu.items.len(), 3);
        assert!(!menu.items.contains(&GhContextMenuAction::Merge));
        assert!(!menu.items.contains(&GhContextMenuAction::CreateBranch));
        assert!(!menu.items.contains(&GhContextMenuAction::Comment));
        assert!(menu.items.contains(&GhContextMenuAction::OpenInBrowser));
    }

    #[test]
    fn gui_issue_list_actions_match_v1_spec() {
        assert_eq!(GUI_ISSUE_LIST_ACTIONS.len(), 3);
        assert!(GUI_ISSUE_LIST_ACTIONS.contains(&GhContextMenuAction::CreateBranch));
        assert!(!GUI_PR_LIST_ACTIONS.contains(&GhContextMenuAction::CreateBranch));
    }

    #[test]
    fn format_issue_agent_prompt_uses_title() {
        let prompt = format_issue_agent_prompt(42, "Add context menu", None);
        assert!(prompt.contains("#42"));
        assert!(prompt.contains("Add context menu"));
    }

    #[test]
    fn format_issue_agent_prompt_includes_body_excerpt() {
        let prompt = format_issue_agent_prompt(42, "Add context menu", Some("Fix GH list UX"));
        assert!(prompt.contains("Issue description:"));
        assert!(prompt.contains("Fix GH list UX"));
    }

    #[test]
    fn issue_body_excerpt_from_detail_skips_empty_body() {
        use crate::github::{IssueDetail, IssueState};

        let detail = IssueDetail {
            number: 1,
            title: "Test".to_string(),
            state: IssueState::Open,
            author: "dev".to_string(),
            labels: Vec::new(),
            assignees: Vec::new(),
            display_lines: vec![
                "#1 Test".to_string(),
                "— Body —".to_string(),
                String::new(),
                "(empty)".to_string(),
                String::new(),
                "— Comments —".to_string(),
            ],
        };
        assert!(issue_body_excerpt_from_detail(&detail).is_none());
    }

    #[test]
    fn issue_body_excerpt_from_detail_truncates_long_body() {
        use crate::github::{IssueDetail, IssueState};

        let body = "a".repeat(500);
        let detail = IssueDetail {
            number: 1,
            title: "Test".to_string(),
            state: IssueState::Open,
            author: "dev".to_string(),
            labels: Vec::new(),
            assignees: Vec::new(),
            display_lines: vec![
                "#1 Test".to_string(),
                "— Body —".to_string(),
                String::new(),
                body,
                String::new(),
                "— Comments —".to_string(),
            ],
        };
        let excerpt = issue_body_excerpt_from_detail(&detail).expect("excerpt");
        assert!(excerpt.ends_with('…'));
        assert!(excerpt.chars().count() <= BODY_EXCERPT_MAX_CHARS + 1);
    }

    #[test]
    fn format_pr_agent_prompt_uses_title() {
        let prompt = format_pr_agent_prompt(7, "GUI follow-up");
        assert!(prompt.contains("#7"));
        assert!(prompt.contains("GUI follow-up"));
    }
}
