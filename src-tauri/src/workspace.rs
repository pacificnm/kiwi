//! Project workspace host for the Explorer sidebar and editor.
//!
//! Owns the resolved project root and a scoped [`FileService`], and exposes
//! directory listing / file reads to the webview. Presentation lives in React;
//! this layer only does safe, scoped file I/O.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use nest_error::{NestError, NestResult};
use nest_file::{FileService, FileServiceConfig};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

/// Environment override for the initial project root.
const PROJECT_ROOT_ENV: &str = "KIWI_PROJECT_ROOT";

/// Directory names hidden from the explorer tree.
const DEFAULT_IGNORE: &[&str] = &[".git", "target", "node_modules", ".venv", "dist", "build"];

/// Largest file the editor will read over IPC (2 MiB).
const MAX_READ_BYTES: u64 = 2 * 1024 * 1024;

/// Largest file search will scan over IPC (5 MiB).
const MAX_SEARCH_BYTES: u64 = 5 * 1024 * 1024;

/// Metadata about the active workspace, sent to the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    /// Absolute project root on disk.
    pub root: String,
    /// Short display name (folder name or configured label).
    pub name: String,
}

/// One entry in a directory listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsEntry {
    /// File or directory name.
    pub name: String,
    /// Path relative to the project root (`/`-separated).
    pub rel_path: String,
    /// Whether the entry is a directory.
    pub is_dir: bool,
}

/// Text contents of a file for the editor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContent {
    /// Path relative to the project root.
    pub rel_path: String,
    /// UTF-8 file contents.
    pub content: String,
}

