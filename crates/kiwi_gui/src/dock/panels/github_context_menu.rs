//! Right-click context menu for GitHub issue/PR list rows (#194).

use egui::Ui;
use kiwi_core::events::AppCommand;
use kiwi_core::github::{GhContextMenuAction, GhContextTarget};

const GUI_ISSUE_LIST_ACTIONS: [GhContextMenuAction; 3] = [
    GhContextMenuAction::View,
    GhContextMenuAction::CreateBranch,
    GhContextMenuAction::SendToAgent,
];

const GUI_PR_LIST_ACTIONS: [GhContextMenuAction; 2] = [
    GhContextMenuAction::View,
    GhContextMenuAction::SendToAgent,
];

use super::github_common::{select_issue_commands, select_pr_commands};
use crate::dock::context::PanelContext;

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

pub fn render_issue_list_context_menu(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    row_index: usize,
) {
    render_list_context_menu(
        ui,
        ctx,
        GhContextTarget::Issue { list_index: row_index },
        &GUI_ISSUE_LIST_ACTIONS,
    );
}

pub fn render_pr_list_context_menu(ui: &mut Ui, ctx: &mut PanelContext<'_>, row_index: usize) {
    render_list_context_menu(
        ui,
        ctx,
        GhContextTarget::PullRequest { list_index: row_index },
        &GUI_PR_LIST_ACTIONS,
    );
}

fn render_list_context_menu(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    target: GhContextTarget,
    actions: &[GhContextMenuAction],
) {
    for action in actions {
        if ui.button(action.label()).clicked() {
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
}
