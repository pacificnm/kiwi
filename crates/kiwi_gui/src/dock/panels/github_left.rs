//! GitHub left dock tab — issue/PR list + hub (TUI GH pane, SPEC-009 / #191).

use egui::{RichText, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::github::{
    issue_selected_row_index, pr_selected_row_index, GitHubLeftPane, IssueState, PrState,
};
use kiwi_core::theme::SemanticRole;

use super::github_common::{
    issue_state_color, pr_state_color, render_auth_gate, select_issue_commands,
    select_pr_commands, sync_github_navigation, LIST_ROW_HEIGHT,
};
use super::layout::render_virtual_rows;
use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    sync_github_navigation(ctx, KiwiTab::GitHubIssues);

    render_hub(ui, ctx);
    ui.separator();

    if render_auth_gate(ui, ctx, KiwiTab::GitHubIssues) {
        return;
    }

    match ctx.state.github.left_pane {
        GitHubLeftPane::Issues => render_issue_list(ui, ctx),
        GitHubLeftPane::Prs => render_pr_list(ui, ctx),
    }

    ui.add_space(4.0);
    render_footer(ui, ctx);
}

fn render_hub(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    ui.horizontal(|ui| {
        hub_button(ui, ctx, GitHubLeftPane::Issues, hub_issues_label(ctx));
        ui.label(RichText::new(" | ").color(ctx.theme.role(SemanticRole::Muted)));
        hub_button(ui, ctx, GitHubLeftPane::Prs, hub_prs_label(ctx));
    });
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
    if ui.add(egui::Label::new(rich).sense(egui::Sense::click())).clicked() {
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
        ctx.state.viewport.github_list_rows = 1;
        return;
    }

    let selected_row = issue_selected_row_index(&ctx.state.github);
    let mut scroll_offset = ctx.state.github.issues_scroll_offset;
    let viewport_rows = render_virtual_rows(
        ui,
        LIST_ROW_HEIGHT,
        total_rows,
        &mut scroll_offset,
        |ui, row_index| {
            render_issue_row(ui, ctx, row_index, selected_row);
        },
    );
    ctx.state.github.issues_scroll_offset = scroll_offset;
    ctx.state.viewport.github_list_rows = viewport_rows;
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
    let viewport_rows = render_virtual_rows(
        ui,
        LIST_ROW_HEIGHT,
        total_rows,
        &mut scroll_offset,
        |ui, row_index| {
            render_pr_row(ui, ctx, row_index, selected_row);
        },
    );
    ctx.state.github.prs_scroll_offset = scroll_offset;
    ctx.state.viewport.github_list_rows = viewport_rows;
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
    let state_color = issue_state_color(ctx.theme, issue.state);
    let title_color = if selected {
        ctx.theme.role(SemanticRole::Accent)
    } else {
        ctx.theme.role(SemanticRole::Fg)
    };
    let prefix = if selected { "▸" } else { " " };
    let number = format!("#{}", issue.number);
    let mut title = issue.title.clone();
    if issue.state == IssueState::Closed {
        title.push_str(" [closed]");
    }

    ui.horizontal(|ui| {
        ui.set_min_height(LIST_ROW_HEIGHT);
        let response = ui
            .horizontal_wrapped(|ui| {
                ui.label(RichText::new(prefix).color(title_color).monospace());
                ui.colored_label(state_color, number);
                ui.add({
                    let mut rich = RichText::new(title).color(title_color);
                    if selected {
                        rich = rich.strong();
                    }
                    egui::Label::new(rich).truncate()
                });
                ui.response()
            })
            .inner
            .interact(egui::Sense::click());
        if response.clicked() {
            for command in select_issue_commands(row_index) {
                let _ = (ctx.dispatch)(command);
            }
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
    let state_color = pr_state_color(ctx.theme, pr.state);
    let title_color = if selected {
        ctx.theme.role(SemanticRole::Accent)
    } else {
        ctx.theme.role(SemanticRole::Fg)
    };
    let prefix = if selected { "▸" } else { " " };
    let number = format!("#{}", pr.number);
    let title = pr.title.clone();
    let badge = if pr.state != PrState::Open {
        Some(format!("[{}]", pr.state.label()))
    } else {
        None
    };

    ui.horizontal(|ui| {
        ui.set_min_height(LIST_ROW_HEIGHT);
        let response = ui
            .horizontal_wrapped(|ui| {
                ui.label(RichText::new(prefix).color(title_color).monospace());
                ui.colored_label(state_color, number);
                ui.add({
                    let mut rich = RichText::new(title).color(title_color);
                    if selected {
                        rich = rich.strong();
                    }
                    egui::Label::new(rich).truncate()
                });
                if let Some(badge) = badge {
                    ui.colored_label(state_color, badge);
                }
                ui.response()
            })
            .inner
            .interact(egui::Sense::click());
        if response.clicked() {
            for command in select_pr_commands(row_index) {
                let _ = (ctx.dispatch)(command);
            }
        }
    });
}

fn render_footer(ui: &mut Ui, ctx: &PanelContext<'_>) {
    let text = match ctx.state.github.left_pane {
        GitHubLeftPane::Issues if ctx.state.github.issues_loading => "Loading issues…",
        GitHubLeftPane::Issues if ctx.state.github.issues.is_empty() => "F5 refresh",
        GitHubLeftPane::Issues => "↑/↓ · Enter · F5",
        GitHubLeftPane::Prs if ctx.state.github.prs_loading => "Loading PRs…",
        GitHubLeftPane::Prs if ctx.state.github.prs.is_empty() => "F5 refresh",
        GitHubLeftPane::Prs => "↑/↓ · Enter · F5",
    };
    ui.label(RichText::new(text).color(ctx.theme.role(SemanticRole::Muted)));
}
