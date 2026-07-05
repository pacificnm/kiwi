//! Recently opened workspace folders.

use std::fs;
use std::path::{Path, PathBuf};

use nest_error::{NestError, NestResult};
use serde::{Deserialize, Serialize};

const MAX_RECENT: usize = 10;
const RECENT_FILE: &str = "recent-projects.toml";

/// Persisted list of workspace roots.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RecentFile {
    roots: Vec<PathBuf>,
}

/// Recent workspace folders for **Open Recent**.
#[derive(Debug, Clone)]
pub struct RecentProjects {
    roots: Vec<PathBuf>,
    store_path: PathBuf,
}

impl RecentProjects {
    /// Loads recent folders from disk, or returns an empty list.
    pub fn load(config_path: Option<&Path>) -> Self {
        let store_path = recent_store_path(config_path);
        let roots = fs::read_to_string(&store_path)
            .ok()
            .and_then(|text| toml::from_str::<RecentFile>(&text).ok())
            .map(|file| file.roots)
            .unwrap_or_default();
        Self { roots, store_path }
    }

    /// Recent roots, most recent first.
    pub fn entries(&self) -> &[PathBuf] {
        &self.roots
    }

    /// Returns true when there are no recent entries.
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    /// Short label for a menu row (folder name, falling back to the path).
    pub fn menu_label(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| path.display().to_string())
    }

    /// Records a workspace root at the front of the list and persists to disk.
    pub fn record(&mut self, root: &Path) -> NestResult<()> {
        let canonical = root
            .canonicalize()
            .map_err(|error| NestError::config(format!("invalid workspace root: {error}")))?;

        self.roots.retain(|entry| entry != &canonical);
        self.roots.insert(0, canonical);
        self.roots.truncate(MAX_RECENT);
        self.save()
    }

    fn save(&self) -> NestResult<()> {
        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| NestError::io(format!("create recent dir: {error}")))?;
        }
        let file = RecentFile {
            roots: self.roots.clone(),
        };
        let text = toml::to_string_pretty(&file)
            .map_err(|error| NestError::config(format!("serialize recent: {error}")))?;
        fs::write(&self.store_path, text)
            .map_err(|error| NestError::io(format!("write recent file: {error}")))?;
        Ok(())
    }
}

fn recent_store_path(config_path: Option<&Path>) -> PathBuf {
    let base = config_path
        .and_then(|path| path.parent())
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    base.join(RECENT_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn record_deduplicates_and_orders_most_recent_first() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        let store = dir.path().join("recent-projects.toml");
        let mut recent = RecentProjects {
            roots: vec![a.clone()],
            store_path: store,
        };

        recent.record(&b).unwrap();
        recent.record(&a).unwrap();

        assert_eq!(recent.roots.len(), 2);
        assert_eq!(recent.roots[0], a.canonicalize().unwrap());
        assert_eq!(recent.roots[1], b.canonicalize().unwrap());
    }
}