/// Search options for the workspace Search panel.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSearchQuery {
    pub query: String,
    pub includes: Option<String>,
    pub excludes: Option<String>,
    pub match_case: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    /// Max matches to return (soft cap). Default: 2000.
    pub max_matches: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSearchMatch {
    pub line: u32,
    pub col: u32,
    pub match_text: String,
    pub line_text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSearchFile {
    pub rel_path: String,
    pub matches: Vec<WorkspaceSearchMatch>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSearchResponse {
    pub files: Vec<WorkspaceSearchFile>,
    pub match_count: u32,
    pub file_count: u32,
    pub truncated: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceReplaceRequest {
    pub search: WorkspaceSearchQuery,
    pub replace: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceReplaceResponse {
    pub file_count: u32,
    pub match_count: u32,
}

struct Inner {
    root: PathBuf,
    name: String,
    files: FileService,
}

/// Managed Tauri state: the currently open workspace.
pub struct Workspace {
    inner: Mutex<Inner>,
}

impl Workspace {
    /// Resolves the initial workspace from env, config, or auto-detection.
    ///
    /// Precedence: `KIWI_PROJECT_ROOT` env → `[project].root` in the config →
    /// nearest `.git` / Cargo workspace ancestor → current directory.
    pub fn resolve(config_path: Option<&Path>) -> NestResult<Self> {
        let root = resolve_root(config_path)?;
        Ok(Self {
            inner: Mutex::new(Inner::open(root)?),
        })
    }

    /// Returns the absolute project root on disk.
    pub fn root(&self) -> PathBuf {
        self.inner.lock().expect("workspace mutex").root.clone()
    }

    /// Returns the active workspace metadata.
    pub fn info(&self) -> WorkspaceInfo {
        let inner = self.inner.lock().expect("workspace mutex");
        WorkspaceInfo {
            root: inner.root.display().to_string(),
            name: inner.name.clone(),
        }
    }

    /// Switches the workspace to a new root directory.
    pub fn open(&self, root: impl Into<PathBuf>) -> NestResult<WorkspaceInfo> {
        let next = Inner::open(root.into())?;
        let info = WorkspaceInfo {
            root: next.root.display().to_string(),
            name: next.name.clone(),
        };
        *self.inner.lock().expect("workspace mutex") = next;
        Ok(info)
    }

    /// Lists a directory relative to the project root, dirs first then names.
    ///
    /// Ignored directory names are hidden. `rel` is `"."` for the root.
    pub fn list(&self, rel: &str) -> NestResult<Vec<FsEntry>> {
        let inner = self.inner.lock().expect("workspace mutex");
        let rel = normalize_rel(rel);
        let mut entries: Vec<FsEntry> = inner
            .files
            .list_dir(&rel)?
            .into_iter()
            .filter(|entry| !is_ignored(&entry.name))
            .map(|entry| {
                let rel_path = if rel == "." {
                    entry.name.clone()
                } else {
                    format!("{rel}/{}", entry.name)
                };
                FsEntry {
                    name: entry.name,
                    rel_path,
                    is_dir: entry.metadata.is_dir,
                }
            })
            .collect();

        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        Ok(entries)
    }

    /// Creates an empty file at `rel`. Errors if it already exists.
    pub fn create_file(&self, rel: &str) -> NestResult<String> {
        let inner = self.inner.lock().expect("workspace mutex");
        let rel = normalize_rel(rel);
        reject_root(&rel, "create")?;
        if inner.files.exists(&rel)? {
            return Err(NestError::validation(format!("already exists: {rel}")));
        }
        inner.files.write_text(&rel, "")?;
        Ok(rel)
    }

    /// Creates a directory (and any missing parents) at `rel`.
    pub fn create_dir(&self, rel: &str) -> NestResult<String> {
        let inner = self.inner.lock().expect("workspace mutex");
        let rel = normalize_rel(rel);
        reject_root(&rel, "create")?;
        if inner.files.exists(&rel)? {
            return Err(NestError::validation(format!("already exists: {rel}")));
        }
        inner.files.create_dir_all(&rel)?;
        Ok(rel)
    }

    /// Renames / moves `from` to `to` (both project-relative).
    pub fn rename(&self, from: &str, to: &str) -> NestResult<String> {
        let inner = self.inner.lock().expect("workspace mutex");
        let from = normalize_rel(from);
        let to = normalize_rel(to);
        reject_root(&from, "rename")?;
        reject_root(&to, "rename")?;
        if !inner.files.exists(&from)? {
            return Err(NestError::validation(format!("does not exist: {from}")));
        }
        if inner.files.exists(&to)? {
            return Err(NestError::validation(format!("already exists: {to}")));
        }
        inner.files.move_file(&from, &to)?;
        Ok(to)
    }

    /// Deletes a file or directory tree at `rel`.
    pub fn delete(&self, rel: &str) -> NestResult<String> {
        let inner = self.inner.lock().expect("workspace mutex");
        let rel = normalize_rel(rel);
        reject_root(&rel, "delete")?;
        if !inner.files.exists(&rel)? {
            return Err(NestError::validation(format!("does not exist: {rel}")));
        }
        let metadata = inner.files.metadata(&rel)?;
        if metadata.is_dir {
            inner.files.delete_dir(&rel, true)?;
        } else {
            inner.files.delete_file(&rel)?;
        }
        Ok(rel)
    }

    /// Copies a file or directory tree from `from` to `to`.
    pub fn copy(&self, from: &str, to: &str) -> NestResult<String> {
        let inner = self.inner.lock().expect("workspace mutex");
        let from = normalize_rel(from);
        let to = normalize_rel(to);
        reject_root(&from, "copy")?;
        reject_root(&to, "copy")?;
        if !inner.files.exists(&from)? {
            return Err(NestError::validation(format!("does not exist: {from}")));
        }
        if inner.files.exists(&to)? {
            return Err(NestError::validation(format!("already exists: {to}")));
        }
        let metadata = inner.files.metadata(&from)?;
        if metadata.is_dir {
            copy_dir_tree(&inner.files, &from, &to)?;
        } else {
            inner.files.copy(&from, &to)?;
        }
        Ok(to)
    }

    /// Reveals a path in the OS file manager (Open Containing Folder).
    pub fn reveal(&self, rel: &str) -> NestResult<()> {
        let inner = self.inner.lock().expect("workspace mutex");
        let rel = normalize_rel(rel);
        let abs = if rel == "." {
            inner.root.clone()
        } else {
            inner.root.join(&rel)
        };
        if !abs.exists() {
            return Err(NestError::validation(format!("does not exist: {rel}")));
        }
        reveal_in_file_manager(&abs)
    }

    /// Searches the workspace for query matches, respecting include/exclude globs
    /// and `.gitignore` (VS Code-like behavior).
    pub fn search(&self, query: WorkspaceSearchQuery) -> NestResult<WorkspaceSearchResponse> {
        let inner = self.inner.lock().expect("workspace mutex");

        let needle = query.query.trim();
        if needle.is_empty() {
            return Ok(WorkspaceSearchResponse {
                files: Vec::new(),
                match_count: 0,
                file_count: 0,
                truncated: false,
            });
        }

        let max_matches = query.max_matches.unwrap_or(2000).max(1);
        let include_set = build_glob_set(query.includes.as_deref())?;
        let exclude_set = build_glob_set(query.excludes.as_deref())?;
        let matcher = build_matcher(needle, query.match_case, query.whole_word, query.use_regex)?;

        let mut out_files: Vec<WorkspaceSearchFile> = Vec::new();
        let mut total_matches: u32 = 0;
        let mut truncated = false;

        let walker = WalkBuilder::new(&inner.root)
            .standard_filters(true)
            .hidden(false)
            .follow_links(false)
            .filter_entry(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| !is_ignored(name))
                    .unwrap_or(true)
            })
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(value) => value,
                Err(_) => continue,
            };
            if !entry
                .file_type()
                .map(|ty| ty.is_file())
                .unwrap_or(false)
            {
                continue;
            }

            let abs = entry.path();
            let rel = match abs.strip_prefix(&inner.root) {
                Ok(value) => normalize_rel(&value.display().to_string()),
                Err(_) => continue,
            };

            if let Some(set) = &exclude_set {
                if set.is_match(&rel) {
                    continue;
                }
            }
            if let Some(set) = &include_set {
                if !set.is_match(&rel) {
                    continue;
                }
            }

            let meta = match std::fs::metadata(abs) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if meta.len() > MAX_SEARCH_BYTES {
                continue;
            }

            let bytes = match inner.files.read_bytes(&rel) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if bytes.contains(&0) {
                continue;
            }
            let content = match String::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => continue,
            };

            let mut matches: Vec<WorkspaceSearchMatch> = Vec::new();
            for (idx, line) in content.lines().enumerate() {
                if total_matches >= max_matches {
                    truncated = true;
                    break;
                }
                for mat in matcher.find_iter(line) {
                    if total_matches >= max_matches {
                        truncated = true;
                        break;
                    }
                    matches.push(WorkspaceSearchMatch {
                        line: (idx as u32) + 1,
                        col: (mat.start() as u32) + 1,
                        match_text: line[mat.start()..mat.end()].to_string(),
                        line_text: line.to_string(),
                    });
                    total_matches += 1;
                }
                if truncated {
                    break;
                }
            }

            if !matches.is_empty() {
                out_files.push(WorkspaceSearchFile {
                    rel_path: rel,
                    matches,
                });
            }
            if truncated {
                break;
            }
        }

        let file_count = out_files.len() as u32;
        Ok(WorkspaceSearchResponse {
            files: out_files,
            match_count: total_matches,
            file_count,
            truncated,
        })
    }

    /// Replaces query matches across the workspace and writes changes back to disk.
    ///
    /// This is a "Replace All" equivalent for the current query/options.
    pub fn replace_all(
        &self,
        request: WorkspaceReplaceRequest,
    ) -> NestResult<WorkspaceReplaceResponse> {
        let inner = self.inner.lock().expect("workspace mutex");
        let needle = request.search.query.trim();
        if needle.is_empty() {
            return Ok(WorkspaceReplaceResponse {
                file_count: 0,
                match_count: 0,
            });
        }

        let include_set = build_glob_set(request.search.includes.as_deref())?;
        let exclude_set = build_glob_set(request.search.excludes.as_deref())?;
        let matcher =
            build_matcher(needle, request.search.match_case, request.search.whole_word, request.search.use_regex)?;

        let walker = WalkBuilder::new(&inner.root)
            .standard_filters(true)
            .hidden(false)
            .follow_links(false)
            .filter_entry(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| !is_ignored(name))
                    .unwrap_or(true)
            })
            .build();

        let mut changed_files: u32 = 0;
        let mut replaced_matches: u32 = 0;

        for entry in walker {
            let entry = match entry {
                Ok(value) => value,
                Err(_) => continue,
            };
            if !entry
                .file_type()
                .map(|ty| ty.is_file())
                .unwrap_or(false)
            {
                continue;
            }

            let abs = entry.path();
            let rel = match abs.strip_prefix(&inner.root) {
                Ok(value) => normalize_rel(&value.display().to_string()),
                Err(_) => continue,
            };

            if let Some(set) = &exclude_set {
                if set.is_match(&rel) {
                    continue;
                }
            }
            if let Some(set) = &include_set {
                if !set.is_match(&rel) {
                    continue;
                }
            }

            let meta = match std::fs::metadata(abs) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if meta.len() > MAX_SEARCH_BYTES {
                continue;
            }

            let bytes = match inner.files.read_bytes(&rel) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if bytes.contains(&0) {
                continue;
            }
            let content = match String::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => continue,
            };

            let count_here = matcher.find_iter(&content).count() as u32;
            if count_here == 0 {
                continue;
            }

            let replaced = matcher.replace_all(&content, request.replace.as_str());
            if replaced == content {
                continue;
            }

            inner.files.write_text(&rel, &replaced)?;
            changed_files += 1;
            replaced_matches += count_here;
        }

        Ok(WorkspaceReplaceResponse {
            file_count: changed_files,
            match_count: replaced_matches,
        })
    }

    /// Writes UTF-8 `content` to an existing file relative to the project root.
    ///
    /// Used by the editor's save action. Rejects the project root and refuses
    /// to overwrite a directory, but creates the file if it does not yet exist
    /// (parent directories must already exist).
    pub fn write_text(&self, rel: &str, content: &str) -> NestResult<String> {
        let inner = self.inner.lock().expect("workspace mutex");
        let rel = normalize_rel(rel);
        reject_root(&rel, "save")?;
        if inner.files.exists(&rel)? {
            let metadata = inner.files.metadata(&rel)?;
            if metadata.is_dir {
                return Err(NestError::validation(format!("{rel} is a directory")));
            }
        }
        inner.files.write_text(&rel, content)?;
        Ok(rel)
    }

    /// Reads a UTF-8 text file relative to the project root.
    ///
    /// Rejects directories, oversized files, and binary content so the editor
    /// never tries to render garbage.
    pub fn read_text(&self, rel: &str) -> NestResult<FileContent> {
        let inner = self.inner.lock().expect("workspace mutex");
        let rel = normalize_rel(rel);
        if rel == "." {
            return Err(NestError::validation("cannot open the project root as a file"));
        }

        let metadata = inner.files.metadata(&rel)?;
        if metadata.is_dir {
            return Err(NestError::validation(format!("{rel} is a directory")));
        }
        if metadata.len > MAX_READ_BYTES {
            return Err(NestError::validation(format!(
                "{rel} is too large to open ({} KiB, limit {} KiB)",
                metadata.len / 1024,
                MAX_READ_BYTES / 1024
            )));
        }

        let bytes = inner.files.read_bytes(&rel)?;
        if bytes.contains(&0) {
            return Err(NestError::validation(format!(
                "{rel} looks like a binary file"
            )));
        }
        let content = String::from_utf8(bytes)
            .map_err(|_| NestError::validation(format!("{rel} is not valid UTF-8")))?;

        Ok(FileContent {
            rel_path: rel,
            content,
        })
    }
}

