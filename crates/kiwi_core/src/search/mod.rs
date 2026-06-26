use std::path::PathBuf;

mod cancel;
mod content;
mod debounce;
mod file;
mod io;
mod state;

pub use cancel::SearchCancelHandle;
pub use content::ContentSearchError;
pub use debounce::DebounceTimer;
pub use io::{spawn_search, SearchJob};
pub use state::SearchState;

pub const MAX_SEARCH_RESULTS: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    #[default]
    Files,
    Content,
}

impl SearchMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Content => "Content",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub id: String,
    pub path: PathBuf,
    pub line: Option<u32>,
    pub preview: String,
}

impl SearchResult {
    #[must_use]
    pub fn file(path: PathBuf, relative: String) -> Self {
        let id = relative.clone();
        Self {
            id,
            path,
            line: None,
            preview: String::new(),
        }
    }

    #[must_use]
    pub fn content(path: PathBuf, line: u32, preview: String) -> Self {
        let id = format!("{}:{line}", path.display());
        Self {
            id,
            path,
            line: Some(line),
            preview,
        }
    }
}
