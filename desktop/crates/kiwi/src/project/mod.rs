//! Project root resolution for the Kiwi workspace.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use nest_config::{ConfigLoader, ConfigService};
use nest_error::{NestError, NestResult};
use serde::Deserialize;

/// Environment variable override for the project root directory.
pub const PROJECT_ROOT_ENV: &str = "KIWI_PROJECT_ROOT";

/// Default directory names hidden from the explorer tree.
pub const DEFAULT_IGNORE: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".venv",
    "dist",
    "build",
];

/// `[project]` section in `config.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectSection {
    /// Path to the project root, relative to the config file unless absolute.
    pub root: Option<String>,
    /// Display name for the title bar.
    pub name: Option<String>,
    /// Extra directory names to hide in the explorer (merged with [`DEFAULT_IGNORE`]).
    pub ignore: Option<Vec<String>>,
}

/// Resolved workspace folder for explorer and editor I/O.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Absolute path to the project root.
    pub root: PathBuf,
    /// Short label for the workspace.
    #[allow(dead_code)]
    pub name: String,
    /// Directory names hidden from the explorer.
    #[allow(dead_code)]
    pub ignore: Vec<String>,
}

/// Returns true when `path` is under the project root and passes through an ignored segment.
#[allow(dead_code)]
pub fn path_is_ignored(path: &Path, root: &Path, ignored: &[String]) -> bool {
    let rel = match path.strip_prefix(root) {
        Ok(rel) => rel,
        Err(_) => return true,
    };
    for component in rel.components() {
        if let std::path::Component::Normal(name) = component {
            let name = name.to_string_lossy();
            if ignored.iter().any(|ignored| ignored == name.as_ref()) {
                return true;
            }
        }
    }
    false
}

/// Merges configured ignore entries with the built-in defaults.
pub fn merged_ignore(extra: Option<Vec<String>>) -> Vec<String> {
    let mut list: Vec<String> = DEFAULT_IGNORE.iter().map(|name| (*name).to_string()).collect();
    if let Some(extra) = extra {
        for name in extra {
            if !list.iter().any(|existing| existing == &name) {
                list.push(name);
            }
        }
    }
    list
}

impl ProjectConfig {
    /// Resolves the project root using CLI, config, environment, and auto-detect.
    pub fn resolve(cli_root: Option<PathBuf>, config_path: Option<&Path>) -> NestResult<Self> {
        let section = load_project_section(config_path)?;
        let ignore = merged_ignore(section.as_ref().and_then(|section| section.ignore.clone()));

        if let Some(root) = cli_root {
            let name = section
                .as_ref()
                .and_then(|section| section.name.clone())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| default_name_from_root(&root));
            return Self::from_root_with_name(root, name, ignore);
        }

        if let Ok(value) = env::var(PROJECT_ROOT_ENV) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                let root = PathBuf::from(trimmed);
                let name = section
                    .as_ref()
                    .and_then(|section| section.name.clone())
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| default_name_from_root(&root));
                return Self::from_root_with_name(root, name, ignore);
            }
        }

        if let Some(raw) = section.as_ref().and_then(|section| section.root.clone()) {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                let root = resolve_root_path(config_path, trimmed);
                if root.is_dir() {
                    let name = section
                        .as_ref()
                        .and_then(|section| section.name.clone())
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or_else(|| default_name_from_root(&root));
                    return Self::from_root_with_name(root, name, ignore);
                }
                tracing::warn!(
                    target: "kiwi",
                    root = %root.display(),
                    "Configured [project].root is missing; falling back to auto-detect"
                );
            }
        }

        let root = auto_detect_root()?;
        let name = section
            .and_then(|section| section.name)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| default_name_from_root(&root));
        Self::from_root_with_name(root, name, ignore)
    }

    /// Loads project settings from a running config service.
    #[allow(dead_code)]
    pub fn from_config_service(service: &ConfigService) -> NestResult<Self> {
        Self::resolve(None, service.path())
    }

    /// Builds config from an absolute root path.
    #[allow(dead_code)]
    pub fn from_root(root: PathBuf) -> NestResult<Self> {
        let name = default_name_from_root(&root);
        Self::from_root_with_name(root, name, merged_ignore(None))
    }

    fn from_root_with_name(root: PathBuf, name: String, ignore: Vec<String>) -> NestResult<Self> {
        let root = root
            .canonicalize()
            .map_err(|error| NestError::config(format!("invalid project root: {error}")))?;
        if !root.is_dir() {
            return Err(NestError::config(format!(
                "project root is not a directory: {}",
                root.display()
            )));
        }
        Ok(Self { root, name, ignore })
    }
}

/// Parses `--project-root` from process arguments.
pub fn project_root_from_args(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .skip(1)
        .zip(args.iter().skip(2))
        .find_map(|(flag, value)| {
            if flag == "--project-root" {
                Some(PathBuf::from(value))
            } else {
                None
            }
        })
}

/// Resolves a configured root path relative to the config file directory.
pub fn resolve_root_path(config_path: Option<&Path>, raw: &str) -> PathBuf {
    let path = Path::new(raw);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    let base = config_path
        .and_then(|value| value.parent())
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    base.join(path)
}

