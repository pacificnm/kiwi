//! Right-click context menu for GitHub issue/PR list rows (#194).

use egui::Ui;
use kiwi_core::events::AppCommand;
use kiwi_core::github::{GhContextMenuAction, GhContextTarget};

use super::context_menu::{github_action_icon, menu_action, render_menu_shell};
use super::github_common::{select_branch_commands, select_issue_commands, select_pr_commands};
use crate::dock::context::PanelContext;

const GUI_ISSUE_LIST_ACTIONS: [GhContextMenuAction; 6] = [
    GhContextMenuAction::View,
    GhContextMenuAction::CreateBranch,
    GhContextMenuAction::AddLabels,
    GhContextMenuAction::AssignMilestone,
    GhContextMenuAction::Comment,
    GhContextMenuAction::SendToAgent,
];

const GUI_PR_LIST_ACTIONS: [GhContextMenuAction; 2] = [
    GhContextMenuAction::View,
    GhContextMenuAction::SendToAgent,
];

const GUI_BRANCH_LIST_ACTIONS: [BranchListAction; 2] =
    [BranchListAction::View, BranchListAction::Checkout];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BranchListAction {
    View,
    Checkout,
}

impl BranchListAction {
    const fn label(self) -> &'static str {
        match self {
            Self::View => "View",
            Self::Checkout => "Checkout",
        }
    }

    const fn icon(self) -> &'static str {
        match self {
            Self::View => "◎",
            Self::Checkout => "⎇",
        }
    }
}

/// Build commands for a branch list context-menu action (testable without egui).
#[must_use]
pub fn commands_for_branch_list_action(
    row_index: usize,
    action: BranchListAction,
) -> Vec<AppCommand> {
    match action {
        BranchListAction::View => select_branch_commands(row_index).to_vec(),
        BranchListAction::Checkout => vec![
            AppCommand::BranchSelect(row_index),
            AppCommand::BranchCheckoutSelected,
        ],
    }
}

/// Build commands for a list context-menu action (testable without egui).
#[must_use]
pub fn commands_for_github_list_action(
    target: GhContextTarget,
    action: GhContextMenuAction,
) -> Vec<AppCommand> {
    match action {
        GhContextMenuAction::View => match target {
            GhContextTarget::Issue { list_index } => select_issue_commands(list_index).to_vec(),
            GhContextTarget::PullRequest { list_index } => {
                select_pr_commands(list_index).to_vec()
            }
        },
        action => vec![AppCommand::GitHubListAction { target, action }],
    }
}

/// Commands to open the new-issue modal (testable without egui).
#[must_use]
pub fn commands_for_new_issue() -> Vec<AppCommand> {
    vec![AppCommand::GitHubIssueCreateOpen]
}

pub fn render_gh_issues_panel_context_menu(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    render_menu_shell(ui, ctx.theme, "GitHub Issues", |ui| {
        if menu_action(ui, ctx.theme, "＋", "New Issue") {
            for command in commands_for_new_issue() {
                let _ = (ctx.dispatch)(command);
            }
            ui.close_menu();
        }
    });
}

pub fn render_issue_list_context_menu(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    row_index: usize,
) {
    let Some(issue) = ctx.state.github.issues.get(row_index) else {
        return;
    };
    let title = format!("#{} · {}", issue.number, issue.title);
    render_menu_shell(ui, ctx.theme, &title, |ui| {
        render_list_context_menu(
            ui,
            ctx,
            GhContextTarget::Issue { list_index: row_index },
            &GUI_ISSUE_LIST_ACTIONS,
        );
    });
}

pub fn render_pr_list_context_menu(ui: &mut Ui, ctx: &mut PanelContext<'_>, row_index: usize) {
    let Some(pr) = ctx.state.github.prs.get(row_index) else {
        return;
    };
    let title = format!("#{} · {}", pr.number, pr.title);
    render_menu_shell(ui, ctx.theme, &title, |ui| {
        render_list_context_menu(
            ui,
            ctx,
            GhContextTarget::PullRequest { list_index: row_index },
            &GUI_PR_LIST_ACTIONS,
        );
    });
}

