use crate::search::{SearchMode, SearchResult};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchState {
    pub mode: SearchMode,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub running: bool,
    pub error: Option<String>,
    pub generation: u64,
    pub debounce_scheduled: bool,
    pub truncated: bool,
    pub scroll_offset: usize,
}

impl SearchState {
    pub fn schedule_query(&mut self, query: String) {
        self.query = query;
        self.generation = self.generation.saturating_add(1);
        self.running = false;
        self.error = None;
        self.debounce_scheduled = true;
        self.truncated = false;
    }

    pub fn cancel(&mut self) {
        self.generation = self.generation.saturating_add(1);
        self.running = false;
        self.debounce_scheduled = false;
    }

    pub fn begin_execute(&mut self) -> u64 {
        self.debounce_scheduled = false;
        if self.query.is_empty() {
            self.results.clear();
            self.selected = 0;
            self.running = false;
            self.error = None;
            self.truncated = false;
            return self.generation;
        }

        self.running = true;
        self.error = None;
        self.truncated = false;
        self.generation
    }

    pub fn apply_results(&mut self, generation: u64, results: Vec<SearchResult>, truncated: bool) {
        if generation != self.generation {
            return;
        }

        self.results = results;
        self.selected = 0;
        self.scroll_offset = 0;
        self.running = false;
        self.truncated = truncated;
    }

    pub fn apply_error(&mut self, generation: u64, message: String) {
        if generation != self.generation {
            return;
        }

        self.error = Some(message);
        self.results.clear();
        self.selected = 0;
        self.running = false;
    }

    pub fn clear_query(&mut self) {
        self.query.clear();
        self.results.clear();
        self.selected = 0;
        self.error = None;
        self.truncated = false;
        self.debounce_scheduled = false;
        self.generation = self.generation.saturating_add(1);
        self.running = false;
    }

    pub fn set_mode(&mut self, mode: SearchMode) {
        if self.mode == mode {
            return;
        }

        self.mode = mode;
        self.results.clear();
        self.selected = 0;
        self.error = None;
        self.truncated = false;
        self.debounce_scheduled = !self.query.is_empty();
        self.generation = self.generation.saturating_add(1);
        self.running = false;
    }

    pub fn move_selection(&mut self, delta: i32, viewport_rows: usize) {
        if self.results.is_empty() || viewport_rows == 0 {
            return;
        }

        let len = self.results.len();
        let current = self.selected as i32;
        let next = (current + delta).clamp(0, len.saturating_sub(1) as i32);
        self.select_index(usize::try_from(next).unwrap_or(0), viewport_rows);
    }

    pub fn select_index(&mut self, index: usize, viewport_rows: usize) {
        if index >= self.results.len() {
            return;
        }

        self.selected = index;
        self.scroll_offset =
            scroll_offset_for_row(self.selected, self.scroll_offset, viewport_rows);
    }

    pub fn result_at_viewport(&self, viewport_index: usize) -> Option<&SearchResult> {
        self.results
            .get(self.scroll_offset.saturating_add(viewport_index))
    }
}

fn scroll_offset_for_row(selected: usize, scroll_offset: usize, viewport_rows: usize) -> usize {
    if viewport_rows == 0 {
        return 0;
    }

    if selected < scroll_offset {
        selected
    } else if selected >= scroll_offset.saturating_add(viewport_rows) {
        selected.saturating_sub(viewport_rows.saturating_sub(1))
    } else {
        scroll_offset
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::search::SearchResult;

    #[test]
    fn schedule_query_bumps_generation_and_arms_debounce() {
        let mut state = SearchState::default();
        state.schedule_query("main".to_string());
        assert_eq!(state.generation, 1);
        assert!(state.debounce_scheduled);
        assert!(!state.running);
    }

    #[test]
    fn apply_results_ignores_stale_generation() {
        let mut state = SearchState::default();
        state.generation = 2;
        state.apply_results(
            1,
            vec![SearchResult::file(
                PathBuf::from("a.rs"),
                "a.rs".to_string(),
            )],
            false,
        );
        assert!(state.results.is_empty());
    }

    #[test]
    fn cancel_invalidates_in_flight_search() {
        let mut state = SearchState::default();
        state.running = true;
        state.debounce_scheduled = true;
        let before = state.generation;
        state.cancel();
        assert_eq!(state.generation, before + 1);
        assert!(!state.running);
        assert!(!state.debounce_scheduled);
    }

    #[test]
    fn move_selection_scrolls_to_keep_selected_visible() {
        let mut state = SearchState {
            results: (0..20)
                .map(|index| {
                    SearchResult::file(
                        PathBuf::from(format!("f{index}.rs")),
                        format!("f{index}.rs"),
                    )
                })
                .collect(),
            ..SearchState::default()
        };
        state.move_selection(10, 5);
        assert_eq!(state.selected, 10);
        assert!(state.scroll_offset >= 6);
    }
}
