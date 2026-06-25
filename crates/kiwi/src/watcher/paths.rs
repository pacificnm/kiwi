use std::path::{Component, Path, PathBuf};

pub const DEFAULT_DEBOUNCE_MS: u64 = 300;

pub fn should_ignore_watch_path(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(name) if name == ".git"))
}

pub fn path_matches_file(changed: &Path, target: &Path) -> bool {
    if changed == target {
        return true;
    }

    match (changed.canonicalize(), target.canonicalize()) {
        (Ok(changed), Ok(target)) => changed == target,
        _ => false,
    }
}

pub fn preview_reload_paths(changed_paths: &[PathBuf], preview_path: &Path) -> bool {
    changed_paths
        .iter()
        .any(|changed| path_matches_file(changed, preview_path))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn ignores_git_internal_paths() {
        assert!(should_ignore_watch_path(Path::new("/repo/.git/index")));
        assert!(!should_ignore_watch_path(Path::new("/repo/src/main.rs")));
    }

    #[test]
    fn path_matches_resolves_symlinks() {
        let temp = std::env::temp_dir().join(format!("kiwi-watcher-path-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let file = temp.join("target.txt");
        fs::write(&file, "x").expect("write");

        assert!(path_matches_file(&file, &file));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn preview_reload_detects_matching_path() {
        let preview = PathBuf::from("/tmp/repo/src/lib.rs");
        let changed = vec![PathBuf::from("/tmp/repo/src/lib.rs")];
        assert!(preview_reload_paths(&changed, &preview));
        assert!(!preview_reload_paths(
            &[PathBuf::from("/tmp/repo/src/other.rs")],
            &preview
        ));
    }
}
