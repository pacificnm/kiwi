//! GitHub left dock tab — issue/PR list + hub (TUI GH pane, SPEC-009 / #191).

use egui::{RichText, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::github::{
    issue_selected_row_index, pr_selected_row_index, GitHubLeftPane, IssueState, PrState,
};
use kiwi_core::git::branch_selected_row_index;
use kiwi_core::theme::SemanticRole;

use super::github_common::{
    branch_list_click_commands, issue_list_click_commands, issue_state_color,
    pr_list_click_commands, pr_state_color, render_auth_gate, sync_github_navigation,
    LIST_ROW_HEIGHT,
};
use super::github_context_menu::{
    render_branch_list_context_menu, render_gh_issues_panel_context_menu,
    render_issue_list_context_menu, render_pr_list_context_menu,
};
use super::layout::{render_virtual_rows, selectable_label, selectable_label_truncate};
use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    sync_github_navigation(ctx, KiwiTab::GitHubIssues);

    render_hub(ui, ctx);
    ui.separator();

    match ctx.state.github.left_pane {
        GitHubLeftPane::Issues => {
            if render_auth_gate(ui, ctx, KiwiTab::GitHubIssues) {
                return;
            }
            render_issue_list(ui, ctx);
        }
        GitHubLeftPane::Prs => {
            if render_auth_gate(ui, ctx, KiwiTab::GitHubIssues) {
                return;
            }
            render_pr_list(ui, ctx);
        }
        GitHubLeftPane::Branches => render_branch_list(ui, ctx),
    }

    ui.add_space(4.0);
    render_footer(ui, ctx);
}

fn render_hub(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let response = ui.horizontal(|ui| {
        hub_button(ui, ctx, GitHubLeftPane::Issues, hub_issues_label(ctx));
        ui.label(RichText::new(" | ").color(ctx.theme.role(SemanticRole::Muted)));
        hub_button(ui, ctx, GitHubLeftPane::Prs, hub_prs_label(ctx));
        ui.label(RichText::new(" | ").color(ctx.theme.role(SemanticRole::Muted)));
        hub_button(ui, ctx, GitHubLeftPane::Branches, hub_branches_label(ctx));
    });
    if ctx.state.github.left_pane == GitHubLeftPane::Issues && ctx.state.github.auth_ok {
        response.response.context_menu(|ui| {
            render_gh_issues_panel_context_menu(ui, ctx);
        });
    }
}

fn hub_branches_label(ctx: &PanelContext<'_>) -> String {
    let mut label = String::from("Branches");
    if ctx.state.branches.loading {
        label.push('…');
    } else if !ctx.state.branches.entries.is_empty() {
        label.push_str(&format!(" · {}", ctx.state.branches.entries.len()));
    }
    label
}

fn hub_issues_label(ctx: &PanelContext<'_>) -> String {
    let mut label = String::from("Issues");
    if ctx.state.github.issues_loading {
        label.push('…');
    } else if !ctx.state.github.issues.is_empty() {
        label.push_str(&format!(" · {}", ctx.state.github.issues.len()));
    }
    label
}

fn hub_prs_label(ctx: &PanelContext<'_>) -> String {
    let mut label = String::from("PRs");
    if ctx.state.github.prs_loading {
        label.push('…');
    } else if !ctx.state.github.prs.is_empty() {
        label.push_str(&format!(" · {}", ctx.state.github.prs.len()));
    }
    label
}

fn hub_button(ui: &mut Ui, ctx: &mut PanelContext<'_>, pane: GitHubLeftPane, label: String) {
    let selected = ctx.state.github.left_pane == pane;
    let color = if selected {
        ctx.theme.role(SemanticRole::Accent)
    } else {
        ctx.theme.role(SemanticRole::Muted)
    };
    let rich = if selected {
        RichText::new(label).color(color).strong().underline()
    } else {
        RichText::new(label).color(color)
    };
    if ui
        .add(
            egui::Label::new(rich)
                .sense(egui::Sense::click()),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        let _ = (ctx.dispatch)(AppCommand::GitHubSelectLeftPane(pane));
    }
}

