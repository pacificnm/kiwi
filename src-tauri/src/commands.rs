//! Kiwi IPC commands.
//!
//! Product commands are invoked from the UI as `plugin:kiwi|<command>`.

use std::path::PathBuf;

use nest_error::{NestError, NestResult};
use nest_logging::ui_buffer;
use nest_tauri::NestHostState;
use nest_theme::{ThemeDefinition, ThemeId, ThemeService};
use serde::Serialize;
use tauri::{plugin::TauriPlugin, AppHandle, Runtime, State};

use crate::accounts::{self, AccountStatus};
use crate::agent::AgentPty;
use crate::agent_config::{AgentConfig, AgentSettings};
use crate::docs::{self, DocEntry};
use crate::git::{self, GitCommit, GitCommitChanges, GitStatus};
use crate::github::{
    self, GitHubAuthStatus, GitHubIssue, GitHubIssueActionResult, GitHubIssueListItem,
    GitHubLabel, GitHubMilestone, GitHubRepoInfo,
};
use crate::swift::{
    SwiftDb, SwiftProjectSummary, SwiftStatus, SwiftTaskDetailResponse, SwiftTasksOverview,
    SwiftWorkspaceLinkSummary,
};
use crate::problems::{self, ProblemsReport, ProblemsState};
use crate::mcp::{self, McpOverview};
use crate::ollama::{self, OllamaAuthStatus, OllamaModel};
use crate::terminal::TerminalManager;
use crate::workspace::{
    FileContent, FsEntry, Workspace, WorkspaceInfo, WorkspaceReplaceRequest, WorkspaceReplaceResponse,
    WorkspaceSearchQuery, WorkspaceSearchResponse,
};

/// Host metadata exposed to the workbench UI.
#[derive(Debug, Serialize)]
pub struct KiwiHostInfo {
    /// Product name.
    pub name: String,
    /// Desktop shell generation.
    pub shell: String,
}

/// One line in the bottom-panel Logs viewer.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub level: String,
    pub target: String,
    pub message: String,
    pub timestamp: String,
}

