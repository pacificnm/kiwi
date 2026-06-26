use std::fs;
use std::path::{Path, PathBuf};

use kiwi_plugin_api::PluginManifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginCandidate {
    pub manifest_path: PathBuf,
    pub manifest: PluginManifest,
    pub library_path: PathBuf,
}

/// Scan `plugins_dir` for plugin subdirectories containing `plugin.toml` and a native library.
#[must_use]
pub fn discover_plugins(plugins_dir: &Path) -> (Vec<PluginCandidate>, Vec<String>) {
    let mut candidates = Vec::new();
    let mut warnings = Vec::new();

    let entries = match fs::read_dir(plugins_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return (candidates, warnings),
        Err(err) => {
            warnings.push(format!(
                "Failed to read plugins directory {}: {err}",
                plugins_dir.display()
            ));
            return (candidates, warnings);
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("plugin.toml");
        if !manifest_path.is_file() {
            continue;
        }

        let manifest = match read_manifest(&manifest_path) {
            Ok(manifest) => manifest,
            Err(message) => {
                warnings.push(message);
                continue;
            }
        };

        let Some(library_path) = find_library(&path) else {
            warnings.push(format!(
                "Plugin `{}` skipped: no native library in {}",
                manifest.name,
                path.display()
            ));
            continue;
        };

        candidates.push(PluginCandidate {
            manifest_path,
            manifest,
            library_path,
        });
    }

    candidates.sort_by(|left, right| left.manifest.name.cmp(&right.manifest.name));
    (candidates, warnings)
}

fn read_manifest(path: &Path) -> Result<PluginManifest, String> {
    let content = fs::read_to_string(path).map_err(|err| {
        format!(
            "Plugin skipped: failed to read manifest {}: {err}",
            path.display()
        )
    })?;
    toml::from_str(&content)
        .map_err(|err| format!("Plugin skipped: invalid manifest {}: {err}", path.display()))
}

fn find_library(plugin_dir: &Path) -> Option<PathBuf> {
    let extension = library_extension();
    let entries = fs::read_dir(plugin_dir).ok()?;
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == extension) {
            matches.push(path);
        }
    }
    matches.sort();
    matches.into_iter().next()
}

fn library_extension() -> &'static str {
    if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempPluginDir {
        path: PathBuf,
    }

    impl TempPluginDir {
        fn new(name: &str) -> Self {
            let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("kiwi-plugin-discovery-{name}-{id}"));
            fs::create_dir_all(&path).expect("temp dir");
            Self { path }
        }

        fn plugin_dir(&self, name: &str) -> PathBuf {
            let dir = self.path.join(name);
            fs::create_dir_all(&dir).expect("plugin dir");
            dir
        }
    }

    impl Drop for TempPluginDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn discovers_plugin_subdirectory_with_manifest_and_library() {
        let temp = TempPluginDir::new("happy");
        let plugin_dir = temp.plugin_dir("hello");
        fs::write(
            plugin_dir.join("plugin.toml"),
            r#"
            name = "hello"
            version = "0.1.0"
            min_kiwi_version = "0.1.0"
            "#,
        )
        .expect("manifest");
        fs::write(plugin_dir.join("libhello.so"), b"fake").expect("library");

        let (candidates, warnings) = discover_plugins(&temp.path);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].manifest.name, "hello");
        assert_eq!(candidates[0].library_path, plugin_dir.join("libhello.so"));
    }

    #[test]
    fn missing_plugins_directory_is_not_an_error() {
        let (candidates, warnings) =
            discover_plugins(Path::new("/tmp/kiwi-plugin-discovery-missing-dir"));
        assert!(candidates.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn invalid_manifest_produces_warning() {
        let temp = TempPluginDir::new("invalid");
        let plugin_dir = temp.plugin_dir("broken");
        fs::write(plugin_dir.join("plugin.toml"), "not = [valid").expect("manifest");
        fs::write(plugin_dir.join("libbroken.so"), b"fake").expect("library");

        let (candidates, warnings) = discover_plugins(&temp.path);
        assert!(candidates.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("invalid manifest"));
    }
}
