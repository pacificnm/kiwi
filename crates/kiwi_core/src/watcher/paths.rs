use std::path::{Component, Path, PathBuf};

use notify::EventKind;

const GIT_METADATA_FILES: &[&str] = &["HEAD", "index"];

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

/// Returns true for `.git/HEAD` and `.git/index`, which drive branch/status refresh per ADR-011.
pub fn is_git_metadata_watch_path(path: &Path) -> bool {
    let mut components = path.components();
    while let Some(component) = components.next() {
        if !matches!(component, Component::Normal(name) if name == ".git") {
            continue;
        }

        return matches!(components.next(), Some(Component::Normal(name))
            if name.to_str().is_some_and(|value| GIT_METADATA_FILES.contains(&value)));
    }

    false
}

/// Returns true when a notify event can change filesystem content visible to Kiwi.
///
/// Access events (read/open/close) are excluded so preview loads do not retrigger reloads.
pub fn should_emit_fs_changed_event(kind: &EventKind) -> bool {
    !matches!(kind, EventKind::Access(_) | EventKind::Other)
}

pub fn should_ignore_watch_path(path: &Path) -> bool {
    if is_git_metadata_watch_path(path) {
        return false;
    }

    path.components()
        .any(|component| matches!(component, Component::Normal(name) if name == ".git"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use notify::event::{AccessKind, AccessMode, CreateKind, ModifyKind};

    use super::*;

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

    #[test]
    fn ignores_access_events() {
        assert!(!should_emit_fs_changed_event(&EventKind::Access(
            AccessKind::Read
        )));
        assert!(!should_emit_fs_changed_event(&EventKind::Access(
            AccessKind::Open(AccessMode::Read)
        )));
        assert!(should_emit_fs_changed_event(&EventKind::Modify(
            ModifyKind::Any
        )));
        assert!(should_emit_fs_changed_event(&EventKind::Create(
            CreateKind::File
        )));
    }

    #[test]
    fn allows_git_metadata_paths() {
        assert!(!should_ignore_watch_path(Path::new("/repo/.git/HEAD")));
        assert!(!should_ignore_watch_path(Path::new("/repo/.git/index")));
        assert!(is_git_metadata_watch_path(Path::new("/repo/.git/HEAD")));
    }

    #[test]
    fn ignores_other_git_internal_paths() {
        assert!(should_ignore_watch_path(Path::new(
            "/repo/.git/objects/pack/foo"
        )));
        assert!(should_ignore_watch_path(Path::new("/repo/.git/logs/HEAD")));
        assert!(should_ignore_watch_path(Path::new("/repo/.git/index.lock")));
        assert!(!should_ignore_watch_path(Path::new("/repo/src/main.rs")));
    }
}