fn render_issue_list(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    if let Some(error) = ctx.state.github.issues_error.as_deref() {
        ui.colored_label(ctx.theme.role(SemanticRole::AgentError), error);
        ctx.state.viewport.github_list_rows = 1;
        return;
    }

    let total_rows = ctx.state.github.issues.len();
    if total_rows == 0 {
        let hint = if ctx.state.github.issues_loading {
            "Loading issues…"
        } else {
            "No open issues"
        };
        ui.label(RichText::new(hint).color(ctx.theme.role(SemanticRole::Muted)));
        ui.allocate_response(ui.available_size(), egui::Sense::empty())
            .context_menu(|ui| {
                render_gh_issues_panel_context_menu(ui, ctx);
            });
        ctx.state.viewport.github_list_rows = 1;
        return;
    }

    let selected_row = issue_selected_row_index(&ctx.state.github);
    let mut scroll_offset = ctx.state.github.issues_scroll_offset;
    let layout = render_virtual_rows(
        ui,
        LIST_ROW_HEIGHT,
        total_rows,
        &mut scroll_offset,
        |ui, row_index| {
            render_issue_row(ui, ctx, row_index, selected_row);
        },
    );
    ctx.state.github.issues_scroll_offset = scroll_offset;
    ctx.state.viewport.github_list_rows = layout.viewport_rows;
}

fn render_branch_list(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    if !ctx.state.workspace_meta.is_git_repo {
        ui.colored_label(
            ctx.theme.role(SemanticRole::Muted),
            "Not a git repository",
        );
        ctx.state.viewport.branches_rows = 1;
        return;
    }

    if let Some(error) = ctx.state.branches.error.as_deref() {
        ui.colored_label(ctx.theme.role(SemanticRole::AgentError), error);
        ctx.state.viewport.branches_rows = 1;
        return;
    }

    let total_rows = ctx.state.branches.entries.len();
    if total_rows == 0 {
        let hint = if ctx.state.branches.loading {
            "Loading branches…"
        } else {
            "No local branches"
        };
        ui.label(RichText::new(hint).color(ctx.theme.role(SemanticRole::Muted)));
        ctx.state.viewport.branches_rows = 1;
        return;
    }

    let selected_row = branch_selected_row_index(&ctx.state.branches);
    let mut scroll_offset = ctx.state.branches.scroll_offset;
    let layout = render_virtual_rows(
        ui,
        LIST_ROW_HEIGHT,
        total_rows,
        &mut scroll_offset,
        |ui, row_index| {
            render_branch_row(ui, ctx, row_index, selected_row);
        },
    );
    ctx.state.branches.scroll_offset = scroll_offset;
    ctx.state.viewport.branches_rows = layout.viewport_rows;
}

fn render_branch_row(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    row_index: usize,
    selected_row: Option<usize>,
) {
    let Some(entry) = ctx.state.branches.entries.get(row_index) else {
        return;
    };
    let is_current = entry.is_current;
    let branch_name = entry.name.clone();
    let selected = selected_row == Some(row_index);
    let marker = if is_current {
        "*"
    } else if selected {
        "▸"
    } else {
        " "
    };
    let color = if is_current || selected {
        ctx.theme.role(SemanticRole::Accent)
    } else {
        ctx.theme.role(SemanticRole::Fg)
    };
    let label = format!("{marker} {branch_name}");

    ui.horizontal(|ui| {
        ui.set_min_height(LIST_ROW_HEIGHT);
        let mut rich = RichText::new(label).color(color).monospace();
        if selected || is_current {
            rich = rich.strong();
        }
        let response = selectable_label(ui, rich);
        if response.clicked() {
            for command in branch_list_click_commands(row_index, false) {
                let _ = (ctx.dispatch)(command);
            }
        }
        if response.double_clicked() {
            for command in branch_list_click_commands(row_index, true) {
                let _ = (ctx.dispatch)(command);
            }
        }
        response.context_menu(|ui| {
            render_branch_list_context_menu(ui, ctx, row_index);
        });
    });
}

fn render_pr_list(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    if let Some(error) = ctx.state.github.prs_error.as_deref() {
        ui.colored_label(ctx.theme.role(SemanticRole::AgentError), error);
        ctx.state.viewport.github_list_rows = 1;
        return;
    }

    let total_rows = ctx.state.github.prs.len();
    if total_rows == 0 {
        let hint = if ctx.state.github.prs_loading {
            "Loading pull requests…"
        } else {
            "No pull requests"
        };
        ui.label(RichText::new(hint).color(ctx.theme.role(SemanticRole::Muted)));
        ctx.state.viewport.github_list_rows = 1;
        return;
    }

    let selected_row = pr_selected_row_index(&ctx.state.github);
    let mut scroll_offset = ctx.state.github.prs_scroll_offset;
    let layout = render_virtual_rows(
        ui,
        LIST_ROW_HEIGHT,
        total_rows,
        &mut scroll_offset,
        |ui, row_index| {
            render_pr_row(ui, ctx, row_index, selected_row);
        },
    );
    ctx.state.github.prs_scroll_offset = scroll_offset;
    ctx.state.viewport.github_list_rows = layout.viewport_rows;
}

