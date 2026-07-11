//! Registry of external documentation sources for the Help Activity panel.
//!
//! Each entry names a git repo + subpath (e.g. Nest's own `docs/`) that gets
//! sparse-checked-out into `~/.config/kiwi/docs/<id>/` so Help can show a
//! project's docs without vendoring them into Kiwi's own source tree.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use nest_error::{NestError, NestResult};
use serde::{Deserialize, Serialize};

use crate::config_host::kiwi_config_dir;

/// On-disk registry record (`~/.config/kiwi/docs.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocProjectRecord {
    id: String,
    name: String,
    repo_url: String,
    docs_path: String,
    branch: Option<String>,
    last_synced_at: Option<i64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DocRegistry {
    #[serde(default)]
    projects: Vec<DocProjectRecord>,
}

/// A doc source as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocProject {
    pub id: String,
    pub name: String,
    pub repo_url: String,
    pub docs_path: String,
    pub branch: Option<String>,
    /// True once the docs subpath has been checked out locally.
    pub synced: bool,
    /// Unix seconds of the last successful sync, if any.
    pub last_synced_at: Option<i64>,
}

/// New-project fields supplied from the Doc Sources settings form.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocProjectInput {
    pub name: String,
    pub repo_url: String,
    pub docs_path: String,
    pub branch: Option<String>,
}

/// Lists all registered doc sources, with live sync status.
pub fn list() -> NestResult<Vec<DocProject>> {
    let registry = load_registry()?;
    Ok(registry.projects.iter().map(to_project).collect())
}

/// Adds a new doc source to the registry (does not sync it).
pub fn add(input: DocProjectInput) -> NestResult<DocProject> {
    let name = input.name.trim();
    let repo_url = input.repo_url.trim();
    let docs_path = input.docs_path.trim().trim_matches('/');
    if name.is_empty() {
        return Err(NestError::validation("name is required"));
    }
    if repo_url.is_empty() {
        return Err(NestError::validation("repo URL is required"));
    }
    let docs_path = if docs_path.is_empty() { "docs" } else { docs_path };

    let mut registry = load_registry()?;
    let id = unique_id(&registry, name);
    let record = DocProjectRecord {
        id,
        name: name.to_string(),
        repo_url: repo_url.to_string(),
        docs_path: docs_path.to_string(),
        branch: input.branch.filter(|b| !b.trim().is_empty()),
        last_synced_at: None,
    };
    let project = to_project(&record);
    registry.projects.push(record);
    save_registry(&registry)?;
    Ok(project)
}

/// Removes a doc source and deletes its local cache, if any.
pub fn remove(id: &str) -> NestResult<()> {
    let mut registry = load_registry()?;
    let before = registry.projects.len();
    registry.projects.retain(|project| project.id != id);
    if registry.projects.len() == before {
        return Err(NestError::validation(format!("unknown doc source: {id}")));
    }
    save_registry(&registry)?;
    let _ = fs::remove_dir_all(project_dir(id));
    Ok(())
}

/// Clones (first sync) or pulls (subsequent syncs) a doc source's sparse checkout.
pub fn sync(id: &str) -> NestResult<DocProject> {
    let mut registry = load_registry()?;
    let record = registry
        .projects
        .iter_mut()
        .find(|project| project.id == id)
        .ok_or_else(|| NestError::validation(format!("unknown doc source: {id}")))?;

    let dir = project_dir(&record.id);
    if dir.join(".git").is_dir() {
        pull(&dir)?;
    } else {
        clone_sparse(&record.repo_url, record.branch.as_deref(), &dir)?;
        sparse_checkout_set(&dir, &record.docs_path)?;
    }
    record.last_synced_at = Some(now_unix());
    let project = to_project(record);
    save_registry(&registry)?;
    Ok(project)
}