/// Returns buffered tracing log lines for the Logs panel.
#[tauri::command]
fn logs_snapshot() -> Vec<LogLine> {
    ui_buffer()
        .map(|buffer| {
            buffer
                .snapshot()
                .into_iter()
                .map(|record| LogLine {
                    level: record.level.to_string(),
                    target: record.target,
                    message: record.message,
                    timestamp: record.timestamp,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Clears the in-memory log buffer (does not delete log files on disk).
#[tauri::command]
fn logs_clear() -> NestResult<()> {
    let Some(buffer) = ui_buffer() else {
        return Ok(());
    };
    buffer.clear();
    Ok(())
}

/// Returns the latest workspace diagnostics for the Problems panel.
#[tauri::command]
fn problems_snapshot(problems: State<'_, ProblemsState>) -> ProblemsReport {
    problems.snapshot()
}

/// Runs Rust / TypeScript / ESLint diagnostics and updates the Problems snapshot.
#[tauri::command]
async fn problems_run(
    problems: State<'_, ProblemsState>,
    workspace: State<'_, Workspace>,
) -> NestResult<ProblemsReport> {
    problems.set_running(true);
    let root = workspace.root();
    let collected = tauri::async_runtime::spawn_blocking(move || problems::collect(&root))
        .await
        .map_err(|error| NestError::task(format!("diagnostics task failed: {error}")))??;

    let error_count = collected
        .iter()
        .filter(|item| item.severity == "error")
        .count();
    let warning_count = collected
        .iter()
        .filter(|item| item.severity == "warning")
        .count();
    let summary = if collected.is_empty() {
        "No problems detected.".into()
    } else {
        format!("{error_count} errors, {warning_count} warnings")
    };
    problems.set_report(collected, summary);
    Ok(problems.snapshot())
}

/// Swift DB status for the Tasks panel.
#[tauri::command]
async fn swift_status(swift: State<'_, SwiftDb>) -> NestResult<SwiftStatus> {
    Ok(swift.status().await)
}

/// Loads Swift projects, workspace link, and tasks for the open repo.
#[tauri::command]
async fn swift_tasks_overview(
    swift: State<'_, SwiftDb>,
    workspace: State<'_, Workspace>,
) -> NestResult<SwiftTasksOverview> {
    swift.overview(&workspace.root()).await
}

/// Lists Swift projects for linking a workspace.
#[tauri::command]
async fn swift_list_projects(swift: State<'_, SwiftDb>) -> NestResult<Vec<SwiftProjectSummary>> {
    swift.list_projects().await
}

/// Links the open workspace to a Swift project.
#[tauri::command]
async fn swift_link_workspace(
    swift: State<'_, SwiftDb>,
    workspace: State<'_, Workspace>,
    project_id: String,
) -> NestResult<SwiftWorkspaceLinkSummary> {
    let project_id = uuid::Uuid::parse_str(project_id.trim())
        .map_err(|error| NestError::validation(format!("invalid project id: {error}")))?;
    swift.link_workspace(&workspace.root(), project_id).await
}

/// Removes the Swift project link for the open workspace.
#[tauri::command]
async fn swift_unlink_workspace(
    swift: State<'_, SwiftDb>,
    workspace: State<'_, Workspace>,
) -> NestResult<()> {
    swift.unlink_workspace(&workspace.root()).await
}

/// Loads a Swift task with project context and subtasks for the editor detail view.
#[tauri::command]
async fn swift_get_task(swift: State<'_, SwiftDb>, task_id: String) -> NestResult<SwiftTaskDetailResponse> {
    let task_id = uuid::Uuid::parse_str(task_id.trim())
        .map_err(|error| NestError::validation(format!("invalid task id: {error}")))?;
    swift.get_task(task_id).await
}

/// Loads MCP servers, tools, and agent configuration for the Tool Activity panel.
#[tauri::command]
async fn mcp_overview(agent_config: State<'_, AgentConfig>) -> NestResult<McpOverview> {
    tracing::info!(target: "kiwi", "mcp_overview: enter");
    let result = mcp::overview(agent_config.config_path()).await;
    match &result {
        Ok(overview) => tracing::info!(
            target: "kiwi",
            servers = overview.servers.len(),
            "mcp_overview: ok"
        ),
        Err(error) => tracing::error!(target: "kiwi", %error, "mcp_overview: err"),
    }
    result
}

/// Lists Kiwi's documentation entries for the Help Activity panel.
#[tauri::command]
fn docs_list() -> NestResult<Vec<DocEntry>> {
    tracing::info!(target: "kiwi", "docs_list: enter");
    let result = docs::list();
    match &result {
        Ok(entries) => tracing::info!(target: "kiwi", count = entries.len(), "docs_list: ok"),
        Err(error) => tracing::error!(target: "kiwi", %error, "docs_list: err"),
    }
    result
}

/// Reads one of Kiwi's documentation files for the Help Activity panel.
#[tauri::command]
fn docs_read(path: String) -> NestResult<String> {
    tracing::info!(target: "kiwi", %path, "docs_read: enter");
    let result = docs::read(&path);
    match &result {
        Ok(content) => tracing::info!(target: "kiwi", bytes = content.len(), "docs_read: ok"),
        Err(error) => tracing::error!(target: "kiwi", %error, "docs_read: err"),
    }
    result
}

/// Lists all registered themes with full token data, for the Theme Activity panel.
#[tauri::command]
fn themes_list(state: State<'_, NestHostState>) -> NestResult<Vec<ThemeDefinition>> {
    tracing::info!(target: "kiwi", "themes_list: enter");
    let themes = state.context.service::<ThemeService>()?;
    let result: NestResult<Vec<ThemeDefinition>> =
        themes.list_themes().iter().map(|id| themes.theme(id)).collect();
    match &result {
        Ok(list) => tracing::info!(target: "kiwi", count = list.len(), "themes_list: ok"),
        Err(error) => tracing::error!(target: "kiwi", %error, "themes_list: err"),
    }
    result
}

/// Switches Kiwi's active theme. The caller must re-invoke `nest_theme_css`
/// (see `nest-tauri`) and re-apply the returned root block to take effect.
#[tauri::command]
fn theme_set_active(state: State<'_, NestHostState>, id: String) -> NestResult<()> {
    tracing::info!(target: "kiwi", %id, "theme_set_active: enter");
    let themes = state.context.service::<ThemeService>()?;
    let result = themes.set_active_theme(&ThemeId::from(id));
    match &result {
        Ok(()) => tracing::info!(target: "kiwi", "theme_set_active: ok"),
        Err(error) => tracing::error!(target: "kiwi", %error, "theme_set_active: err"),
    }
    result
}

/// Returns Kiwi desktop host metadata.
#[tauri::command]
fn kiwi_host_info() -> KiwiHostInfo {
    tracing::info!(target: "kiwi", "kiwi_host_info: enter");
    let info = KiwiHostInfo {
        name: "kiwi".into(),
        shell: "tauri-workbench-v1".into(),
    };
    tracing::info!(target: "kiwi", shell = %info.shell, "kiwi_host_info: ok");
    info
}

/// Launches `ollama launch <runtime> --model <model>` in the agent PTY.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn agent_launch<R: Runtime>(
    app: AppHandle<R>,
    agent: State<'_, AgentPty>,
    runtime: String,
    model: String,
    ollama_host: Option<String>,
    cwd: Option<String>,
    direct: Option<bool>,
    rows: u16,
    cols: u16,
) -> NestResult<()> {
    let direct = direct.unwrap_or(false);
    tracing::info!(
        target: "kiwi",
        %runtime,
        %model,
        direct,
        ollama_host = ollama_host.as_deref().unwrap_or("(local)"),
        cwd = cwd.as_deref().unwrap_or("(default)"),
        rows,
        cols,
        "agent_launch: enter"
    );
    let result = agent.launch(
        app,
        &runtime,
        &model,
        ollama_host.as_deref(),
        cwd.map(PathBuf::from),
        direct,
        rows,
        cols,
    );
    match &result {
        Ok(()) => tracing::info!(target: "kiwi", %runtime, "agent_launch: ok"),
        Err(error) => tracing::error!(target: "kiwi", %runtime, %error, "agent_launch: err"),
    }
    result
}

/// Sends keystrokes / input to the running agent.
#[tauri::command]
fn agent_input(agent: State<'_, AgentPty>, data: String) -> NestResult<()> {
    tracing::debug!(target: "kiwi", bytes = data.len(), "agent_input: enter");
    let result = agent.input(&data);
    if let Err(error) = &result {
        tracing::error!(target: "kiwi", %error, "agent_input: err");
    }
    result
}

/// Resizes the agent PTY to match the terminal viewport.
#[tauri::command]
fn agent_resize(agent: State<'_, AgentPty>, rows: u16, cols: u16) -> NestResult<()> {
    tracing::debug!(target: "kiwi", rows, cols, "agent_resize: enter");
    let result = agent.resize(rows, cols);
    if let Err(error) = &result {
        tracing::error!(target: "kiwi", %error, "agent_resize: err");
    }
    result
}

/// Stops the running agent session.
#[tauri::command]
fn agent_stop(agent: State<'_, AgentPty>) -> NestResult<()> {
    tracing::info!(target: "kiwi", "agent_stop: enter");
    agent.stop();
    tracing::info!(target: "kiwi", "agent_stop: ok");
    Ok(())
}

/// Reports whether an agent session is currently running.
#[tauri::command]
fn agent_status(agent: State<'_, AgentPty>) -> bool {
    let running = agent.is_running();
    tracing::debug!(target: "kiwi", running, "agent_status");
    running
}

/// Returns agent sidebar settings (endpoint, models, default runtime).
#[tauri::command]
fn agent_settings_get(config: State<'_, AgentConfig>) -> AgentSettings {
    tracing::debug!(target: "kiwi", "agent_settings_get");
    config.get()
}

/// Updates agent sidebar settings in memory and persists to config.toml.
#[tauri::command]
fn agent_settings_save(
    config: State<'_, AgentConfig>,
    settings: AgentSettings,
) -> NestResult<AgentSettings> {
    tracing::info!(
        target: "kiwi",
        host = %settings.host,
        model = %settings.model,
        runtime = %settings.runtime,
        "agent_settings_save"
    );
    config.set(settings);
    config.save()
}

/// Lists models on the Ollama inference server (`ollama list` via `OLLAMA_HOST`).
#[tauri::command]
fn ollama_list_models(host: String) -> NestResult<Vec<OllamaModel>> {
    tracing::info!(target: "kiwi", %host, "ollama_list_models");
    ollama::list_models(&host)
}

/// Returns ollama.com sign-in status for the configured inference server.
#[tauri::command]
fn ollama_auth_status(host: String) -> OllamaAuthStatus {
    ollama::auth_status(&host)
}

/// Opens the ollama.com sign-in flow in the system browser.
#[tauri::command]
fn ollama_signin(host: String) -> NestResult<()> {
    tracing::info!(target: "kiwi", %host, "ollama_signin");
    ollama::sign_in(&host)
}

/// Signs out of ollama.com on the configured Ollama server.
#[tauri::command]
fn ollama_signout(host: String) -> NestResult<()> {
    tracing::info!(target: "kiwi", %host, "ollama_signout");
    ollama::sign_out(&host)
}

/// Returns Codex (`codex login status`) account state for direct mode.
#[tauri::command]
fn codex_account_status() -> AccountStatus {
    accounts::codex_status()
}

/// Launches `codex login` (opens browser OAuth) for direct account mode.
#[tauri::command]
fn codex_login() -> NestResult<()> {
    tracing::info!(target: "kiwi", "codex_login");
    accounts::codex_login()
}

/// Runs `codex logout` to clear stored credentials.
#[tauri::command]
fn codex_logout() -> NestResult<()> {
    tracing::info!(target: "kiwi", "codex_logout");
    accounts::codex_logout()
}

/// Opens an interactive shell terminal in the bottom panel, keyed by `id`.
#[tauri::command]
fn terminal_open<R: Runtime>(
    app: AppHandle<R>,
    terminals: State<'_, TerminalManager>,
    id: String,
    cwd: Option<String>,
    shell: Option<String>,
    rows: u16,
    cols: u16,
) -> NestResult<()> {
    tracing::info!(
        target: "kiwi",
        %id,
        cwd = cwd.as_deref().unwrap_or("(default)"),
        shell = shell.as_deref().unwrap_or("(auto)"),
        rows,
        cols,
        "terminal_open: enter"
    );
    let result = terminals.open(
        app,
        id.clone(),
        cwd.map(PathBuf::from),
        shell.as_deref(),
        rows,
        cols,
    );
    match &result {
        Ok(()) => tracing::info!(target: "kiwi", %id, "terminal_open: ok"),
        Err(error) => tracing::error!(target: "kiwi", %id, %error, "terminal_open: err"),
    }
    result
}

/// Sends keystrokes to a terminal session.
#[tauri::command]
fn terminal_input(terminals: State<'_, TerminalManager>, id: String, data: String) -> NestResult<()> {
    tracing::debug!(target: "kiwi", %id, bytes = data.len(), "terminal_input");
    terminals.input(&id, &data)
}

/// Resizes a terminal session's PTY.
#[tauri::command]
fn terminal_resize(
    terminals: State<'_, TerminalManager>,
    id: String,
    rows: u16,
    cols: u16,
) -> NestResult<()> {
    tracing::debug!(target: "kiwi", %id, rows, cols, "terminal_resize");
    terminals.resize(&id, rows, cols)
}

/// Closes a terminal session (hangs up its shell).
#[tauri::command]
fn terminal_close(terminals: State<'_, TerminalManager>, id: String) -> NestResult<()> {
    tracing::info!(target: "kiwi", %id, "terminal_close");
    terminals.close(&id);
    Ok(())
}

/// Lists the ids of all live terminal sessions.
#[tauri::command]
fn terminal_list(terminals: State<'_, TerminalManager>) -> Vec<String> {
    terminals.list()
}

/// Returns metadata about the active workspace (root + display name).
#[tauri::command]
fn workspace_info(workspace: State<'_, Workspace>) -> WorkspaceInfo {
    tracing::info!(target: "kiwi", "workspace_info: enter");
    let info = workspace.info();
    tracing::info!(target: "kiwi", root = %info.root, "workspace_info: ok");
    info
}

/// Lists a directory relative to the project root for the Explorer tree.
#[tauri::command]
fn workspace_list(workspace: State<'_, Workspace>, rel: String) -> NestResult<Vec<FsEntry>> {
    let started = std::time::Instant::now();
    tracing::info!(target: "kiwi", %rel, "workspace_list: enter");
    let result = workspace.list(&rel);
    match &result {
        Ok(entries) => tracing::info!(
            target: "kiwi",
            %rel,
            count = entries.len(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "workspace_list: ok"
        ),
        Err(error) => tracing::error!(target: "kiwi", %rel, %error, "workspace_list: err"),
    }
    result
}

/// Reads a UTF-8 text file relative to the project root for the editor.
#[tauri::command]
fn workspace_read(workspace: State<'_, Workspace>, rel: String) -> NestResult<FileContent> {
    let started = std::time::Instant::now();
    tracing::info!(target: "kiwi", %rel, "workspace_read: enter");
    let result = workspace.read_text(&rel);
    match &result {
        Ok(file) => tracing::info!(
            target: "kiwi",
            %rel,
            bytes = file.content.len(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "workspace_read: ok"
        ),
        Err(error) => tracing::error!(target: "kiwi", %rel, %error, "workspace_read: err"),
    }
    result
}

/// Writes editor `content` back to `rel` (Save). Returns the saved rel path.
#[tauri::command]
fn workspace_write(
    workspace: State<'_, Workspace>,
    rel: String,
    content: String,
) -> NestResult<String> {
    tracing::info!(target: "kiwi", %rel, bytes = content.len(), "workspace_write: enter");
    let result = workspace.write_text(&rel, &content);
    match &result {
        Ok(path) => tracing::info!(target: "kiwi", %path, "workspace_write: ok"),
        Err(error) => tracing::error!(target: "kiwi", %rel, %error, "workspace_write: err"),
    }
    result
}

/// Creates an empty file at `rel` (New File). Returns the created rel path.
#[tauri::command]
fn workspace_create_file(workspace: State<'_, Workspace>, rel: String) -> NestResult<String> {
    tracing::info!(target: "kiwi", %rel, "workspace_create_file: enter");
    let result = workspace.create_file(&rel);
    match &result {
        Ok(path) => tracing::info!(target: "kiwi", %path, "workspace_create_file: ok"),
        Err(error) => tracing::error!(target: "kiwi", %rel, %error, "workspace_create_file: err"),
    }
    result
}

/// Creates a directory at `rel` (New Folder). Returns the created rel path.
#[tauri::command]
fn workspace_create_dir(workspace: State<'_, Workspace>, rel: String) -> NestResult<String> {
    tracing::info!(target: "kiwi", %rel, "workspace_create_dir: enter");
    let result = workspace.create_dir(&rel);
    match &result {
        Ok(path) => tracing::info!(target: "kiwi", %path, "workspace_create_dir: ok"),
        Err(error) => tracing::error!(target: "kiwi", %rel, %error, "workspace_create_dir: err"),
    }
    result
}

/// Renames / moves `from` → `to` (Rename, or paste-move). Returns the new rel path.
#[tauri::command]
fn workspace_rename(
    workspace: State<'_, Workspace>,
    from: String,
    to: String,
) -> NestResult<String> {
    tracing::info!(target: "kiwi", %from, %to, "workspace_rename: enter");
    let result = workspace.rename(&from, &to);
    match &result {
        Ok(path) => tracing::info!(target: "kiwi", %path, "workspace_rename: ok"),
        Err(error) => tracing::error!(target: "kiwi", %from, %to, %error, "workspace_rename: err"),
    }
    result
}

/// Deletes a file or directory tree at `rel` (Delete). Returns the removed rel path.
#[tauri::command]
fn workspace_delete(workspace: State<'_, Workspace>, rel: String) -> NestResult<String> {
    tracing::info!(target: "kiwi", %rel, "workspace_delete: enter");
    let result = workspace.delete(&rel);
    match &result {
        Ok(path) => tracing::info!(target: "kiwi", %path, "workspace_delete: ok"),
        Err(error) => tracing::error!(target: "kiwi", %rel, %error, "workspace_delete: err"),
    }
    result
}

/// Copies a file or directory tree `from` → `to` (paste-copy). Returns the new rel path.
#[tauri::command]
fn workspace_copy(
    workspace: State<'_, Workspace>,
    from: String,
    to: String,
) -> NestResult<String> {
    tracing::info!(target: "kiwi", %from, %to, "workspace_copy: enter");
    let result = workspace.copy(&from, &to);
    match &result {
        Ok(path) => tracing::info!(target: "kiwi", %path, "workspace_copy: ok"),
        Err(error) => tracing::error!(target: "kiwi", %from, %to, %error, "workspace_copy: err"),
    }
    result
}

/// Reveals a path in the OS file manager (Open Containing Folder).
#[tauri::command]
fn workspace_reveal(workspace: State<'_, Workspace>, rel: String) -> NestResult<()> {
    tracing::info!(target: "kiwi", %rel, "workspace_reveal: enter");
    let result = workspace.reveal(&rel);
    if let Err(error) = &result {
        tracing::error!(target: "kiwi", %rel, %error, "workspace_reveal: err");
    }
    result
}

/// Switches the workspace to a new root directory (Open Folder).
#[tauri::command]
fn workspace_open(workspace: State<'_, Workspace>, root: String) -> NestResult<WorkspaceInfo> {
    tracing::info!(target: "kiwi", %root, "workspace_open: enter");
    let result = workspace.open(root);
    match &result {
        Ok(info) => tracing::info!(target: "kiwi", root = %info.root, "workspace_open: ok"),
        Err(error) => tracing::error!(target: "kiwi", %error, "workspace_open: err"),
    }
    result
}

/// Searches the workspace (Search sidebar) with include/exclude globs.
#[tauri::command]
fn workspace_search(
    workspace: State<'_, Workspace>,
    query: WorkspaceSearchQuery,
) -> NestResult<WorkspaceSearchResponse> {
    let started = std::time::Instant::now();
    tracing::info!(
        target: "kiwi",
        bytes = query.query.len(),
        match_case = query.match_case,
        whole_word = query.whole_word,
        use_regex = query.use_regex,
        "workspace_search: enter"
    );
    let result = workspace.search(query);
    match &result {
        Ok(value) => tracing::info!(
            target: "kiwi",
            files = value.file_count,
            matches = value.match_count,
            truncated = value.truncated,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "workspace_search: ok"
        ),
        Err(error) => tracing::error!(target: "kiwi", %error, "workspace_search: err"),
    }
    result
}

/// Replaces all matches for a query across the workspace ("Replace All").
#[tauri::command]
fn workspace_replace_all(
    workspace: State<'_, Workspace>,
    request: WorkspaceReplaceRequest,
) -> NestResult<WorkspaceReplaceResponse> {
    let started = std::time::Instant::now();
    tracing::info!(
        target: "kiwi",
        bytes = request.search.query.len(),
        replace_bytes = request.replace.len(),
        "workspace_replace_all: enter"
    );
    let result = workspace.replace_all(request);
    match &result {
        Ok(value) => tracing::info!(
            target: "kiwi",
            files = value.file_count,
            matches = value.match_count,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "workspace_replace_all: ok"
        ),
        Err(error) => tracing::error!(target: "kiwi", %error, "workspace_replace_all: err"),
    }
    result
}

/// Reads the repository status (branch, ahead/behind, changed files).
#[tauri::command]
fn git_status(workspace: State<'_, Workspace>) -> NestResult<GitStatus> {
    tracing::info!(target: "kiwi", "git_status: enter");
    let result = git::status(&workspace.root());
    match &result {
        Ok(status) => tracing::info!(
            target: "kiwi",
            is_repo = status.is_repo,
            branch = %status.branch,
            changes = status.changes.len(),
            "git_status: ok"
        ),
        Err(error) => tracing::error!(target: "kiwi", %error, "git_status: err"),
    }
    result
}

/// Stages one repository-relative path (`git add`).
#[tauri::command]
fn git_stage(workspace: State<'_, Workspace>, path: String) -> NestResult<()> {
    tracing::info!(target: "kiwi", %path, "git_stage: enter");
    git::stage(&workspace.root(), &path)
}

/// Stages all changes (`git add -A`).
#[tauri::command]
fn git_stage_all(workspace: State<'_, Workspace>) -> NestResult<()> {
    tracing::info!(target: "kiwi", "git_stage_all: enter");
    git::stage_all(&workspace.root())
}

/// Unstages one repository-relative path (`git restore --staged`).
#[tauri::command]
fn git_unstage(workspace: State<'_, Workspace>, path: String) -> NestResult<()> {
    tracing::info!(target: "kiwi", %path, "git_unstage: enter");
    git::unstage(&workspace.root(), &path)
}

/// Discards working-tree edits for one path (`git restore`).
#[tauri::command]
fn git_discard(workspace: State<'_, Workspace>, path: String) -> NestResult<()> {
    tracing::info!(target: "kiwi", %path, "git_discard: enter");
    git::discard(&workspace.root(), &path)
}

/// Commits staged changes; stages everything first when `stageAll` is set.
#[tauri::command]
fn git_commit(
    workspace: State<'_, Workspace>,
    message: String,
    stage_all: bool,
) -> NestResult<()> {
    tracing::info!(target: "kiwi", bytes = message.len(), stage_all, "git_commit: enter");
    let result = git::commit(&workspace.root(), &message, stage_all);
    match &result {
        Ok(()) => tracing::info!(target: "kiwi", "git_commit: ok"),
        Err(error) => tracing::error!(target: "kiwi", %error, "git_commit: err"),
    }
    result
}

/// Pushes the current branch to its upstream (or publishes to `origin` on first push).
#[tauri::command]
fn git_push(workspace: State<'_, Workspace>) -> NestResult<()> {
    tracing::info!(target: "kiwi", "git_push: enter");
    let result = git::push(&workspace.root());
    match &result {
        Ok(()) => tracing::info!(target: "kiwi", "git_push: ok"),
        Err(error) => tracing::error!(target: "kiwi", %error, "git_push: err"),
    }
    result
}

/// Pulls from the current branch's upstream.
#[tauri::command]
fn git_pull(workspace: State<'_, Workspace>) -> NestResult<()> {
    tracing::info!(target: "kiwi", "git_pull: enter");
    let result = git::pull(&workspace.root());
    match &result {
        Ok(()) => tracing::info!(target: "kiwi", "git_pull: ok"),
        Err(error) => tracing::error!(target: "kiwi", %error, "git_pull: err"),
    }
    result
}

/// Reads up to `limit` recent commits for the graph section.
#[tauri::command]
fn git_log(workspace: State<'_, Workspace>, limit: Option<u32>) -> NestResult<Vec<GitCommit>> {
    let limit = limit.unwrap_or(100);
    tracing::info!(target: "kiwi", limit, "git_log: enter");
    let result = git::log(&workspace.root(), limit);
    match &result {
        Ok(commits) => tracing::info!(target: "kiwi", count = commits.len(), "git_log: ok"),
        Err(error) => tracing::error!(target: "kiwi", %error, "git_log: err"),
    }
    result
}

/// Loads per-file diffs for a commit (Open Changes view).
#[tauri::command]
fn git_commit_changes(workspace: State<'_, Workspace>, hash: String) -> NestResult<GitCommitChanges> {
    tracing::info!(target: "kiwi", %hash, "git_commit_changes: enter");
    let result = git::commit_changes(&workspace.root(), &hash);
    match &result {
        Ok(changes) => tracing::info!(
            target: "kiwi",
            files = changes.files.len(),
            "git_commit_changes: ok"
        ),
        Err(error) => tracing::error!(target: "kiwi", %error, "git_commit_changes: err"),
    }
    result
}

/// Returns GitHub CLI auth status for the Issues panel.
#[tauri::command]
fn github_auth_status() -> GitHubAuthStatus {
    tracing::debug!(target: "kiwi", "github_auth_status");
    github::auth_status()
}

/// Resolves `owner/repo` from the workspace `origin` remote.
#[tauri::command]
fn github_repo(workspace: State<'_, Workspace>) -> NestResult<Option<GitHubRepoInfo>> {
    tracing::info!(target: "kiwi", "github_repo: enter");
    github::read_repo(&workspace.root())
}

/// Lists GitHub issues for the workspace repository.
#[tauri::command]
fn github_issue_list(
    workspace: State<'_, Workspace>,
    state: Option<String>,
    limit: Option<u32>,
) -> NestResult<Vec<GitHubIssueListItem>> {
    let limit = limit.unwrap_or(100);
    let state = state.as_deref().unwrap_or("open");
    tracing::info!(target: "kiwi", limit, %state, "github_issue_list: enter");
    let repo = github::read_repo(&workspace.root())?
        .ok_or_else(|| NestError::validation("no GitHub origin remote found"))?;
    github::issue_list(&repo.repo, state, limit)
}

/// Loads one GitHub issue (body + metadata) for an editor tab.
#[tauri::command]
fn github_issue_view(workspace: State<'_, Workspace>, number: u64) -> NestResult<GitHubIssue> {
    tracing::info!(target: "kiwi", number, "github_issue_view: enter");
    let repo = github::read_repo(&workspace.root())?
        .ok_or_else(|| NestError::validation("no GitHub origin remote found"))?;
    github::issue_view(&repo.repo, number)
}

/// Creates a GitHub issue in the workspace repository.
#[tauri::command]
fn github_issue_create(
    workspace: State<'_, Workspace>,
    title: String,
    body: String,
) -> NestResult<GitHubIssueActionResult> {
    tracing::info!(target: "kiwi", bytes = title.len(), "github_issue_create: enter");
    let repo = github::read_repo(&workspace.root())?
        .ok_or_else(|| NestError::validation("no GitHub origin remote found"))?;
    github::issue_create(&repo.repo, &title, &body)
}

/// Posts a comment on a GitHub issue.
#[tauri::command]
fn github_issue_comment(
    workspace: State<'_, Workspace>,
    number: u64,
    body: String,
) -> NestResult<GitHubIssueActionResult> {
    tracing::info!(target: "kiwi", number, bytes = body.len(), "github_issue_comment: enter");
    let repo = github::read_repo(&workspace.root())?
        .ok_or_else(|| NestError::validation("no GitHub origin remote found"))?;
    github::issue_comment(&repo.repo, number, &body)
}

/// Lists repository labels.
#[tauri::command]
fn github_label_list(workspace: State<'_, Workspace>) -> NestResult<Vec<GitHubLabel>> {
    tracing::info!(target: "kiwi", "github_label_list: enter");
    let repo = github::read_repo(&workspace.root())?
        .ok_or_else(|| NestError::validation("no GitHub origin remote found"))?;
    github::label_list(&repo.repo)
}

/// Lists open milestones.
#[tauri::command]
fn github_milestone_list(workspace: State<'_, Workspace>) -> NestResult<Vec<GitHubMilestone>> {
    tracing::info!(target: "kiwi", "github_milestone_list: enter");
    let repo = github::read_repo(&workspace.root())?
        .ok_or_else(|| NestError::validation("no GitHub origin remote found"))?;
    github::milestone_list(&repo.repo)
}

/// Registers the Kiwi Tauri plugin (`plugin:kiwi|<command>` from the UI).
pub fn kiwi_plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("kiwi")
        .invoke_handler(tauri::generate_handler![
            kiwi_host_info,
            logs_snapshot,
            logs_clear,
            problems_snapshot,
            problems_run,
            docs_list,
            docs_read,
            themes_list,
            theme_set_active,
            swift_status,
            swift_tasks_overview,
            swift_list_projects,
            swift_link_workspace,
            swift_unlink_workspace,
            swift_get_task,
            mcp_overview,
            agent_launch,
            agent_input,
            agent_resize,
            agent_stop,
            agent_status,
            agent_settings_get,
            agent_settings_save,
            ollama_list_models,
            ollama_auth_status,
            ollama_signin,
            ollama_signout,
            codex_account_status,
            codex_login,
            codex_logout,
            terminal_open,
            terminal_input,
            terminal_resize,
            terminal_close,
            terminal_list,
            workspace_info,
            workspace_list,
            workspace_read,
            workspace_write,
            workspace_create_file,
            workspace_create_dir,
            workspace_rename,
            workspace_delete,
            workspace_copy,
            workspace_reveal,
            workspace_open,
            workspace_search,
            workspace_replace_all,
            git_status,
            git_stage,
            git_stage_all,
            git_unstage,
            git_discard,
            git_commit,
            git_push,
            git_pull,
            git_log,
            git_commit_changes,
            github_auth_status,
            github_repo,
            github_issue_list,
            github_issue_view,
            github_issue_create,
            github_issue_comment,
            github_label_list,
            github_milestone_list,
        ])
        .build()
}