fn render_issue_row(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    row_index: usize,
    selected_row: Option<usize>,
) {
    let Some(issue) = ctx.state.github.issues.get(row_index) else {
        return;
    };
    let selected = selected_row == Some(row_index);
    let title_color = if selected {
        ctx.theme.role(SemanticRole::Accent)
    } else {
        issue_state_color(ctx.theme, issue.state)
    };
    let prefix = if selected { "▸ " } else { "  " };
    let number = format!("#{}", issue.number);
    let mut title = issue.title.clone();
    if issue.state == IssueState::Closed {
        title.push_str(" [closed]");
    }
    let label = format!("{prefix}{number} {title}");

    ui.horizontal(|ui| {
        ui.set_min_height(LIST_ROW_HEIGHT);
        let mut rich = RichText::new(label).color(title_color).monospace();
        if selected {
            rich = rich.strong();
        }
        let response = selectable_label_truncate(ui, rich);
        if response.clicked() {
            for command in issue_list_click_commands(row_index, false) {
                let _ = (ctx.dispatch)(command);
            }
        }
        if response.double_clicked() {
            for command in issue_list_click_commands(row_index, true) {
                let _ = (ctx.dispatch)(command);
            }
        }
        if ctx.state.github.auth_ok {
            response.context_menu(|ui| {
                render_issue_list_context_menu(ui, ctx, row_index);
            });
        }
    });
}

fn render_pr_row(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    row_index: usize,
    selected_row: Option<usize>,
) {
    let Some(pr) = ctx.state.github.prs.get(row_index) else {
        return;
    };
    let selected = selected_row == Some(row_index);
    let title_color = if selected {
        ctx.theme.role(SemanticRole::Accent)
    } else {
        pr_state_color(ctx.theme, pr.state)
    };
    let prefix = if selected { "▸ " } else { "  " };
    let number = format!("#{}", pr.number);
    let mut label = format!("{prefix}{number} {}", pr.title);
    if pr.state != PrState::Open {
        label.push_str(&format!(" [{}]", pr.state.label()));
    }

    ui.horizontal(|ui| {
        ui.set_min_height(LIST_ROW_HEIGHT);
        let mut rich = RichText::new(label).color(title_color).monospace();
        if selected {
            rich = rich.strong();
        }
        let response = selectable_label_truncate(ui, rich);
        if response.clicked() {
            for command in pr_list_click_commands(row_index, false) {
                let _ = (ctx.dispatch)(command);
            }
        }
        if response.double_clicked() {
            for command in pr_list_click_commands(row_index, true) {
                let _ = (ctx.dispatch)(command);
            }
        }
        if ctx.state.github.auth_ok {
            response.context_menu(|ui| {
                render_pr_list_context_menu(ui, ctx, row_index);
            });
        }
    });
}

fn render_footer(ui: &mut Ui, ctx: &PanelContext<'_>) {
    let text = match ctx.state.github.left_pane {
        GitHubLeftPane::Issues if ctx.state.github.issues_loading => "Loading issues…",
        GitHubLeftPane::Issues if ctx.state.github.issues.is_empty() => "F5 refresh",
        GitHubLeftPane::Issues => "↑/↓ · Enter/double-click · right-click · F5",
        GitHubLeftPane::Branches if ctx.state.branches.loading => "Loading branches…",
        GitHubLeftPane::Branches if ctx.state.branches.entries.is_empty() => "F5 refresh",
        GitHubLeftPane::Branches => "↑/↓ · Enter/double-click · right-click · F5",
        GitHubLeftPane::Prs if ctx.state.github.prs_loading => "Loading PRs…",
        GitHubLeftPane::Prs if ctx.state.github.prs.is_empty() => "F5 refresh",
        GitHubLeftPane::Prs => "↑/↓ · Enter/double-click · right-click · F5",
    };
    ui.label(RichText::new(text).color(ctx.theme.role(SemanticRole::Muted)));
}
