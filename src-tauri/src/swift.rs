//! Swift PostgreSQL integration for Kiwi Tasks (projects + tasks + workspace links).

use std::path::{Path, PathBuf};

use nest_data_postgres::{PostgresConfig, PostgresConnection};
use nest_error::{NestError, NestResult};
use serde::{Deserialize, Serialize};
use swift_data::{
    baseline_existing_schema, swift_migrations, AsyncRepository, KiwiWorkspaceLink,
    KiwiWorkspaceLinkRepository, Project, ProjectRepository, Task, TaskRepository,
};
use tokio::sync::OnceCell;
use uuid::Uuid;

/// `[swift]` section in Kiwi `config.toml`.
#[derive(Debug, Deserialize)]
struct SwiftSection {
    /// Direct PostgreSQL URL (overrides `config_path`).
    database_url: Option<String>,
    /// Path to Swift `config.toml` containing `[database].url`.
    config_path: Option<String>,
}

/// `[database]` section inside Swift's config.
#[derive(Debug, Deserialize)]
struct SwiftDatabaseSection {
    url: String,
}

/// Swift DB connection status for the Tasks panel.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwiftStatus {
    pub connected: bool,
    pub database: String,
    pub error: Option<String>,
}

/// Swift project row for the link picker.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwiftProjectSummary {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub archived: bool,
    pub percent_complete: i16,
}

/// Swift task row for the Tasks sidebar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwiftTaskSummary {
    pub id: String,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub outline_level: i16,
    pub is_summary: bool,
    pub is_milestone: bool,
    pub title: String,
    pub percent_complete: i16,
    pub start_date: Option<String>,
    pub finish_date: Option<String>,
    pub priority: Option<String>,
    pub sort_order: i32,
}

/// Full Swift task payload for the editor detail view.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwiftTaskDetail {
    pub id: String,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub outline_level: i16,
    pub is_summary: bool,
    pub is_milestone: bool,
    pub title: String,
    pub notes: Option<String>,
    pub duration_days: i32,
    pub duration_minutes: Option<i32>,
    pub start_date: Option<String>,
    pub finish_date: Option<String>,
    pub percent_complete: i16,
    pub resource_names: String,
    pub priority: Option<String>,
    pub constraint_type: Option<String>,
    pub constraint_date: Option<String>,
    pub deadline: Option<String>,
    pub effort_driven: bool,
    pub task_type: Option<String>,
    pub sort_order: i32,
    pub actual_start: Option<String>,
    pub actual_finish: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwiftTaskDetailResponse {
    pub task: SwiftTaskDetail,
    pub project: SwiftProjectSummary,
    pub subtasks: Vec<SwiftTaskSummary>,
}

/// Workspace link + loaded tasks for the active repo.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwiftTasksOverview {
    pub status: SwiftStatus,
    pub workspace_root: String,
    pub link: Option<SwiftWorkspaceLinkSummary>,
    pub project: Option<SwiftProjectSummary>,
    pub tasks: Vec<SwiftTaskSummary>,
    pub projects: Vec<SwiftProjectSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwiftWorkspaceLinkSummary {
    pub workspace_root: String,
    pub project_id: String,
    pub project_name: Option<String>,
}

/// Lazy Swift DB handle (connects on first Tasks IPC call).
pub struct SwiftDb {
    kiwi_config_path: Option<PathBuf>,
    connection: OnceCell<PostgresConnection>,
}

impl SwiftDb {
    pub fn new(kiwi_config_path: Option<PathBuf>) -> Self {
        Self {
            kiwi_config_path,
            connection: OnceCell::new(),
        }
    }

    pub async fn status(&self) -> SwiftStatus {
        let database = resolve_database_url(self.kiwi_config_path.as_deref())
            .map(|url| redact_database_url(&url))
            .unwrap_or_else(|error| format!("swift ({error})"));
        match self.connection().await {
            Ok(_) => SwiftStatus {
                connected: true,
                database,
                error: None,
            },
            Err(error) => SwiftStatus {
                connected: false,
                database,
                error: Some(error.to_string()),
            },
        }
    }

    pub async fn overview(&self, workspace_root: &Path) -> NestResult<SwiftTasksOverview> {
        let normalized = normalize_workspace_root(workspace_root)?;
        let status = self.status().await;
        if !status.connected {
            return Ok(SwiftTasksOverview {
                status,
                workspace_root: normalized,
                link: None,
                project: None,
                tasks: Vec::new(),
                projects: Vec::new(),
            });
        }

        let conn = self.connection().await?;
        let projects = list_projects(&conn).await?;
        let link_repo = KiwiWorkspaceLinkRepository::new(conn.clone());
        let link_row = link_repo.by_workspace_root(&normalized).await.map_err(map_data)?;

        let link = link_row.as_ref().map(link_summary);
        let project = link_row
            .as_ref()
            .and_then(|row| projects.iter().find(|item| item.id == row.project_id.to_string()))
            .cloned();

        let tasks = if let Some(row) = &link_row {
            list_tasks(&conn, row.project_id).await?
        } else {
            Vec::new()
        };

        Ok(SwiftTasksOverview {
            status,
            workspace_root: normalized,
            link,
            project,
            tasks,
            projects,
        })
    }

