//! Precomputed row/column counts for list panes (updated when terminal layout changes).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ViewportMetrics {
    /// Settings list rows in the main pane.
    pub settings_rows: usize,
    /// Issue/PR list rows in the left GH pane.
    pub github_list_rows: usize,
    /// Issue/PR detail rows in the main pane.
    pub github_detail_rows: usize,
    /// Branch list rows in the main pane.
    pub branches_rows: usize,
    /// Git changed-files rows in the left pane.
    pub git_rows: usize,
    /// File tree rows in the left pane.
    pub file_tree_rows: usize,
    /// Preview content rows in the main pane.
    pub preview_rows: usize,
    /// Preview content columns (for horizontal diff-style scroll).
    pub preview_cols: usize,
    /// Search results rows in the left pane.
    pub search_rows: usize,
    /// Shell PTY rows.
    pub shell_rows: u16,
    /// Shell PTY columns.
    pub shell_cols: u16,
    /// Active agent PTY rows.
    pub agent_rows: u16,
    /// Active agent PTY columns.
    pub agent_cols: u16,
}