fn load_project_section(config_path: Option<&Path>) -> NestResult<Option<ProjectSection>> {
    let loaded = ConfigLoader::file_or_search(
        "kiwi",
        config_path.map(std::path::PathBuf::from),
    )
    .load()?;
    Ok(loaded.document.section("project").ok())
}

fn auto_detect_root() -> NestResult<PathBuf> {
    let start = env::current_dir().map_err(|error| NestError::io(error.to_string()))?;
    let mut current = start.as_path();

    loop {
        if is_project_marker(current) {
            return current.canonicalize().map_err(|error| {
                NestError::config(format!("failed to canonicalize {}: {error}", current.display()))
            });
        }

        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }

    start.canonicalize().map_err(|error| NestError::io(error.to_string()))
}

fn is_project_marker(path: &Path) -> bool {
    if path.join(".git").exists() {
        return true;
    }

    let cargo = path.join("Cargo.toml");
    if !cargo.is_file() {
        return false;
    }

    fs::read_to_string(cargo)
        .map(|content| content.contains("[workspace]"))
        .unwrap_or(false)
}

fn default_name_from_root(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project")
        .to_string()
}

/// Writes `[project].root` and `name` to the Kiwi config file.
#[allow(dead_code)]
pub fn persist_project_root(config_path: &Path, root: &Path) -> NestResult<()> {
    use std::fs;
    use toml::Value;

    let root = root
        .canonicalize()
        .map_err(|error| NestError::config(format!("invalid project root: {error}")))?;
    if !root.is_dir() {
        return Err(NestError::config(format!(
            "project root is not a directory: {}",
            root.display()
        )));
    }

    let content = fs::read_to_string(config_path).map_err(|error| {
        NestError::io(format!("failed to read {}: {error}", config_path.display()))
    })?;
    let mut document: Value = content.parse().map_err(|error| {
        NestError::config(format!("failed to parse {}: {error}", config_path.display()))
    })?;

    let table = document
        .as_table_mut()
        .ok_or_else(|| NestError::config("config root must be a table"))?;

    let project = table
        .entry("project")
        .or_insert_with(|| Value::Table(toml::map::Map::new()));

    let project_table = project
        .as_table_mut()
        .ok_or_else(|| NestError::config("[project] must be a table"))?;

    let stored_root = relativize_root_for_config(config_path, &root);
    project_table.insert("root".into(), Value::String(stored_root));
    project_table.insert(
        "name".into(),
        Value::String(default_name_from_root(&root)),
    );

    let serialized = toml::to_string_pretty(&document).map_err(|error| {
        NestError::config(format!("failed to serialize config: {error}"))
    })?;
    fs::write(config_path, serialized).map_err(|error| {
        NestError::io(format!("failed to write {}: {error}", config_path.display()))
    })?;
    Ok(())
}

fn relativize_root_for_config(config_path: &Path, root: &Path) -> String {
    let Some(config_dir) = config_path.parent() else {
        return root.display().to_string();
    };
    if root == config_dir {
        return ".".into();
    }
    if let Ok(rel) = root.strip_prefix(config_dir) {
        let rel = rel.to_string_lossy();
        if rel.is_empty() {
            ".".into()
        } else {
            rel.replace('\\', "/")
        }
    } else {
        root.display().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolve_relative_root_from_config_directory() {
        let dir = tempdir().unwrap();
        let config = dir.path().join("config.toml");
        let project = dir.path().join("workspace");
        fs::create_dir_all(&project).unwrap();

        let root = resolve_root_path(Some(&config), "workspace");
        assert_eq!(root, project);
    }

    #[test]
    fn auto_detect_finds_git_directory() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::create_dir_all(dir.path().join("nested")).unwrap();

        env::set_current_dir(dir.path().join("nested")).unwrap();
        let root = auto_detect_root().unwrap();
        assert_eq!(root, dir.path().canonicalize().unwrap());
    }

    #[test]
    fn from_root_assigns_default_ignore() {
        let dir = tempdir().unwrap();
        let cfg = ProjectConfig::from_root(dir.path().to_path_buf()).unwrap();
        assert!(cfg.ignore.iter().any(|name| name == "target"));
    }

    #[test]
    fn relativize_root_for_config_uses_dot_for_config_directory() {
        let dir = tempdir().unwrap();
        let config = dir.path().join("config.toml");
        assert_eq!(
            relativize_root_for_config(&config, dir.path()),
            "."
        );
    }

    #[test]
    fn persist_project_root_writes_section() {
        let dir = tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        let config = dir.path().join("config.toml");
        fs::write(&config, "[project]\nname = \"Kiwi\"\n").unwrap();

        persist_project_root(&config, &workspace).unwrap();
        let content = fs::read_to_string(&config).unwrap();
        assert!(content.contains("[project]"));
        assert!(content.contains("root = \"workspace\""));
    }

    #[test]
    fn project_root_from_args_parses_flag() {
        let args = vec![
            "kiwi".into(),
            "--project-root".into(),
            "/tmp/project".into(),
        ];
        assert_eq!(
            project_root_from_args(&args),
            Some(PathBuf::from("/tmp/project"))
        );
    }
}