/// Resolves a registered project's local docs root, erroring if unknown or not yet synced.
pub fn docs_root(id: &str) -> NestResult<PathBuf> {
    let registry = load_registry()?;
    let record = registry
        .projects
        .iter()
        .find(|project| project.id == id)
        .ok_or_else(|| NestError::validation(format!("unknown doc source: {id}")))?;
    let root = project_dir(&record.id).join(&record.docs_path);
    if !root.is_dir() {
        return Err(NestError::validation(format!(
            "\"{}\" has not been synced yet — sync it from Settings \u{2192} Help \u{2192} Doc Sources",
            record.name
        )));
    }
    Ok(root)
}

fn to_project(record: &DocProjectRecord) -> DocProject {
    let synced = project_dir(&record.id).join(&record.docs_path).is_dir();
    DocProject {
        id: record.id.clone(),
        name: record.name.clone(),
        repo_url: record.repo_url.clone(),
        docs_path: record.docs_path.clone(),
        branch: record.branch.clone(),
        synced,
        last_synced_at: record.last_synced_at,
    }
}

fn project_dir(id: &str) -> PathBuf {
    kiwi_config_dir().join("docs").join(id)
}

fn registry_path() -> PathBuf {
    kiwi_config_dir().join("docs.toml")
}

fn load_registry() -> NestResult<DocRegistry> {
    let path = registry_path();
    if !path.is_file() {
        return Ok(DocRegistry::default());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|error| NestError::io(format!("failed to read {}: {error}", path.display())))?;
    toml::from_str(&raw)
        .map_err(|error| NestError::config(format!("invalid {}: {error}", path.display())))
}

fn save_registry(registry: &DocRegistry) -> NestResult<()> {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| NestError::io(format!("failed to create {}: {error}", parent.display())))?;
    }
    let raw = toml::to_string_pretty(registry)
        .map_err(|error| NestError::config(format!("failed to serialize doc registry: {error}")))?;
    fs::write(&path, raw)
        .map_err(|error| NestError::io(format!("failed to write {}: {error}", path.display())))
}

fn unique_id(registry: &DocRegistry, name: &str) -> String {
    let base = slugify(name);
    let mut candidate = base.clone();
    let mut suffix = 2;
    while registry.projects.iter().any(|project| project.id == candidate) {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
    candidate
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for ch in name.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }
    let trimmed = slug.trim_end_matches('-').to_string();
    if trimmed.is_empty() {
        "project".to_string()
    } else {
        trimmed
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn clone_sparse(repo_url: &str, branch: Option<&str>, dest: &Path) -> NestResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| NestError::io(format!("failed to create {}: {error}", parent.display())))?;
    }
    let mut command = Command::new("git");
    command
        .arg("clone")
        .arg("--filter=blob:none")
        .arg("--sparse");
    if let Some(branch) = branch {
        command.arg("--branch").arg(branch);
    }
    command.arg(repo_url).arg(dest);
    run(&mut command, "clone")
}

fn sparse_checkout_set(dir: &Path, docs_path: &str) -> NestResult<()> {
    run(
        Command::new("git")
            .current_dir(dir)
            .args(["sparse-checkout", "set", docs_path]),
        "sparse-checkout set",
    )
}

fn pull(dir: &Path) -> NestResult<()> {
    run(Command::new("git").current_dir(dir).arg("pull"), "pull")
}

fn run(command: &mut Command, action: &str) -> NestResult<()> {
    let output = command
        .output()
        .map_err(|error| NestError::io(format!("git {action} failed: {error}")))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if !stderr.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };
    Err(NestError::io(format!("git {action} failed: {detail}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_lowercases_and_dashes() {
        assert_eq!(slugify("Nest Framework"), "nest-framework");
        assert_eq!(slugify("  Loon (webOS)  "), "loon-webos");
    }

    #[test]
    fn unique_id_dedupes() {
        let registry = DocRegistry {
            projects: vec![DocProjectRecord {
                id: "nest".into(),
                name: "Nest".into(),
                repo_url: "x".into(),
                docs_path: "docs".into(),
                branch: None,
                last_synced_at: None,
            }],
        };
        assert_eq!(unique_id(&registry, "Nest"), "nest-2");
    }
}
