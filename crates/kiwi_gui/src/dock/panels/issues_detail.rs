//! Issues main dock tab — issue detail body (TUI Issues main pane, SPEC-009 / #191).

use egui::{RichText, Ui};
use kiwi_core::theme::SemanticRole;

use super::github_common::{render_auth_gate, render_detail_lines, sync_github_navigation};
use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    sync_github_navigation(ctx, KiwiTab::Issues);

    ui.label(
        RichText::new(detail_title(ctx))
            .color(ctx.theme.role(SemanticRole::Muted))
            .strong(),
    );
    ui.separator();

    if render_auth_gate(ui, ctx, KiwiTab::Issues) {
        return;
    }

    let mut scroll = ctx.state.github.issue_detail_scroll_offset;
    if ctx.state.github.selected_issue.is_none() {
        render_detail_lines(
            ui,
            ctx,
            &[],
            &mut scroll,
            false,
            None,
            "Select an issue in the GH panel and press Enter",
        );
        ctx.state.github.issue_detail_scroll_offset = scroll;
        render_footer(ui, ctx);
        return;
    }

    let lines = ctx
        .state
        .github
        .issue_detail
        .as_ref()
        .map(|detail| detail.display_lines.clone())
        .unwrap_or_default();
    let loading =
        ctx.state.github.issue_detail_loading && ctx.state.github.issue_detail.is_none();
    let error = ctx.state.github.issue_detail_error.clone();
    render_detail_lines(
        ui,
        ctx,
        &lines,
        &mut scroll,
        loading,
        error.as_deref(),
        "Press Enter on an issue in the GH panel",
    );
    ctx.state.github.issue_detail_scroll_offset = scroll;
    render_footer(ui, ctx);
}

fn detail_title(ctx: &PanelContext<'_>) -> String {
    let mut title = String::from("Issues");
    if ctx.state.github.issue_detail_loading {
        title.push_str(" · loading");
    } else if ctx.state.github.issue_detail_error.is_some() {
        title.push_str(" · error");
    } else if let Some(number) = ctx.state.github.selected_issue {
        title.push_str(&format!(" · #{number}"));
    }
    title
}

fn render_footer(ui: &mut Ui, ctx: &PanelContext<'_>) {
    let text = if ctx.state.github.selected_issue.is_none() {
        "Select an issue in GH · Enter to open"
    } else if ctx.state.github.issue_detail_loading {
        "Loading detail…"
    } else if ctx.state.github.issue_detail.is_some() {
        "↑/↓ scroll · Ctrl+Enter browser · F5 refresh"
    } else {
        "Enter on GH list · F5 refresh"
    };
    ui.add_space(4.0);
    ui.label(RichText::new(text).color(ctx.theme.role(SemanticRole::Muted)));
}
