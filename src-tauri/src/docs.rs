//! Documentation listing/reading for the Help Activity panel.
//!
//! Rooted at whatever local directory [`crate::doc_sources`] resolves for a
//! given registered project (a synced sparse-checkout under
//! `~/.config/kiwi/docs/<id>/`) — this module has no knowledge of the
//! registry itself, just Markdown-tree traversal.

use std::fs;
use std::path::Path;

use nest_error::{NestError, NestResult};
use serde::Serialize;

/// One entry in the Help sidebar's table of contents.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocEntry {
    /// Path relative to the project's docs root (`/`-separated).
    pub path: String,
    /// Display name, derived from the file/parent-directory name.
    pub name: String,
    /// Nesting depth, for indentation.
    pub depth: u32,
}

/// Lists a project's documentation entries (all `**/*.md` under `root`, sorted by path).
pub fn list(root: &Path) -> NestResult<Vec<DocEntry>> {
    let mut paths = Vec::new();
    collect_markdown(root, root, &mut paths)?;
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

/// Reads a Markdown file relative to `root`.
pub fn read(root: &Path, rel_path: &str) -> NestResult<String> {
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
