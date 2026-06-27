#![allow(dead_code)] // load_snapshot helpers remain for tests and merge saves.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::state::ReduceView;

use super::snapshot::{
    trim_history, GuiWorkspaceSnapshot, TuiWorkspaceSnapshot, WorkspaceFile, WorkspaceSnapshot,
    WORKSPACE_SCHEMA_VERSION, WORKSPACE_SCHEMA_VERSION_V1,
};

pub fn load_snapshot(repo_root: &Path) -> Option<WorkspaceSnapshot> {
    load_workspace_file(repo_root, false).map(|file| file.tui_snapshot())
}

/// Load workspace snapshot for startup (SPEC-017). Logs warnings on corrupt or incompatible files.
pub fn try_load_workspace(repo_root: &Path) -> Option<WorkspaceSnapshot> {
    load_workspace_file(repo_root, true).map(|file| file.tui_snapshot())
}

/// Load full workspace file including optional GUI section (ADR-022 / SPEC-017 v2).
pub fn load_workspace_file(repo_root: &Path, log_errors: bool) -> Option<WorkspaceFile> {
    let path = workspace_file_path(repo_root);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return None,
        Err(err) => {
            if log_errors {
                eprintln!("workspace: failed to read {}: {err}", path.display());
            }
            return None;
        }
    };

    parse_workspace_contents(&contents, log_errors)
}

/// Load full workspace file for startup; logs warnings on corrupt or incompatible files.
pub fn try_load_workspace_file(repo_root: &Path) -> Option<WorkspaceFile> {
    load_workspace_file(repo_root, true)
}

fn parse_workspace_contents(contents: &str, log_errors: bool) -> Option<WorkspaceFile> {
    if let Ok(file) = serde_json::from_str::<WorkspaceFile>(contents) {
        if file.schema_version == WORKSPACE_SCHEMA_VERSION {
            return Some(file);
        }
        if log_errors {
            eprintln!(
                "workspace: unsupported schema version {} (expected {WORKSPACE_SCHEMA_VERSION})",
                file.schema_version
            );
        }
        return None;
    }

    match serde_json::from_str::<WorkspaceSnapshot>(contents) {
        Ok(snapshot) if snapshot.schema_version == WORKSPACE_SCHEMA_VERSION_V1 => {
            Some(WorkspaceFile::from_v1(snapshot))
        }
        Ok(snapshot) => {
            if log_errors {
                eprintln!(
                    "workspace: unsupported schema version {} (expected {WORKSPACE_SCHEMA_VERSION_V1} or {WORKSPACE_SCHEMA_VERSION})",
                    snapshot.schema_version
                );
            }
            None
        }
        Err(err) => {
            if log_errors {
                eprintln!("workspace: corrupt snapshot: {err}");
            }
            None
        }
    }
}

pub fn save_from_reduce_view(view: &ReduceView<'_>) -> std::io::Result<()> {
    merge_save_tui(
        view.repo_root,
        &TuiWorkspaceSnapshot::from_reduce_view(view),
    )
}

/// Persist current app state when `workspace.persist` is enabled (SPEC-017).
pub fn try_save_from_reduce_view(view: &ReduceView<'_>) {
    if !view.config.workspace.persist {
        return;
    }
    if let Err(err) = save_from_reduce_view(view) {
        eprintln!(
            "workspace: failed to save {}: {err}",
            workspace_file_path(view.repo_root).display()
        );
    }
}

pub fn merge_save_tui(repo_root: &Path, tui: &TuiWorkspaceSnapshot) -> std::io::Result<()> {
    let mut file = load_workspace_file(repo_root, false).unwrap_or_default();
    file.schema_version = WORKSPACE_SCHEMA_VERSION;
    file.tui = tui.clone();
    save_workspace_file(repo_root, &file)
}

pub fn merge_save_gui(repo_root: &Path, gui: &GuiWorkspaceSnapshot) -> std::io::Result<()> {
    let mut file = load_workspace_file(repo_root, false).unwrap_or_default();
    file.schema_version = WORKSPACE_SCHEMA_VERSION;
    file.gui = Some(gui.clone());
    save_workspace_file(repo_root, &file)
}

