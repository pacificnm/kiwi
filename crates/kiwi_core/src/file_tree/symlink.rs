use std::path::{Path, PathBuf};

pub const MAX_EXPAND_DEPTH: usize = 40;

/// Returns an error message when expanding `target` would exceed depth or form a symlink loop.
#[must_use]
pub fn detect_symlink_loop(root: &Path, target: &Path) -> Option<String> {
    let mut seen = Vec::<PathBuf>::new();
    let mut current = Some(target);
    let mut depth = 0usize;

    while let Some(path) = current {
        if depth > MAX_EXPAND_DEPTH {
            return Some(format!(
                "directory depth exceeds {MAX_EXPAND_DEPTH} under {}",
                root.display()
            ));
        }

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if seen.iter().any(|visited| visited == &canonical) {
            return Some(format!("symlink loop detected at {}", path.display()));
        }
        seen.push(canonical);

        if path == root {
            break;
        }

        current = path.parent();
        depth += 1;
    }

    None
}
