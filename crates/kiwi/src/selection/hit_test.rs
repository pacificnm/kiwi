use ratatui::layout::Rect;

use crate::navigation::MainTab;
use crate::shell::ScrollbackBuffer;
use crate::state::AppState;
use crate::ui::agent_scrollback_area;

use super::state::{SelectionPane, TextPosition};

pub fn hit_test_text(
    state: &AppState,
    column: u16,
    row: u16,
) -> Option<(SelectionPane, TextPosition)> {
    let rects = state.layout.rects;

    if point_in_rect(column, row, rects.shell) {
        return hit_test_scrollback(
            rects.shell,
            column,
            row,
            &state.shell.scrollback,
            state.shell.follow_tail,
            state.shell.viewport_offset,
            false,
            SelectionPane::Shell,
        );
    }

    if state.navigation.main_tab == MainTab::Preview
        && point_in_rect(column, row, rects.main_content)
    {
        return hit_test_preview(state, column, row);
    }

    if state.navigation.main_tab == MainTab::Issues
        && point_in_rect(column, row, rects.main_content)
    {
        return hit_test_issue_detail(state, column, row);
    }

    if state.navigation.main_tab == MainTab::Prs && point_in_rect(column, row, rects.main_content) {
        return hit_test_pr_detail(state, column, row);
    }

    if state.navigation.main_tab == MainTab::Agent
        && point_in_rect(column, row, agent_scrollback_area(state))
    {
        return hit_test_scrollback(
            agent_scrollback_area(state),
            column,
            row,
            &state.active_agent().scrollback,
            state.active_agent().follow_tail,
            state.active_agent().viewport_offset,
            state.active_agent().restart_hint.is_some(),
            SelectionPane::Agent,
        );
    }

    None
}

fn hit_test_preview(
    state: &AppState,
    column: u16,
    row: u16,
) -> Option<(SelectionPane, TextPosition)> {
    if state.preview.lines.is_empty() || state.preview.loading && state.preview.lines.is_empty() {
        return None;
    }

    let inner = block_inner(state.layout.rects.main_content);
    let footer_rows = 1_u16;
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(footer_rows),
    };

    if !point_in_rect(column, row, content_area) {
        return None;
    }

    let gutter = if state.config.preview.line_numbers {
        preview_gutter_width(state.preview.line_count()).max(4)
    } else {
        0
    };

    let text_x = content_area.x.saturating_add(gutter as u16);
    if column < text_x {
        return None;
    }

    let viewport_row = usize::from(row.saturating_sub(content_area.y));
    let line = state.preview.scroll_offset.saturating_add(viewport_row);
    if line >= state.preview.lines.len() {
        return None;
    }

    let col = usize::from(column.saturating_sub(text_x));
    let max_col = state.preview.lines[line].chars().count();
    Some((
        SelectionPane::Preview,
        TextPosition {
            line,
            col: col.min(max_col),
        },
    ))
}

fn hit_test_issue_detail(
    state: &AppState,
    column: u16,
    row: u16,
) -> Option<(SelectionPane, TextPosition)> {
    let detail = state.github.issue_detail.as_ref()?;
    if detail.display_lines.is_empty() {
        return None;
    }

    let inner = block_inner(state.layout.rects.main_content);
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(1),
    };

    if !point_in_rect(column, row, content_area) {
        return None;
    }

    let viewport_row = usize::from(row.saturating_sub(content_area.y));
    let line = state
        .github
        .issue_detail_scroll_offset
        .saturating_add(viewport_row);
    if line >= detail.display_lines.len() {
        return None;
    }

    let col = usize::from(column.saturating_sub(content_area.x));
    let max_col = detail.display_lines[line].chars().count();
    Some((
        SelectionPane::IssueDetail,
        TextPosition {
            line,
            col: col.min(max_col),
        },
    ))
}