/// Persist GUI dock layout when `workspace.persist` is enabled (SPEC-022 / #186).
pub fn try_merge_save_gui(repo_root: &Path, persist: bool, gui: &GuiWorkspaceSnapshot) {
    if !persist {
        return;
    }
    if let Err(err) = merge_save_gui(repo_root, gui) {
        eprintln!(
            "workspace: failed to save gui layout {}: {err}",
            workspace_file_path(repo_root).display()
        );
    }
}

pub fn save_snapshot(repo_root: &Path, snapshot: &WorkspaceSnapshot) -> std::io::Result<()> {
    merge_save_tui(repo_root, &TuiWorkspaceSnapshot::from(snapshot.clone()))
}

pub fn save_workspace_file(repo_root: &Path, file: &WorkspaceFile) -> std::io::Result<()> {
    let path = workspace_file_path(repo_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let serialized = serde_json::to_string_pretty(file)?;
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
    let mut file = load_workspace_file(repo_root, false).unwrap_or_default();
    file.schema_version = WORKSPACE_SCHEMA_VERSION;
    file.tui.command_palette_history = trim_history(history.to_vec());
    save_workspace_file(repo_root, &file)
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

    use crate::config::ResolvedConfig;
    use crate::navigation::{LeftNavTab, MainTab};
    use crate::state::{AppState, ReduceView, ViewportMetrics};
    use crate::theme::{load_theme_with_capabilities, TerminalCapabilities};

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

    fn test_state(repo: &Path) -> AppState {
        AppState::from_startup(
            repo.to_path_buf(),
            false,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics {
                settings_rows: 10,
                github_list_rows: 10,
                github_detail_rows: 20,
                branches_rows: 10,
                git_rows: 10,
                file_tree_rows: 10,
                preview_rows: 20,
                preview_cols: 80,
                search_rows: 10,
                shell_rows: 20,
                shell_cols: 80,
                agent_rows: 15,
                agent_cols: 100,
            },
        )
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
    fn save_from_reduce_view_round_trip() {
        let _dir = TempStateDir::new("from-state");
        let repo = Path::new("/tmp/kiwi-workspace-from-state");
        let mut state = test_state(repo);
        state.navigation.left_tab = LeftNavTab::Git;
        state.navigation.main_tab = MainTab::Preview;
        state.config.app.left_width = 27;
        state.palette.history = vec!["quit".to_string()];

        save_from_reduce_view(&ReduceView::from_app_state(&mut state)).expect("save");
        let loaded = load_snapshot(repo).expect("load");

        assert_eq!(loaded.left_nav_tab, "Git");
        assert_eq!(loaded.main_tab, "Preview");
        assert_eq!(loaded.left_width, 27);
        assert_eq!(loaded.command_palette_history, vec!["quit".to_string()]);
    }

    #[test]
    fn two_repos_have_isolated_palette_history() {
        let _dir = TempStateDir::new("two-repos");
        let repo_a = Path::new("/tmp/kiwi-workspace-repo-a");
        let repo_b = Path::new("/tmp/kiwi-workspace-repo-b");
        save_palette_history(repo_a, &["git.refresh".to_string()]).expect("save a");
        save_palette_history(repo_b, &["quit".to_string()]).expect("save b");

        assert_ne!(workspace_file_path(repo_a), workspace_file_path(repo_b));
        assert_eq!(
            load_palette_history(repo_a),
            Some(vec!["git.refresh".to_string()])
        );
        assert_eq!(load_palette_history(repo_b), Some(vec!["quit".to_string()]));
    }

    #[test]
    fn palette_history_round_trips_through_workspace_restore() {
        let _dir = TempStateDir::new("palette-restore");
        let repo = Path::new("/tmp/kiwi-workspace-palette-restore");
        let mut state = test_state(repo);
        state.palette.history = vec!["git.refresh".to_string(), "quit".to_string()];

        save_from_reduce_view(&ReduceView::from_app_state(&mut state)).expect("save");

        let mut restored = test_state(repo);
        let snapshot = try_load_workspace(repo).expect("load snapshot");
        snapshot.apply_to_reduce_view(&mut ReduceView::from_app_state(&mut restored));

        assert_eq!(
            restored.palette.history,
            vec!["git.refresh".to_string(), "quit".to_string()]
        );
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

    #[test]
    fn v1_flat_file_migrates_on_load() {
        let _dir = TempStateDir::new("v1-migrate");
        let repo = Path::new("/tmp/kiwi-workspace-v1-migrate");
        let path = workspace_file_path(repo);
        fs::create_dir_all(path.parent().expect("parent")).expect("create dir");
        fs::write(
            path,
            r#"{"schema_version":1,"left_nav_tab":"Git","main_tab":"Diff","focus":"Main","left_width":28,"expanded_paths":[],"selected_path":null,"scroll_positions":{},"command_palette_history":["quit"]}"#,
        )
        .expect("write");

        let file = load_workspace_file(repo, false).expect("load");
        assert_eq!(file.schema_version, WORKSPACE_SCHEMA_VERSION);
        assert_eq!(file.tui.left_nav_tab, "Git");
        assert_eq!(file.tui.main_tab, "Diff");
        assert_eq!(file.gui, None);
    }

    #[test]
    fn merge_save_tui_preserves_gui_section() {
        let _dir = TempStateDir::new("merge-tui-gui");
        let repo = Path::new("/tmp/kiwi-workspace-merge-tui-gui");
        let gui = GuiWorkspaceSnapshot {
            dock_layout: serde_json::json!({"tabs": ["Explorer"]}),
            open_tabs: vec!["Explorer".to_string()],
        };
        merge_save_gui(repo, &gui).expect("save gui");

        let mut state = test_state(repo);
        state.navigation.main_tab = MainTab::Preview;
        merge_save_tui(
            repo,
            &TuiWorkspaceSnapshot::from_reduce_view(&ReduceView::from_app_state(&mut state)),
        )
        .expect("save tui");

        let file = load_workspace_file(repo, false).expect("load");
        assert_eq!(file.tui.main_tab, "Preview");
        assert_eq!(file.gui.as_ref(), Some(&gui));
    }

    #[test]
    fn merge_save_gui_preserves_tui_section() {
        let _dir = TempStateDir::new("merge-gui-tui");
        let repo = Path::new("/tmp/kiwi-workspace-merge-gui-tui");
        let snapshot = WorkspaceSnapshot {
            main_tab: "Diff".to_string(),
            left_width: 40,
            ..WorkspaceSnapshot::default()
        };
        save_snapshot(repo, &snapshot).expect("save tui");

        let gui = GuiWorkspaceSnapshot {
            dock_layout: serde_json::json!({"tabs": ["Agent"]}),
            open_tabs: vec!["Agent".to_string()],
        };
        merge_save_gui(repo, &gui).expect("save gui");

        let file = load_workspace_file(repo, false).expect("load");
        assert_eq!(file.tui.main_tab, "Diff");
        assert_eq!(file.tui.left_width, 40);
        assert_eq!(file.gui.as_ref(), Some(&gui));
    }

    #[test]
    fn saved_file_uses_v2_schema_layout() {
        let _dir = TempStateDir::new("v2-layout");
        let repo = Path::new("/tmp/kiwi-workspace-v2-layout");
        save_snapshot(
            repo,
            &WorkspaceSnapshot {
                main_tab: "Agent".to_string(),
                ..WorkspaceSnapshot::default()
            },
        )
        .expect("save");

        let raw = fs::read_to_string(workspace_file_path(repo)).expect("read");
        let parsed: serde_json::Value = serde_json::from_str(&raw).expect("json");
        assert_eq!(parsed["schema_version"], WORKSPACE_SCHEMA_VERSION);
        assert!(parsed.get("tui").is_some());
    }
}
