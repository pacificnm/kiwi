//! Workspace switching — open folder dialog and project reload.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use nest_core::AppContext;
use nest_error::NestResult;
use nest_file::{FileService, FileServiceConfig};
use nest_gui::StatusBarService;

use crate::project::{ProjectConfig, RecentProjects};
use crate::workbench::editor::EditorState;
use crate::workbench::explorer::ExplorerState;
use crate::workbench::source_control::SourceControlState;
use crate::workbench::state::WorkbenchState;
use crate::workbench::watcher::ProjectWatcher;

/// Result of a native folder picker on a background thread.
#[derive(Debug)]
pub enum FolderDialogEvent {
    /// User selected a directory.
    Selected(PathBuf),
    /// Dialog was cancelled.
    Cancelled,
}

/// Opens a native folder picker on a background thread.
pub fn spawn_folder_dialog() -> Receiver<FolderDialogEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match rfd::FileDialog::new().pick_folder() {
            Some(path) => FolderDialogEvent::Selected(path),
            None => FolderDialogEvent::Cancelled,
        };
        let _ = tx.send(event);
    });
    rx
}

/// Applies a new workspace root to workbench state and related shell services.
pub fn switch_workspace(
    state: &mut WorkbenchState,
    project_watcher: &mut Option<ProjectWatcher>,
    recent: &mut RecentProjects,
    app_ctx: &AppContext,
    root: PathBuf,
) -> NestResult<()> {
    let project = ProjectConfig::from_root(root)?;
    let files = FileService::with_config(FileServiceConfig::scoped(project.root.clone()))?;

    state.project = project.clone();
    state.files = files;
    state.explorer = ExplorerState::new(
        &project.root,
        &project.name,
        project.ignore.clone(),
    );
    state.editor = EditorState::empty();
    state.terminal.set_cwd(project.root.clone());
    state.source_control = SourceControlState::new(project.root.clone());
    state.source_control.request_refresh(&project.root);
    state.source_control.request_branch_list(&project.root);
    state.issues = crate::workbench::issues::IssuesState::new();

    *project_watcher = ProjectWatcher::new(&project.root, project.ignore.clone()).ok();

    recent.record(&project.root)?;

    if let Ok(status) = app_ctx.service::<StatusBarService>() {
        status.set(format!(
            "Opened folder: {} ({})",
            project.name,
            project.root.display()
        ));
    }

    tracing::info!(
        target: "kiwi",
        root = %project.root.display(),
        name = %project.name,
        "Workspace opened"
    );

    Ok(())
}