    pub async fn list_projects(&self) -> NestResult<Vec<SwiftProjectSummary>> {
        let conn = self.connection().await?;
        list_projects(&conn).await
    }

    pub async fn get_task(&self, task_id: Uuid) -> NestResult<SwiftTaskDetailResponse> {
        let conn = self.connection().await?;
        let task_repo = TaskRepository::new(conn.clone());
        let project_repo = ProjectRepository::new(conn.clone());
        let task = task_repo
            .get(task_id)
            .await
            .map_err(map_data)?
            .ok_or_else(|| NestError::validation(format!("swift task not found: {task_id}")))?;
        let project = project_repo
            .get(task.project_id)
            .await
            .map_err(map_data)?
            .ok_or_else(|| {
                NestError::validation(format!("swift project not found: {}", task.project_id))
            })?;
        let all_tasks = task_repo
            .list_by_project(task.project_id)
            .await
            .map_err(map_data)?;
        let subtasks = all_tasks
            .iter()
            .filter(|row| row.parent_id == Some(task_id))
            .map(|row| task_summary(row.clone()))
            .collect();
        Ok(SwiftTaskDetailResponse {
            task: task_detail(&task),
            project: project_summary(project),
            subtasks,
        })
    }

    pub async fn link_workspace(
        &self,
        workspace_root: &Path,
        project_id: Uuid,
    ) -> NestResult<SwiftWorkspaceLinkSummary> {
        let normalized = normalize_workspace_root(workspace_root)?;
        let conn = self.connection().await?;
        let project_repo = ProjectRepository::new(conn.clone());
        let project = project_repo
            .get(project_id)
            .await
            .map_err(map_data)?
            .ok_or_else(|| NestError::validation(format!("swift project not found: {project_id}")))?;

        let link = KiwiWorkspaceLinkRepository::new(conn)
            .upsert(&normalized, project_id, Some(&project.name))
            .await
            .map_err(map_data)?;
        Ok(link_summary(&link))
    }

    pub async fn unlink_workspace(&self, workspace_root: &Path) -> NestResult<()> {
        let normalized = normalize_workspace_root(workspace_root)?;
        let conn = self.connection().await?;
        KiwiWorkspaceLinkRepository::new(conn)
            .delete_by_workspace_root(&normalized)
            .await
            .map_err(map_data)
    }

    async fn connection(&self) -> NestResult<PostgresConnection> {
        self.connection
            .get_or_try_init(|| async {
                let url = resolve_database_url(self.kiwi_config_path.as_deref())?;
                let conn = PostgresConnection::connect(&PostgresConfig::new(url))
                    .await
                    .map_err(|error| NestError::config(format!("swift database connect failed: {error}")))?;
                baseline_existing_schema(conn.pool())
                    .await
                    .map_err(|error| NestError::config(format!("swift schema baseline failed: {error}")))?;
                let migrations = swift_migrations();
                nest_data_postgres::migration::apply_migrations(conn.pool(), &migrations)
                    .await
                    .map_err(|error| NestError::config(format!("swift migrations failed: {error}")))?;
                Ok(conn)
            })
            .await
            .cloned()
    }
}

fn resolve_database_url(kiwi_config_path: Option<&Path>) -> NestResult<String> {
    if let Some(path) = kiwi_config_path {
        let text = std::fs::read_to_string(path)
            .map_err(|error| NestError::config(format!("read kiwi config failed: {error}")))?;
        let root: toml::Value = toml::from_str(&text)
            .map_err(|error| NestError::config(format!("parse kiwi config failed: {error}")))?;
        if let Some(section) = root.get("swift") {
            let swift: SwiftSection = section
                .clone()
                .try_into()
                .map_err(|error| NestError::config(format!("invalid [swift] section: {error}")))?;
            if let Some(url) = swift
                .database_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                return Ok(url.to_string());
            }
            if let Some(config_path) = swift.config_path.as_deref() {
                let swift_config = resolve_relative_path(path, config_path);
                let swift_text = std::fs::read_to_string(&swift_config).map_err(|error| {
                    NestError::config(format!(
                        "read swift config `{}` failed: {error}",
                        swift_config.display()
                    ))
                })?;
                let swift_root: toml::Value = toml::from_str(&swift_text).map_err(|error| {
                    NestError::config(format!(
                        "parse swift config `{}` failed: {error}",
                        swift_config.display()
                    ))
                })?;
                let database: SwiftDatabaseSection = swift_root
                    .get("database")
                    .ok_or_else(|| NestError::config("swift config missing [database] section"))?
                    .clone()
                    .try_into()
                    .map_err(|error| NestError::config(format!("invalid swift [database]: {error}")))?;
                return Ok(database.url);
            }
        }
    }
    Err(NestError::config(
        "Swift database not configured — add [swift] config_path or database_url to Kiwi config.toml",
    ))
}

