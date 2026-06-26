//! TUI-facing search API; domain logic lives in [`kiwi_core::search`].

#[allow(unused_imports)]
pub use kiwi_core::search::{
    spawn_search, DebounceTimer, SearchCancelHandle, SearchJob, SearchMode, SearchResult,
    SearchState, MAX_SEARCH_RESULTS,
};
