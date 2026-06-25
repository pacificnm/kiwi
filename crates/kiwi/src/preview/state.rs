use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreviewState {
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub scroll_offset: usize,
    pub cursor_line: usize,
    pub truncated: bool,
    pub oversize: bool,
    pub binary: bool,
    pub lossy_utf8: bool,
    pub file_size: u64,
    pub load_error: Option<String>,
    pub loading: bool,
    pub preserve_scroll_on_load: bool,
    pub goto_line: Option<u32>,
}

impl PreviewState {
    pub fn begin_load(&mut self, path: PathBuf, line: Option<u32>) {
        self.path = Some(path);
        self.lines.clear();
        self.scroll_offset = 0;
        self.cursor_line = 0;
        self.truncated = false;
        self.oversize = false;
        self.binary = false;
        self.lossy_utf8 = false;
        self.file_size = 0;
        self.load_error = None;
        self.loading = true;
        self.preserve_scroll_on_load = false;
        self.goto_line = line;
    }

    pub fn begin_reload(&mut self) {
        self.load_error = None;
        self.loading = true;
        self.preserve_scroll_on_load = true;
        self.goto_line = None;
    }

    pub fn apply_loaded(
        &mut self,
        path: PathBuf,
        result: super::loader::PreviewLoadResult,
        viewport_rows: usize,
    ) {
        let preserve_scroll = self.preserve_scroll_on_load;
        let previous_scroll = self.scroll_offset;
        self.loading = false;
        self.path = Some(path);
        self.lines = result.lines;
        self.truncated = result.truncated;
        self.oversize = result.oversize;
        self.binary = result.binary;
        self.lossy_utf8 = result.lossy_utf8;
        self.file_size = result.file_size;
        self.load_error = result.error;
        if preserve_scroll {
            let max_offset = self.lines.len().saturating_sub(viewport_rows.max(1));
            self.scroll_offset = previous_scroll.min(max_offset);
        } else if let Some(line) = self.goto_line.take() {
            let line_index = line.saturating_sub(1) as usize;
            let max_offset = self.lines.len().saturating_sub(viewport_rows.max(1));
            self.scroll_offset = line_index.min(max_offset);
        } else {
            self.scroll_offset = 0;
        }
        self.cursor_line = self.scroll_offset;
        self.preserve_scroll_on_load = false;
    }

    pub fn scroll(&mut self, delta: i32, viewport_rows: usize) {
        if viewport_rows == 0 {
            return;
        }

        let max_offset = self.lines.len().saturating_sub(viewport_rows);
        let current = self.scroll_offset as i32;
        let next = (current + delta).clamp(0, max_offset as i32);
        self.scroll_offset = usize::try_from(next).unwrap_or(0);
        self.cursor_line = self.scroll_offset;
    }

    pub fn page_scroll(&mut self, delta: i32, viewport_rows: usize) {
        if viewport_rows == 0 {
            return;
        }
        let page = i32::try_from(viewport_rows.saturating_sub(1).max(1)).unwrap_or(1);
        self.scroll(delta * page, viewport_rows);
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    #[must_use]
    pub fn path_display(&self) -> Option<String> {
        self.path.as_ref().map(|path| path.display().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_loaded_scrolls_to_requested_line() {
        let mut state = PreviewState::default();
        state.begin_load(PathBuf::from("/tmp/a.rs"), Some(42));
        state.apply_loaded(
            PathBuf::from("/tmp/a.rs"),
            super::super::loader::PreviewLoadResult {
                lines: (0..100).map(|index| format!("line {index}")).collect(),
                truncated: false,
                oversize: false,
                binary: false,
                lossy_utf8: false,
                file_size: 1,
                error: None,
            },
            10,
        );
        assert_eq!(state.scroll_offset, 41);
    }

    #[test]
    fn apply_loaded_preserves_scroll_when_requested() {
        let mut state = PreviewState {
            lines: (0..100).map(|index| format!("line {index}")).collect(),
            scroll_offset: 40,
            preserve_scroll_on_load: true,
            ..PreviewState::default()
        };
        state.apply_loaded(
            PathBuf::from("/tmp/a.rs"),
            super::super::loader::PreviewLoadResult {
                lines: (0..120).map(|index| format!("line {index}")).collect(),
                truncated: false,
                oversize: false,
                binary: false,
                lossy_utf8: false,
                file_size: 1,
                error: None,
            },
            10,
        );
        assert_eq!(state.scroll_offset, 40);
    }

    #[test]
    fn scroll_clamps_to_visible_range() {
        let mut state = PreviewState {
            lines: (0..100).map(|index| format!("line {index}")).collect(),
            ..PreviewState::default()
        };
        state.scroll(100, 10);
        assert_eq!(state.scroll_offset, 90);
        state.scroll(-100, 10);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn page_scroll_moves_by_viewport() {
        let mut state = PreviewState {
            lines: (0..100).map(|index| format!("line {index}")).collect(),
            ..PreviewState::default()
        };
        state.page_scroll(1, 10);
        assert_eq!(state.scroll_offset, 9);
    }
}
