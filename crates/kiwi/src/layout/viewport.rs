//! Precomputed viewport metrics derived from TUI layout rects.

use kiwi_core::state::ViewportMetrics;

use super::{agent_pty_size, shell_pty_size, LayoutRects};

const GH_HUB_ROWS: u16 = 1;
const ISSUES_STATUS_ROWS: u16 = 1;
const ISSUE_DETAIL_STATUS_ROWS: u16 = 1;
const GIT_BRANCH_ROWS: u16 = 1;
const GIT_STATUS_ROWS: u16 = 1;
const SETTINGS_STATUS_ROWS: u16 = 1;
const SETTINGS_HEADER_ROWS: u16 = 1;

fn pane_inner_height(height: u16) -> u16 {
    height.saturating_sub(2)
}

fn settings_list_rows(area_height: u16) -> usize {
    pane_inner_height(area_height).saturating_sub(SETTINGS_STATUS_ROWS + SETTINGS_HEADER_ROWS)
        as usize
}

fn github_list_rows(area_height: u16) -> usize {
    pane_inner_height(area_height).saturating_sub(ISSUES_STATUS_ROWS + GH_HUB_ROWS) as usize
}

fn github_detail_rows(area_height: u16) -> usize {
    pane_inner_height(area_height).saturating_sub(ISSUE_DETAIL_STATUS_ROWS) as usize
}

fn git_rows(area_height: u16) -> usize {
    let inner = pane_inner_height(area_height);
    inner
        .saturating_sub(GIT_BRANCH_ROWS)
        .saturating_sub(if inner > GIT_BRANCH_ROWS {
            GIT_STATUS_ROWS
        } else {
            0
        }) as usize
}

#[must_use]
pub fn viewport_metrics_from_layout(rects: &LayoutRects) -> ViewportMetrics {
    let (shell_cols, shell_rows) = shell_pty_size(rects);
    let (agent_cols, agent_rows) = agent_pty_size(rects);

    ViewportMetrics {
        settings_rows: settings_list_rows(rects.main_content.height),
        github_list_rows: github_list_rows(rects.left_content.height),
        github_detail_rows: github_detail_rows(rects.main_content.height),
        branches_rows: settings_list_rows(rects.main_content.height),
        git_rows: git_rows(rects.left_content.height),
        file_tree_rows: rects.left_content.height.saturating_sub(2) as usize,
        preview_rows: rects.main_content.height.saturating_sub(4) as usize,
        preview_cols: rects.main_content.width.saturating_sub(4) as usize,
        search_rows: rects.left_content.height.saturating_sub(4) as usize,
        shell_rows,
        shell_cols,
        agent_rows,
        agent_cols,
    }
}