fn build_glob_set(raw: Option<&str>) -> NestResult<Option<GlobSet>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    let mut any = false;
    for part in raw
        .split(|ch| ch == ',' || ch == '\n')
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        any = true;
        let glob = Glob::new(part).map_err(|error| {
            NestError::validation(format!("invalid glob '{part}': {error}"))
        })?;
        builder.add(glob);
    }
    if !any {
        return Ok(None);
    }
    builder
        .build()
        .map(Some)
        .map_err(|error| NestError::validation(format!("invalid glob set: {error}")))
}

fn build_matcher(
    needle: &str,
    match_case: bool,
    whole_word: bool,
    use_regex: bool,
) -> NestResult<Regex> {
    let pattern = if use_regex {
        needle.to_string()
    } else {
        regex::escape(needle)
    };
    let pattern = if whole_word {
        format!(r"\b(?:{pattern})\b")
    } else {
        pattern
    };

    RegexBuilder::new(&pattern)
        .case_insensitive(!match_case)
        .build()
        .map_err(|error| NestError::validation(format!("invalid search pattern: {error}")))
}

impl Inner {
    fn open(root: PathBuf) -> NestResult<Self> {
        let root = root
            .canonicalize()
            .map_err(|error| NestError::config(format!("invalid project root: {error}")))?;
        if !root.is_dir() {
            return Err(NestError::config(format!(
                "project root is not a directory: {}",
                root.display()
            )));
        }
        let name = root
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("project")
            .to_string();
        let files = FileService::with_config(FileServiceConfig::scoped(&root))?;
        Ok(Self { root, name, files })
    }
}

