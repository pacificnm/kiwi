//! PRs main dock tab — pull request detail (TUI PRs main pane, SPEC-009 / #191).

use egui::{RichText, Ui};
use kiwi_core::github::PrState;
use kiwi_core::theme::SemanticRole;

use super::github_common::{
    pr_state_color, render_auth_gate, render_detail_lines, sync_github_navigation,
    DETAIL_ROW_HEIGHT,
};
use super::layout::render_virtual_rows;
use crate::dock::context::PanelContext;
use crate::dock::tab::KiwiTab;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    sync_github_navigation(ctx, KiwiTab::GitHubPrs);

    ui.label(
        RichText::new(detail_title(ctx))
            .color(ctx.theme.role(SemanticRole::Muted))
            .strong(),
    );
    ui.separator();

    if render_auth_gate(ui, ctx, KiwiTab::GitHubPrs) {
        return;
    }

    let mut scroll = ctx.state.github.pr_detail_scroll_offset;
    if ctx.state.github.selected_pr.is_none() {
        render_detail_lines(
            ui,
            ctx,
            &[],
            &mut scroll,
            false,
            None,
            "Select a pull request in the GH panel and press Enter",
        );
        ctx.state.github.pr_detail_scroll_offset = scroll;
        render_footer(ui, ctx);
        return;
    }

    let (lines, detail_state) = ctx
        .state
        .github
        .pr_detail
        .as_ref()
        .map(|detail| (detail.display_lines.clone(), Some(detail.state)))
        .unwrap_or_else(|| (Vec::new(), None));
    let loading = ctx.state.github.pr_detail_loading && ctx.state.github.pr_detail.is_none();
    let error = ctx.state.github.pr_detail_error.clone();

    if lines.is_empty() {
        render_detail_lines(
            ui,
            ctx,
            &lines,
            &mut scroll,
            loading,
            error.as_deref(),
            "Press Enter on a PR in the GH panel",
        );
    } else {
        render_pr_detail_lines(ui, ctx, &lines, &mut scroll, detail_state);
    }
    ctx.state.github.pr_detail_scroll_offset = scroll;
    render_footer(ui, ctx);
}

fn render_pr_detail_lines(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    lines: &[String],
    scroll_offset: &mut usize,
    detail_state: Option<PrState>,
) {
    let total_rows = lines.len();
    let mut offset = *scroll_offset;
    let layout = render_virtual_rows(
        ui,
        DETAIL_ROW_HEIGHT,
        total_rows,
        &mut offset,
        |ui, row_index| {
            let text = &lines[row_index];
            let color = if row_index == 1 {
                detail_state
                    .map(|state| pr_state_color(ctx.theme, state))
                    .unwrap_or_else(|| ctx.theme.role(SemanticRole::Fg))
            } else {
                ctx.theme.role(SemanticRole::Fg)
            };
            let rich = if row_index == 0 {
                RichText::new(text).color(color).strong()
            } else {
                RichText::new(text).color(color)
            };
            ui.horizontal(|ui| {
                ui.set_min_height(DETAIL_ROW_HEIGHT);
                ui.label(rich);
            });
        },
    );
    *scroll_offset = offset;
    ctx.state.viewport.github_detail_rows = layout.viewport_rows;
}

fn detail_title(ctx: &PanelContext<'_>) -> String {
    let mut title = String::from("PRs");
    if ctx.state.github.pr_detail_loading {
        title.push_str(" · loading");
    } else if ctx.state.github.pr_detail_error.is_some() {
        title.push_str(" · error");
    } else if let Some(number) = ctx.state.github.selected_pr {
        title.push_str(&format!(" · #{number}"));
    }
    title
}

fn render_footer(ui: &mut Ui, ctx: &PanelContext<'_>) {
    let text = if ctx.state.github.selected_pr.is_none() {
        "Select a PR in GH · Enter to open"
    } else if ctx.state.github.pr_detail_loading {
        "Loading detail…"
    } else if ctx.state.github.pr_detail.is_some() {
        "↑/↓ scroll · Ctrl+Enter browser · F5 refresh"
    } else {
        "Enter on GH list · F5 refresh"
    };
    ui.add_space(4.0);
    ui.label(RichText::new(text).color(ctx.theme.role(SemanticRole::Muted)));
}
