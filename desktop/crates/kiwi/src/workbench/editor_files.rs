//! Async file loading and saving for editor tabs.

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use nest_error::NestError;
use nest_file::FileService;

use super::editor::{EditorState, EditorTab, EditorTabView};
use crate::workbench::source_control::{spawn_git_diff, DiffSide, GitDiffEvent};

/// Result of a background file read for one editor tab.
#[derive(Debug)]
pub enum FileLoadEvent {
    /// Loaded UTF-8 text for the tab at the given index.
    Loaded {
        /// Tab index.
        tab_index: usize,
        /// File contents.
        content: String,
    },
    /// Failed to read the file.
    Failed {
        /// Tab index.
        tab_index: usize,
        /// Error message.
        error: String,
    },
}

/// Result of a background file write for one editor tab.
#[derive(Debug)]
pub enum FileSaveEvent {
    /// Saved the tab at the given index.
    Saved {
        /// Tab index.
        tab_index: usize,
        /// Contents written to disk.
        content: String,
    },
    /// Failed to write the file.
    Failed {
        /// Tab index.
        tab_index: usize,
        /// Error message.
        error: String,
    },
}

/// Reads a file on a background thread using a cloned [`FileService`].
pub fn spawn_read_file(
    files: FileService,
    rel_path: String,
    tab_index: usize,
) -> mpsc::Receiver<FileLoadEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match files.read_text(&rel_path) {
            Ok(content) => FileLoadEvent::Loaded {
                tab_index,
                content,
            },
            Err(error) => FileLoadEvent::Failed {
                tab_index,
                error: error.to_string(),
            },
        };
        let _ = tx.send(event);
    });
    rx
}

/// Writes a file on a background thread using a cloned [`FileService`].
pub fn spawn_save_file(
    files: FileService,
    rel_path: String,
    content: String,
    tab_index: usize,
) -> mpsc::Receiver<FileSaveEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match files.write_text(&rel_path, &content) {
            Ok(()) => FileSaveEvent::Saved {
                tab_index,
                content,
            },
            Err(error) => FileSaveEvent::Failed {
                tab_index,
                error: error.to_string(),
            },
        };
        let _ = tx.send(event);
    });
    rx
}

/// Opens or focuses a source tab and returns the tab index when a load was started.
pub fn open_file_tab(editor: &mut EditorState, rel_path: String, abs_path: PathBuf) -> Option<usize> {
    if let Some(index) = editor.tabs.iter().position(|tab| {
        tab.rel_path == rel_path && tab.view == EditorTabView::Source
    }) {
        editor.active_tab = index;
        return None;
    }

    editor.tabs.push(EditorTab {
        rel_path: rel_path.clone(),
        abs_path,
        content: String::new(),
        saved_content: String::new(),
        dirty: false,
        loading: true,
        saving: false,
        error: None,
        save_error: None,
        view: EditorTabView::Source,
    });
    editor.active_tab = editor.tabs.len() - 1;
    Some(editor.active_tab)
}

/// Opens or focuses a read-only diff tab and returns the tab index when a load was started.
pub fn open_diff_tab(
    editor: &mut EditorState,
    rel_path: String,
    abs_path: PathBuf,
    staged: bool,
) -> Option<usize> {
    let view = EditorTabView::Diff { staged };
    if let Some(index) = editor.tabs.iter().position(|tab| tab.rel_path == rel_path && tab.view == view)
    {
        editor.active_tab = index;
        let tab = &mut editor.tabs[index];
        tab.loading = true;
        tab.error = None;
        return Some(index);
    }

    editor.tabs.push(EditorTab {
        rel_path: rel_path.clone(),
        abs_path,
        content: String::new(),
        saved_content: String::new(),
        dirty: false,
        loading: true,
        saving: false,
        error: None,
        save_error: None,
        view,
    });
    editor.active_tab = editor.tabs.len() - 1;
    Some(editor.active_tab)
}

/// Reads a git diff on a background thread and reports via [`FileLoadEvent`].
pub fn spawn_git_diff_load(
    root: PathBuf,
    rel_path: String,
    side: DiffSide,
    tab_index: usize,
) -> mpsc::Receiver<FileLoadEvent> {
    let git_rx = spawn_git_diff(root, rel_path, side);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match git_rx.recv() {
            Ok(GitDiffEvent::Ready { path, diff }) => {
                let _ = path;
                FileLoadEvent::Loaded {
                    tab_index,
                    content: diff,
                }
            }
            Ok(GitDiffEvent::Failed { path, error }) => FileLoadEvent::Failed {
                tab_index,
                error: format!("{path}: {error}"),
            },
            Err(_) => FileLoadEvent::Failed {
                tab_index,
                error: "Git diff interrupted".into(),
            },
        };
        let _ = tx.send(event);
    });
    rx
}

/// Applies a completed file or diff load to the target tab.
pub fn apply_file_load(editor: &mut EditorState, event: FileLoadEvent) {
    match event {
        FileLoadEvent::Loaded { tab_index, content } => {
            if let Some(tab) = editor.tabs.get_mut(tab_index) {
                tab.content = content;
                if tab.view == EditorTabView::Source {
                    tab.saved_content = tab.content.clone();
                }
                tab.dirty = false;
                tab.loading = false;
                tab.error = None;
                tab.save_error = None;
            }
        }
        FileLoadEvent::Failed { tab_index, error } => {
            if let Some(tab) = editor.tabs.get_mut(tab_index) {
                tab.loading = false;
                tab.error = Some(error);
            }
        }
    }
}

/// Marks a tab as saving before dispatching a background write.
pub fn begin_file_save(editor: &mut EditorState, tab_index: usize) -> Option<(String, String)> {
    let tab = editor.tabs.get_mut(tab_index)?;
    if tab.loading || tab.saving || !tab.dirty {
        return None;
    }
    tab.saving = true;
    tab.save_error = None;
    Some((tab.rel_path.clone(), tab.content.clone()))
}

/// Applies a completed file save to the target tab.
pub fn apply_file_save(editor: &mut EditorState, event: FileSaveEvent) {
    match event {
        FileSaveEvent::Saved { tab_index, content } => {
            if let Some(tab) = editor.tabs.get_mut(tab_index) {
                tab.saved_content = content;
                tab.dirty = tab.content != tab.saved_content;
                tab.saving = false;
                tab.save_error = None;
            }
        }
        FileSaveEvent::Failed { tab_index, error } => {
            if let Some(tab) = editor.tabs.get_mut(tab_index) {
                tab.saving = false;
                tab.save_error = Some(error);
            }
        }
    }
}

/// Maps a relative project path to an absolute path for display/tooling.
pub fn abs_path_for_rel(project_root: &std::path::Path, rel_path: &str) -> PathBuf {
    if rel_path == "." {
        project_root.to_path_buf()
    } else {
        project_root.join(rel_path)
    }
}

/// Validates that a relative path is safe to request from scoped file I/O.
pub fn validate_rel_path(rel_path: &str) -> Result<(), NestError> {
    if rel_path.contains("..") {
        return Err(NestError::validation("path traversal is not allowed"));
    }
    Ok(())
}