fn resolve_relative_path(base_config: &Path, relative: &str) -> PathBuf {
    base_config
        .parent()
        .map(|dir| dir.join(relative))
        .unwrap_or_else(|| PathBuf::from(relative))
}

pub fn normalize_workspace_root(workspace_root: &Path) -> NestResult<String> {
    let canonical = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    Ok(canonical.to_string_lossy().replace('\\', "/"))
}

async fn list_projects(conn: &PostgresConnection) -> NestResult<Vec<SwiftProjectSummary>> {
    let repo = ProjectRepository::new(conn.clone());
    let rows = repo.list_library().await.map_err(map_data)?;
    Ok(rows.into_iter().map(project_summary).collect())
}

async fn list_tasks(conn: &PostgresConnection, project_id: Uuid) -> NestResult<Vec<SwiftTaskSummary>> {
    let repo = TaskRepository::new(conn.clone());
    let rows = repo.list_by_project(project_id).await.map_err(map_data)?;
    Ok(rows.into_iter().map(task_summary).collect())
}

fn project_summary(project: Project) -> SwiftProjectSummary {
    SwiftProjectSummary {
        id: project.id.to_string(),
        slug: project.slug,
        name: project.name,
        archived: project.archived,
        percent_complete: project.percent_complete,
    }
}

fn task_summary(task: Task) -> SwiftTaskSummary {
    SwiftTaskSummary {
        id: task.id.to_string(),
        project_id: task.project_id.to_string(),
        parent_id: task.parent_id.map(|id| id.to_string()),
        outline_level: task.outline_level,
        is_summary: task.is_summary,
        is_milestone: task.is_milestone,
        title: task.title,
        percent_complete: task.percent_complete,
        start_date: task.start_date.map(|date| date.to_string()),
        finish_date: task.finish_date.map(|date| date.to_string()),
        priority: task.priority,
        sort_order: task.sort_order,
    }
}

fn task_detail(task: &Task) -> SwiftTaskDetail {
    SwiftTaskDetail {
        id: task.id.to_string(),
        project_id: task.project_id.to_string(),
        parent_id: task.parent_id.map(|id| id.to_string()),
        outline_level: task.outline_level,
        is_summary: task.is_summary,
        is_milestone: task.is_milestone,
        title: task.title.clone(),
        notes: task.notes.clone(),
        duration_days: task.duration_days,
        duration_minutes: task.duration_minutes,
        start_date: task.start_date.map(|date| date.to_string()),
        finish_date: task.finish_date.map(|date| date.to_string()),
        percent_complete: task.percent_complete,
        resource_names: task.resource_names.clone(),
        priority: task.priority.clone(),
        constraint_type: task.constraint_type.clone(),
        constraint_date: task.constraint_date.map(|date| date.to_string()),
        deadline: task.deadline.map(|date| date.to_string()),
        effort_driven: task.effort_driven,
        task_type: task.task_type.clone(),
        sort_order: task.sort_order,
        actual_start: task.actual_start.map(|date| date.to_string()),
        actual_finish: task.actual_finish.map(|date| date.to_string()),
        created_at: task.created_at.to_rfc3339(),
        updated_at: task.updated_at.to_rfc3339(),
    }
}

fn link_summary(link: &KiwiWorkspaceLink) -> SwiftWorkspaceLinkSummary {
    SwiftWorkspaceLinkSummary {
        workspace_root: link.workspace_root.clone(),
        project_id: link.project_id.to_string(),
        project_name: link.project_name.clone(),
    }
}

fn map_data(error: swift_data::DataError) -> NestError {
    NestError::data(error.to_string())
}

fn redact_database_url(url: &str) -> String {
    if let Some(at) = url.find('@') {
        if let Some(scheme_end) = url.find("://") {
            let scheme = &url[..scheme_end + 3];
            let host = &url[at + 1..];
            return format!("{scheme}***@{host}");
        }
    }
    url.to_string()
}
