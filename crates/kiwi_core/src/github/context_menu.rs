#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhContextMenuAction {
    View,
    CreateBranch,
    Comment,
    AddLabels,
    AssignMilestone,
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
            Self::AssignMilestone => "Assign Milestone",
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
                GhContextMenuAction::AssignMilestone,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_menu_includes_github_actions() {
        let menu = GhContextMenuState::new(GhContextTarget::Issue { list_index: 0 }, 10, 5, false);
        assert_eq!(menu.items.len(), 7);
        assert!(menu.items.contains(&GhContextMenuAction::CreateBranch));
        assert!(menu.items.contains(&GhContextMenuAction::Comment));
        assert!(menu.items.contains(&GhContextMenuAction::AddLabels));
        assert!(menu.items.contains(&GhContextMenuAction::AssignMilestone));
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
}
