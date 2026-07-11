//! Kiwi's own documentation — list and read Markdown under `docs/`.
//!
//! Powers the Help sidebar. Unlike the project [`Workspace`](crate::workspace),
//! this is always rooted at Kiwi's own `docs/` directory, not the open project.

use std::fs;
use std::path::{Path, PathBuf};

use nest_error::{NestError, NestResult};
use serde::Serialize;

/// One entry in the Help sidebar's table of contents.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocEntry {
    /// Path relative to Kiwi's `docs/` directory (`/`-separated).
    pub path: String,
    /// Display name, derived from the file/parent-directory name.
    pub name: String,
    /// Nesting depth, for indentation.
    pub depth: u32,
}

/// Lists Kiwi's documentation entries (all `docs/**/*.md`, sorted by path).
pub fn list() -> NestResult<Vec<DocEntry>> {
    let root = resolve_docs_dir()?;
    let mut paths = Vec::new();
    collect_markdown(&root, &root, &mut paths)?;
    paths.sort();

    Ok(paths
        .into_iter()
        .map(|path| DocEntry {
            depth: path.matches('/').count() as u32,
            name: display_name(&path),
            path,
        })
        .collect())
}

/// Reads a Markdown file relative to Kiwi's `docs/` directory.
pub fn read(rel_path: &str) -> NestResult<String> {
    let root = resolve_docs_dir()?;
    let rel = rel_path.trim().trim_start_matches('/');
    if rel.is_empty() || rel.contains("..") {
        return Err(NestError::validation("invalid document path"));
    }

    let path = root.join(rel);
    if !path.is_file() {
        return Err(NestError::validation(format!("document not found: {rel}")));
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err(NestError::validation("only .md files can be read"));
    }

    fs::read_to_string(&path)
        .map_err(|error| NestError::validation(format!("failed to read {rel}: {error}")))
}

/// Locates Kiwi's `docs/` directory (dev tree only — `../docs` from
/// `src-tauri/`, matching [`crate::config_host`]'s dev-path conventions).
fn resolve_docs_dir() -> NestResult<PathBuf> {
    for candidate in ["../docs", "docs"] {
        let path = PathBuf::from(candidate);
        if path.is_dir() {
            return Ok(path);
        }
    }
    Err(NestError::config("could not locate Kiwi's docs/ directory"))
}

fn collect_markdown(dir: &Path, root: &Path, paths: &mut Vec<String>) -> NestResult<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|error| {
        NestError::validation(format!("failed to read {}: {error}", dir.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| NestError::validation(error.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, root, paths)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            let rel = path
                .strip_prefix(root)
                .map_err(|_| NestError::validation("path outside docs directory"))?;
            paths.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn display_name(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    if file.eq_ignore_ascii_case("README.md") {
        if let Some(parent) = path.rsplit('/').nth(1) {
            return humanize_segment(parent);
        }
        return "Kiwi".into();
    }
    humanize_segment(file.strip_suffix(".md").unwrap_or(file))
}

fn humanize_segment(segment: &str) -> String {
    segment
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_humanizes_top_level_file() {
        assert_eq!(display_name("explorer-v1.md"), "Explorer V1");
    }

    #[test]
    fn display_name_uses_parent_for_nested_readme() {
        assert_eq!(display_name("agent/README.md"), "Agent");
    }
}
