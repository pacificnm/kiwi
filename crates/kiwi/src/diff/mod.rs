//! TUI-facing diff API; domain logic lives in [`kiwi_core::diff`].

#[allow(unused_imports)]
pub use kiwi_core::diff::{
    spawn_file_diff_load, DiffLine, DiffLineKind, DiffSource, FileDiffLoadResult,
};