/// Collapses empty / `"./"` prefixes to the canonical relative form.
fn normalize_rel(rel: &str) -> String {
    let trimmed = rel.trim().trim_start_matches("./").trim_matches('/');
    if trimmed.is_empty() {
        ".".to_string()
    } else {
        trimmed.replace('\\', "/")
    }
}

fn is_ignored(name: &str) -> bool {
    DEFAULT_IGNORE.contains(&name)
}

/// Rejects operating on the project root itself.
fn reject_root(rel: &str, verb: &str) -> NestResult<()> {
    if rel == "." {
        return Err(NestError::validation(format!("cannot {verb} the project root")));
    }
    Ok(())
}

/// Recursively copies a directory `from` → `to` (both project-relative).
fn copy_dir_tree(files: &FileService, from: &str, to: &str) -> NestResult<()> {
    files.create_dir_all(to)?;
    for entry in files.list_dir(from)? {
        let from_child = format!("{from}/{}", entry.name);
        let to_child = format!("{to}/{}", entry.name);
        if entry.metadata.is_dir {
            copy_dir_tree(files, &from_child, &to_child)?;
        } else if entry.metadata.is_file {
            files.copy(&from_child, &to_child)?;
        }
    }
    Ok(())
}

/// Opens the OS file manager focused on `abs` (best-effort, platform-specific).
fn reveal_in_file_manager(abs: &Path) -> NestResult<()> {
    use std::process::{Command, Stdio};

    let spawn = |mut command: Command| -> NestResult<()> {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|error| NestError::io(format!("failed to open file manager: {error}")))
    };

    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("open");
        if abs.is_dir() {
            command.arg(abs);
        } else {
            command.arg("-R").arg(abs);
        }
        return spawn(command);
    }

    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("explorer");
        if abs.is_dir() {
            command.arg(abs);
        } else {
            command.arg(format!("/select,{}", abs.display()));
        }
        return spawn(command);
    }

    #[cfg(target_os = "linux")]
    {
        let folder = if abs.is_dir() {
            abs.to_path_buf()
        } else {
            abs.parent().unwrap_or(abs).to_path_buf()
        };
        let mut command = Command::new("xdg-open");
        command.arg(&folder);
        return spawn(command);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = abs;
        Err(NestError::validation(
            "open containing folder is not supported on this platform",
        ))
    }
}

