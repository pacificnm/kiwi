use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: u32 = 1;
const MAX_PERSISTED_HISTORY: usize = crate::workspace::MAX_PALETTE_HISTORY_ENTRIES;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct WorkspaceFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    palette_history: Vec<String>,
}

pub fn load_palette_history(repo_root: &Path) -> Option<Vec<String>> {
    let path = workspace_file_path(repo_root);
    let contents = fs::read_to_string(path).ok()?;
    let file: WorkspaceFile = serde_json::from_str(&contents).ok()?;
    if file.schema_version != SCHEMA_VERSION {
        return None;
    }

    Some(trim_history(file.palette_history))
}

pub fn save_palette_history(repo_root: &Path, history: &[String]) -> std::io::Result<()> {
    let path = workspace_file_path(repo_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = WorkspaceFile {
        schema_version: SCHEMA_VERSION,
        palette_history: trim_history(history.to_vec()),
    };

    let serialized = serde_json::to_string_pretty(&file)?;
    let temp_path = path.with_extension("json.tmp");
    {
        let mut temp = fs::File::create(&temp_path)?;
        temp.write_all(serialized.as_bytes())?;
        temp.sync_all()?;
    }
    fs::rename(temp_path, path)
}

fn trim_history(mut history: Vec<String>) -> Vec<String> {
    if history.len() > MAX_PERSISTED_HISTORY {
        let overflow = history.len() - MAX_PERSISTED_HISTORY;
        history.drain(0..overflow);
    }
    history
}

fn workspace_file_path(repo_root: &Path) -> PathBuf {
    state_dir().join(format!("{}.json", repo_hash(repo_root)))
}

fn state_dir() -> PathBuf {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .unwrap_or_else(|| PathBuf::from(".local/state"));

    base.join("kiwi").join("workspaces")
}

fn repo_hash(repo_root: &Path) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_hash_is_stable_for_same_path() {
        let first = repo_hash(Path::new("/tmp/kiwi"));
        let second = repo_hash(Path::new("/tmp/kiwi"));
        assert_eq!(first, second);
        assert_eq!(first.len(), 16);
    }

    #[test]
    fn save_and_load_palette_history_round_trip() {
        let temp =
            std::env::temp_dir().join(format!("kiwi-palette-history-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("create temp dir");

        let original = std::env::var_os("XDG_STATE_HOME");
        std::env::set_var("XDG_STATE_HOME", &temp);

        let history = vec![
            "git.refresh".to_string(),
            "quit".to_string(),
            "focus.shell".to_string(),
        ];
        save_palette_history(Path::new("/tmp/kiwi"), &history).expect("save");
        let loaded = load_palette_history(Path::new("/tmp/kiwi")).expect("load");
        assert_eq!(loaded, history);

        match original {
            Some(value) => std::env::set_var("XDG_STATE_HOME", value),
            None => std::env::remove_var("XDG_STATE_HOME"),
        }
        let _ = fs::remove_dir_all(temp);
    }
}
