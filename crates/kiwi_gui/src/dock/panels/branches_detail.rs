//! Branches main dock tab — detail for the branch selected in GH left list.

use egui::{RichText, Ui};
use kiwi_core::git::branch_selected_name;
use kiwi_core::theme::SemanticRole;

use super::github_common::{render_detail_lines, sync_github_navigation};
use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    sync_github_navigation(ctx, KiwiTab::GitLog);

    ui.label(
        RichText::new(detail_title(ctx))
            .color(ctx.theme.role(SemanticRole::Muted))
            .strong(),
    );
    ui.separator();

    if !ctx.state.workspace_meta.is_git_repo {
        ui.colored_label(
            ctx.theme.role(SemanticRole::Muted),
            "Not a git repository",
        );
        render_footer(ui, ctx);
        return;
    }

    let mut scroll = ctx.state.branches.detail_scroll_offset;
    if branch_selected_name(&ctx.state.branches).is_none() {
        render_detail_lines(
            ui,
            ctx,
            &[],
            &mut scroll,
            false,
            None,
            "Select a branch in the GH panel and double-click to open",
        );
        ctx.state.branches.detail_scroll_offset = scroll;
        render_footer(ui, ctx);
        return;
    }

    let lines = ctx
        .state
        .branches
        .detail
        .as_ref()
        .map(|detail| detail.display_lines(ctx.state.git.ahead, ctx.state.git.behind))
        .unwrap_or_default();
    let loading = ctx.state.branches.detail_loading && ctx.state.branches.detail.is_none();
    let error = ctx.state.branches.detail_error.clone();
    render_detail_lines(
        ui,
        ctx,
        &lines,
        &mut scroll,
        loading,
        error.as_deref(),
        "Double-click a branch in the GH panel to view details",
    );
    ctx.state.branches.detail_scroll_offset = scroll;
    render_footer(ui, ctx);
}

fn detail_title(ctx: &PanelContext<'_>) -> String {
    let mut title = String::from("Branches");
    if ctx.state.branches.detail_loading {
        title.push_str(" · loading");
    } else if ctx.state.branches.detail_error.is_some() {
        title.push_str(" · error");
    } else if let Some(name) = branch_selected_name(&ctx.state.branches) {
        title.push_str(&format!(" · {name}"));
    }
    title
}

fn render_footer(ui: &mut Ui, ctx: &PanelContext<'_>) {
    let text = if !ctx.state.workspace_meta.is_git_repo {
        "Git features disabled"
    } else if branch_selected_name(&ctx.state.branches).is_none() {
        "Select a branch in GH · double-click to open"
    } else if ctx.state.branches.detail_loading {
        "Loading branch details…"
    } else if ctx.state.branches.checkout_loading {
        "Checking out…"
    } else if let Some(error) = ctx.state.branches.checkout_error.as_deref() {
        error
    } else if ctx.state.branches.detail.is_some() {
        "↑/↓ scroll · Enter checkout · F5 refresh"
    } else {
        "Double-click branch in GH · F5 refresh"
    };
    ui.add_space(4.0);
    let role = if ctx.state.branches.checkout_error.is_some() {
        SemanticRole::AgentError
    } else {
        SemanticRole::Muted
    };
    ui.label(RichText::new(text).color(ctx.theme.role(role)));
}
