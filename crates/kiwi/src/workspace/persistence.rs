#![allow(dead_code)] // Public persistence API (SPEC-017); wired in #65–#66.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::snapshot::{trim_history, WorkspaceSnapshot, WORKSPACE_SCHEMA_VERSION};

pub fn load_snapshot(repo_root: &Path) -> Option<WorkspaceSnapshot> {
    let path = workspace_file_path(repo_root);
    let contents = fs::read_to_string(path).ok()?;
    let snapshot: WorkspaceSnapshot = serde_json::from_str(&contents).ok()?;
    if !snapshot.is_compatible() {
        return None;
    }
    Some(snapshot)
}

pub fn save_snapshot(repo_root: &Path, snapshot: &WorkspaceSnapshot) -> std::io::Result<()> {
    let path = workspace_file_path(repo_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let serialized = serde_json::to_string_pretty(snapshot)?;
    let temp_path = path.with_extension("json.tmp");
    {
        let mut temp = fs::File::create(&temp_path)?;
        temp.write_all(serialized.as_bytes())?;
        temp.sync_all()?;
    }
    fs::rename(temp_path, path)
}

pub fn load_palette_history(repo_root: &Path) -> Option<Vec<String>> {
    load_snapshot(repo_root).map(|snapshot| snapshot.command_palette_history)
}

pub fn save_palette_history(repo_root: &Path, history: &[String]) -> std::io::Result<()> {
    let mut snapshot = load_snapshot(repo_root).unwrap_or_default();
    snapshot.schema_version = WORKSPACE_SCHEMA_VERSION;
    snapshot.command_palette_history = trim_history(history.to_vec());
    save_snapshot(repo_root, &snapshot)
}

#[must_use]
pub fn workspace_file_path(repo_root: &Path) -> PathBuf {
    state_dir().join(format!("{}.json", repo_hash(repo_root)))
}

#[must_use]
pub fn repo_hash(repo_root: &Path) -> String {
    let canonical = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
    let path = canonical.to_string_lossy();
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in path.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x00000100000001B3);
    }
    format!("{hash:016x}")
}

fn state_dir() -> PathBuf {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .unwrap_or_else(|| PathBuf::from(".local/state"));

    base.join("kiwi").join("workspaces")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;
    use crate::workspace::snapshot::{scroll_view, WorkspaceSnapshot};

    static TEST_ENV: Mutex<()> = Mutex::new(());

    struct TempStateDir {
        _guard: std::sync::MutexGuard<'static, ()>,
        path: PathBuf,
        original: Option<std::ffi::OsString>,
    }

    impl TempStateDir {
        fn new(label: &str) -> Self {
            let guard = TEST_ENV.lock().expect("test env lock");
            let path = std::env::temp_dir().join(format!(
                "kiwi-workspace-persist-{label}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp dir");
            let original = std::env::var_os("XDG_STATE_HOME");
            std::env::set_var("XDG_STATE_HOME", &path);
            Self {
                _guard: guard,
                path,
                original,
            }
        }
    }

    impl Drop for TempStateDir {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => std::env::set_var("XDG_STATE_HOME", value),
                None => std::env::remove_var("XDG_STATE_HOME"),
            }
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn repo_hash_is_stable_for_same_path() {
        let first = repo_hash(Path::new("/tmp/kiwi"));
        let second = repo_hash(Path::new("/tmp/kiwi"));
        assert_eq!(first, second);
        assert_eq!(first.len(), 16);
    }

    #[test]
    fn save_and_load_snapshot_round_trip() {
        let _dir = TempStateDir::new("snapshot");
        let repo = Path::new("/tmp/kiwi-workspace-snapshot");
        let snapshot = WorkspaceSnapshot {
            left_nav_tab: "Files".to_string(),
            main_tab: "Preview".to_string(),
            focus: "Left".to_string(),
            left_width: 24,
            expanded_paths: vec!["src".to_string()],
            selected_path: Some("src/lib.rs".to_string()),
            scroll_positions: HashMap::from([(scroll_view::PREVIEW.to_string(), 12)]),
            command_palette_history: vec!["quit".to_string()],
            ..WorkspaceSnapshot::default()
        };

        save_snapshot(repo, &snapshot).expect("save");
        let loaded = load_snapshot(repo).expect("load");
        assert_eq!(loaded, snapshot);
    }

    #[test]
    fn save_palette_history_preserves_other_snapshot_fields() {
        let _dir = TempStateDir::new("palette-merge");
        let repo = Path::new("/tmp/kiwi-workspace-palette-merge");
        let snapshot = WorkspaceSnapshot {
            main_tab: "Diff".to_string(),
            left_width: 33,
            ..WorkspaceSnapshot::default()
        };
        save_snapshot(repo, &snapshot).expect("save initial snapshot");

        let history = vec!["git.refresh".to_string(), "quit".to_string()];
        save_palette_history(repo, &history).expect("save palette history");

        let loaded = load_snapshot(repo).expect("load");
        assert_eq!(loaded.main_tab, "Diff");
        assert_eq!(loaded.left_width, 33);
        assert_eq!(loaded.command_palette_history, history);
    }

    #[test]
    fn save_and_load_palette_history_round_trip() {
        let _dir = TempStateDir::new("palette");
        let repo = Path::new("/tmp/kiwi-workspace-palette");
        let history = vec![
            "git.refresh".to_string(),
            "quit".to_string(),
            "focus.shell".to_string(),
        ];
        save_palette_history(repo, &history).expect("save");
        let loaded = load_palette_history(repo).expect("load");
        assert_eq!(loaded, history);
    }

    #[test]
    fn load_snapshot_returns_none_for_incompatible_schema() {
        let _dir = TempStateDir::new("incompatible");
        let repo = Path::new("/tmp/kiwi-workspace-incompatible");
        let path = workspace_file_path(repo);
        fs::create_dir_all(path.parent().expect("parent")).expect("create dir");
        fs::write(
            path,
            r#"{"schema_version":99,"left_nav_tab":"Files","main_tab":"Agent","focus":"Main","left_width":30,"expanded_paths":[],"selected_path":null,"scroll_positions":{},"command_palette_history":[]}"#,
        )
        .expect("write");

        assert!(load_snapshot(repo).is_none());
    }
}
