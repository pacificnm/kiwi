//! Shared domain logic for Kiwi TUI and GUI frontends.
//!
//! See [SPEC-024](https://github.com/pacificnm/kiwi/blob/main/docs/specs/SPEC-024-core-library-extraction.md).
//! UI frameworks (`ratatui`, `crossterm`, `egui`, `eframe`) must not appear in this crate.

pub mod config;
pub mod theme;

/// Crate version string for smoke tests and diagnostics.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_has_version() {
        assert!(!super::CRATE_VERSION.is_empty());
    }
}
