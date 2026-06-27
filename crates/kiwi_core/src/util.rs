use std::collections::HashSet;
use std::path::PathBuf;

/// Deduplicates changed paths before emitting `FsChanged`.
#[must_use]
pub fn coalesce_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}
