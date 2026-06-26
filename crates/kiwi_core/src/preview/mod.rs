use std::time::SystemTime;

mod state;

pub use state::PreviewState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewLoadResult {
    pub lines: Vec<String>,
    pub truncated: bool,
    pub oversize: bool,
    pub binary: bool,
    pub lossy_utf8: bool,
    pub file_size: u64,
    pub modified_at: Option<SystemTime>,
    pub error: Option<String>,
}

impl PreviewLoadResult {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            lines: Vec::new(),
            truncated: false,
            oversize: false,
            binary: false,
            lossy_utf8: false,
            file_size: 0,
            modified_at: None,
            error: Some(message.into()),
        }
    }
}