pub fn render_branch_list_context_menu(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    row_index: usize,
) {
    let Some(entry) = ctx.state.branches.entries.get(row_index) else {
        return;
    };
    let title = if entry.is_current {
        format!("* {} (current)", entry.name)
    } else {
        entry.name.clone()
    };
    render_menu_shell(ui, ctx.theme, &title, |ui| {
        for action in GUI_BRANCH_LIST_ACTIONS {
            if menu_action(ui, ctx.theme, action.icon(), action.label()) {
                for command in commands_for_branch_list_action(row_index, action) {
                    let _ = (ctx.dispatch)(command);
                }
                ui.close_menu();
            }
        }
    });
}

fn render_list_context_menu(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    target: GhContextTarget,
    actions: &[GhContextMenuAction],
) {
    for action in actions {
        if menu_action(ui, ctx.theme, github_action_icon(*action), action.label()) {
            for command in commands_for_github_list_action(target, *action) {
                let _ = (ctx.dispatch)(command);
            }
            ui.close_menu();
        }
    }
}

#[cfg(test)]
mod tests {
    use kiwi_core::navigation::{FocusTarget, MainTab, NavCommand};

    use super::*;

    #[test]
    fn new_issue_dispatches_open_command() {
        assert_eq!(
            commands_for_new_issue(),
            vec![AppCommand::GitHubIssueCreateOpen]
        );
    }

    #[test]
    fn view_issue_dispatches_select_and_unpaired_main_focus() {
        let commands = commands_for_github_list_action(
            GhContextTarget::Issue { list_index: 2 },
            GhContextMenuAction::View,
        );
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::GitHubSelectIssue(2)
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(MainTab::Issues))
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Main))
        )));
    }

    #[test]
    fn create_branch_dispatches_list_action() {
        let commands = commands_for_github_list_action(
            GhContextTarget::Issue { list_index: 1 },
            GhContextMenuAction::CreateBranch,
        );
        assert_eq!(
            commands,
            vec![AppCommand::GitHubListAction {
                target: GhContextTarget::Issue { list_index: 1 },
                action: GhContextMenuAction::CreateBranch,
            }]
        );
    }

    #[test]
    fn add_labels_dispatches_list_action() {
        let commands = commands_for_github_list_action(
            GhContextTarget::Issue { list_index: 4 },
            GhContextMenuAction::AddLabels,
        );
        assert_eq!(
            commands,
            vec![AppCommand::GitHubListAction {
                target: GhContextTarget::Issue { list_index: 4 },
                action: GhContextMenuAction::AddLabels,
            }]
        );
    }

    #[test]
    fn comment_dispatches_list_action() {
        let commands = commands_for_github_list_action(
            GhContextTarget::Issue { list_index: 2 },
            GhContextMenuAction::Comment,
        );
        assert_eq!(
            commands,
            vec![AppCommand::GitHubListAction {
                target: GhContextTarget::Issue { list_index: 2 },
                action: GhContextMenuAction::Comment,
            }]
        );
    }

    #[test]
    fn assign_milestone_dispatches_list_action() {
        let commands = commands_for_github_list_action(
            GhContextTarget::Issue { list_index: 1 },
            GhContextMenuAction::AssignMilestone,
        );
        assert_eq!(
            commands,
            vec![AppCommand::GitHubListAction {
                target: GhContextTarget::Issue { list_index: 1 },
                action: GhContextMenuAction::AssignMilestone,
            }]
        );
    }

    #[test]
    fn send_to_agent_dispatches_list_action_for_pr() {
        let commands = commands_for_github_list_action(
            GhContextTarget::PullRequest { list_index: 0 },
            GhContextMenuAction::SendToAgent,
        );
        assert_eq!(
            commands,
            vec![AppCommand::GitHubListAction {
                target: GhContextTarget::PullRequest { list_index: 0 },
                action: GhContextMenuAction::SendToAgent,
            }]
        );
    }

    #[test]
    fn view_branch_dispatches_select_and_unpaired_main_focus() {
        let commands = commands_for_branch_list_action(2, BranchListAction::View);
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::BranchSelect(2)
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(MainTab::Branches))
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Main))
        )));
    }

    #[test]
    fn checkout_branch_dispatches_select_and_checkout() {
        let commands = commands_for_branch_list_action(1, BranchListAction::Checkout);
        assert_eq!(
            commands,
            vec![
                AppCommand::BranchSelect(1),
                AppCommand::BranchCheckoutSelected,
            ]
        );
    }
}