fn resolve_root(config_path: Option<&Path>) -> NestResult<PathBuf> {
    if let Ok(value) = std::env::var(PROJECT_ROOT_ENV) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    if let Some(root) = configured_root(config_path) {
        if root.is_dir() {
            return Ok(root);
        }
    }

    Ok(auto_detect_root())
}

/// Reads `[project].root` from the Kiwi config file, if present.
fn configured_root(config_path: Option<&Path>) -> Option<PathBuf> {
    let path = config_path?;
    let text = std::fs::read_to_string(path).ok()?;
    let document: toml::Value = text.parse().ok()?;
    let raw = document
        .get("project")
        .and_then(|section| section.get("root"))
        .and_then(|value| value.as_str())?
        .trim()
        .to_string();
    if raw.is_empty() {
        return None;
    }

    let candidate = Path::new(&raw);
    if candidate.is_absolute() {
        return Some(candidate.to_path_buf());
    }
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    Some(base.join(candidate))
}

/// Walks up from the current directory to the nearest project marker.
fn auto_detect_root() -> PathBuf {
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut current = start.as_path();
    loop {
        if is_project_marker(current) {
            return current.to_path_buf();
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }
    start
}

fn is_project_marker(path: &Path) -> bool {
    if path.join(".git").exists() {
        return true;
    }
    let cargo = path.join("Cargo.toml");
    cargo.is_file()
        && std::fs::read_to_string(&cargo)
            .map(|content| content.contains("[workspace]"))
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn list_hides_ignored_and_sorts_dirs_first() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "workspace").unwrap();

        let workspace = Workspace {
            inner: Mutex::new(Inner::open(dir.path().to_path_buf()).unwrap()),
        };
        let names: Vec<_> = workspace
            .list(".")
            .unwrap()
            .into_iter()
            .map(|entry| entry.name)
            .collect();
        assert_eq!(names, vec!["src", "Cargo.toml"]);
    }

    #[test]
    fn read_text_rejects_binary() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("blob.bin"), [0_u8, 1, 2, 3]).unwrap();

        let workspace = Workspace {
            inner: Mutex::new(Inner::open(dir.path().to_path_buf()).unwrap()),
        };
        assert!(workspace.read_text("blob.bin").is_err());
    }

    #[test]
    fn read_text_reads_utf8() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("hello.txt"), "hi").unwrap();

        let workspace = Workspace {
            inner: Mutex::new(Inner::open(dir.path().to_path_buf()).unwrap()),
        };
        let file = workspace.read_text("hello.txt").unwrap();
        assert_eq!(file.content, "hi");
        assert_eq!(file.rel_path, "hello.txt");
    }

    fn workspace_at(root: &std::path::Path) -> Workspace {
        Workspace {
            inner: Mutex::new(Inner::open(root.to_path_buf()).unwrap()),
        }
    }

    #[test]
    fn create_file_and_dir() {
        let dir = tempdir().unwrap();
        let ws = workspace_at(dir.path());

        ws.create_dir("src").unwrap();
        assert!(dir.path().join("src").is_dir());
        ws.create_file("src/main.rs").unwrap();
        assert!(dir.path().join("src/main.rs").is_file());

        assert!(ws.create_file("src/main.rs").is_err(), "duplicate rejected");
        assert!(ws.create_dir(".").is_err(), "root rejected");
    }

    #[test]
    fn write_text_saves_and_rejects_root() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "old").unwrap();
        let ws = workspace_at(dir.path());

        ws.write_text("a.txt", "new contents").unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "new contents"
        );

        ws.write_text("created.txt", "fresh").unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("created.txt")).unwrap(),
            "fresh"
        );

        assert!(ws.write_text(".", "x").is_err(), "root rejected");
    }

    #[test]
    fn rename_moves_entry() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "x").unwrap();
        let ws = workspace_at(dir.path());

        ws.rename("a.txt", "b.txt").unwrap();
        assert!(!dir.path().join("a.txt").exists());
        assert!(dir.path().join("b.txt").is_file());
    }

    #[test]
    fn copy_then_delete_directory_tree() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/sub")).unwrap();
        fs::write(dir.path().join("src/sub/f.txt"), "y").unwrap();
        let ws = workspace_at(dir.path());

        ws.copy("src", "src-copy").unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("src-copy/sub/f.txt")).unwrap(),
            "y"
        );
        assert!(dir.path().join("src/sub/f.txt").exists());

        ws.delete("src-copy").unwrap();
        assert!(!dir.path().join("src-copy").exists());
        assert!(ws.delete(".").is_err(), "root rejected");
    }
}