fn hit_test_pr_detail(
    state: &AppState,
    column: u16,
    row: u16,
) -> Option<(SelectionPane, TextPosition)> {
    let detail = state.github.pr_detail.as_ref()?;
    if detail.display_lines.is_empty() {
        return None;
    }

    let inner = block_inner(state.layout.rects.main_content);
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(1),
    };

    if !point_in_rect(column, row, content_area) {
        return None;
    }

    let viewport_row = usize::from(row.saturating_sub(content_area.y));
    let line = state
        .github
        .pr_detail_scroll_offset
        .saturating_add(viewport_row);
    if line >= detail.display_lines.len() {
        return None;
    }

    let col = usize::from(column.saturating_sub(content_area.x));
    let max_col = detail.display_lines[line].chars().count();
    Some((
        SelectionPane::PrDetail,
        TextPosition {
            line,
            col: col.min(max_col),
        },
    ))
}

#[allow(clippy::too_many_arguments)]
fn hit_test_scrollback(
    area: Rect,
    column: u16,
    row: u16,
    scrollback: &ScrollbackBuffer,
    follow_tail: bool,
    viewport_offset: usize,
    has_footer: bool,
    pane: SelectionPane,
) -> Option<(SelectionPane, TextPosition)> {
    let inner = block_inner(area);
    let footer_rows = u16::from(has_footer);
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(footer_rows),
    };

    if !point_in_rect(column, row, content_area) {
        return None;
    }

    let visible_height = content_area.height as usize;
    let max_width = content_area.width as usize;
    if visible_height == 0 || max_width == 0 {
        return None;
    }

    let viewport_row = usize::from(row.saturating_sub(content_area.y));
    if viewport_row >= visible_height {
        return None;
    }

    let lines = scrollback.viewport_lines(
        scrollback.viewport_start(visible_height, follow_tail, viewport_offset),
        visible_height,
        max_width,
        follow_tail,
    );
    let plain = lines
        .get(viewport_row)
        .map(|text| crate::ansi::strip_ansi(text))?;

    let col = usize::from(column.saturating_sub(content_area.x));
    let max_col = plain.chars().count();

    Some((
        pane,
        TextPosition {
            line: viewport_row,
            col: col.min(max_col),
        },
    ))
}

fn block_inner(area: Rect) -> Rect {
    Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

fn preview_gutter_width(line_count: usize) -> usize {
    let digits = line_count.max(1).ilog10() as usize + 1;
    digits + 1
}

fn point_in_rect(column: u16, row: u16, area: Rect) -> bool {
    column >= area.x
        && column < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::navigation::{MainTab, NavCommand};
    use crate::preview::PreviewState;
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("."),
            false,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        )
    }

    #[test]
    fn hit_test_preview_maps_row_and_column() {
        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));
        state.preview = PreviewState {
            path: Some(PathBuf::from("a.rs")),
            lines: vec!["alpha beta".to_string(); 20],
            scroll_offset: 5,
            ..PreviewState::default()
        };

        let inner = block_inner(state.layout.rects.main_content);
        let gutter = preview_gutter_width(20).max(4);
        let col = inner.x + gutter as u16 + 3;
        let row = inner.y + 2;

        let (pane, pos) = hit_test_text(&state, col, row).expect("hit");
        assert_eq!(pane, SelectionPane::Preview);
        assert_eq!(pos.line, 7);
        assert_eq!(pos.col, 3);
    }

    #[test]
    fn hit_test_issue_detail_maps_row_and_column() {
        use crate::github::{IssueDetail, IssueState};

        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));
        state.github.issue_detail = Some(IssueDetail {
            number: 1,
            title: "Test".to_string(),
            state: IssueState::Open,
            author: "user".to_string(),
            labels: vec![],
            assignees: vec![],
            display_lines: (0..20)
                .map(|i| {
                    if i == 0 {
                        "Issue title".to_string()
                    } else {
                        "Body line".to_string()
                    }
                })
                .collect(),
        });
        state.github.issue_detail_scroll_offset = 3;

        let inner = block_inner(state.layout.rects.main_content);
        let col = inner.x + 4;
        let row = inner.y + 2;

        let (pane, pos) = hit_test_text(&state, col, row).expect("hit");
        assert_eq!(pane, SelectionPane::IssueDetail);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.col, 4);
    }
}
