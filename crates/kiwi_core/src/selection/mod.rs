/// Row-relative scroll offset that keeps `selected_row` visible in `viewport_rows`.
/// Used by git, GitHub, and branch selection panels.
pub fn scroll_offset_for_row(
    selected_row: usize,
    scroll_offset: usize,
    viewport_rows: usize,
) -> usize {
    if viewport_rows == 0 {
        return 0;
    }
    if selected_row < scroll_offset {
        selected_row
    } else if selected_row >= scroll_offset.saturating_add(viewport_rows) {
        selected_row.saturating_sub(viewport_rows.saturating_sub(1))
    } else {
        scroll_offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionPane {
    Preview,
    IssueDetail,
    PrDetail,
    Agent,
    Shell,
}
