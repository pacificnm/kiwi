//! Shared domain logic for Kiwi TUI and GUI frontends.
//!
//! See [SPEC-024](https://github.com/pacificnm/kiwi/blob/main/docs/specs/SPEC-024-core-library-extraction.md).
//! UI frameworks (`ratatui`, `crossterm`, `egui`, `eframe`) must not appear in this crate.

pub mod agent;
pub mod ansi;
pub mod clipboard;
pub mod commands;
pub mod config;
pub mod diff;
pub mod editor;
pub mod env;
pub mod events;
pub mod file_tree;
pub mod git;
pub mod github;
pub mod navigation;
pub mod plugins;
pub mod preview;
pub mod reducer;
pub mod repo;
pub mod search;
pub mod selection;
pub mod settings;
pub mod shell;
pub mod state;
pub mod status_bar;
pub mod theme;
pub mod util;
pub mod watcher;
pub mod workspace;

/// Crate version string for smoke tests and diagnostics.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_has_version() {
        assert!(!super::CRATE_VERSION.is_empty());
    }
}
