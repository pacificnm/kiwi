//! Cursor-inspired IDE workbench layout for Kiwi.

mod activity;
mod bottom_panel;
mod editor;
mod editor_diff;
mod editor_files;
mod editor_syntax;
mod explorer;
mod issues;
mod menu;
mod prompt;
mod sidebar;
mod source_control;
mod state;
mod watcher;
mod workspace;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};

use egui::{Align, CentralPanel, ComboBox, Frame, Layout, RichText, ScrollArea, SidePanel, TextEdit, TopBottomPanel, Ui, UiBuilder};
use nest_agent::AgentEvent;
use nest_ai::{AiService, ChatMessage, ChatRole};
use nest_config::ConfigService;
use nest_core::AppContext;
use nest_error::NestResult;
use nest_gui::{ActionButton, ButtonSize, StatusBarService, WorkbenchView};
use nest_icon::{font, Icon};

pub use editor_files::{FileLoadEvent, FileSaveEvent};
pub use state::{ChatEntry, ToolActivityStatus, WorkbenchState};

use crate::agent::AgentLoopConfig;
use crate::agent::AgentSettings;
use crate::agent::try_persist_preferences;
use crate::chat::{self, AgentRunEvent, ChatStreamEvent};
use crate::project::{ProjectConfig, RecentProjects};
use crate::theme::PALETTE;
use crate::workbench::activity::Activity;
use crate::workbench::bottom_panel::BottomTab;
use crate::workbench::editor::{active_issue_number, editor_panel, EditorPanelRequest, EditorTabDragPayload, EditorTabView};
use crate::workbench::editor_files::{
    apply_file_load, apply_file_save, apply_issue_created, apply_pr_load, begin_file_save, issue_tab_key,
    open_new_issue_tab, parse_issue_tab_repo, pr_tab_key, spawn_save_file,
};
use crate::workbench::issues::{
    comment_modal, load_token_from_config, read_github_repo, show_issue_metadata_modal,
    show_labels_modal, show_milestones_modal, show_pr_merge_modal, CommentModalAction,
    IssueCreateEvent, IssueMetadataModalAction, LabelsModalAction, MilestonesModalAction,
    PrMergeModalAction,
};
use crate::workbench::source_control::{show_branch_create_modal, BranchCreateModalAction};
use crate::workbench::watcher::ProjectWatcher;
use nest_ai_ollama::{OllamaConfig, OllamaSharedConfig};

/// Background file read channel for editor tabs.
pub type FileLoadPending = Receiver<FileLoadEvent>;

/// Background file write channel for editor tabs.
pub type FileSavePending = Receiver<FileSaveEvent>;

/// Background GitHub issue creation channel for editor tabs.
type IssueCreatePending = Receiver<IssueCreateEvent>;

/// Native folder picker result channel.
type FolderDialogPending = Receiver<FolderDialogEvent>;

use activity::{activity_bar, ACTIVITY_BAR_WIDTH};
use sidebar::{section_heading, sidebar};

use menu::MenuState;
use workspace::{spawn_folder_dialog, switch_workspace, FolderDialogEvent};

const TITLE_BAR_HEIGHT: f32 = 36.0;
const SIDEBAR_WIDTH: f32 = 260.0;
const AI_PANEL_WIDTH: f32 = 360.0;
/// Chat header row (title + agent toggle).
const AI_CHAT_HEADER_HEIGHT: f32 = 36.0;
/// Prompt input, model selector, buttons, and token stats row.
const AI_PROMPT_SECTION_HEIGHT: f32 = 192.0;
/// Matches [`ButtonSize::Small`] action buttons in the chat toolbar.
const CHAT_CONTROL_HEIGHT: f32 = 28.0;
use prompt::PROMPT_INPUT_HEIGHT;

const CHAT_MODEL_WIDTH: f32 = 148.0;

/// Kiwi IDE workbench shell (layout MVP).
pub struct KiwiWorkbench {
    state: WorkbenchState,
    fonts_installed: bool,
    theme_applied: bool,
    ai_config_loaded: bool,
    project_loaded: bool,
    github_auth_loaded: bool,
    chat_pending: Option<Receiver<ChatStreamEvent>>,
    agent_pending: Option<Receiver<AgentRunEvent>>,
    file_pending: Option<FileLoadPending>,
    file_save_pending: Option<FileSavePending>,
    issue_create_pending: Option<IssueCreatePending>,
    project_watcher: Option<ProjectWatcher>,
    source_control_loaded: bool,
    last_activity: Activity,
    menu: MenuState,
    recent: RecentProjects,
    recent_loaded: bool,
    folder_dialog: Option<FolderDialogPending>,
    workspace_error: Option<String>,
}

impl Default for KiwiWorkbench {
    fn default() -> Self {
        Self {
            state: WorkbenchState::demo(),
            fonts_installed: false,
            theme_applied: false,
            ai_config_loaded: false,
            project_loaded: false,
            github_auth_loaded: false,
            chat_pending: None,
            agent_pending: None,
            file_pending: None,
            file_save_pending: None,
            issue_create_pending: None,
            project_watcher: None,
            source_control_loaded: false,
            last_activity: Activity::Explorer,
            menu: MenuState::default(),
            recent: RecentProjects::load(None),
            recent_loaded: false,
            folder_dialog: None,
            workspace_error: None,
        }
    }
}

impl WorkbenchView for KiwiWorkbench {
    fn ui(&mut self, ctx: &egui::Context, app_ctx: &AppContext) -> NestResult<()> {
        font::ensure_installed(ctx);
        if !self.fonts_installed {
            crate::fonts::install(ctx);
            self.fonts_installed = true;
        }
        if !self.theme_applied {
            crate::theme::apply(ctx);
            self.theme_applied = true;
        }
        self.sync_ai_config(app_ctx);
        self.sync_project_config(app_ctx);
        self.sync_github_auth(app_ctx);
        self.poll_chat(ctx);
        self.poll_agent(ctx);
        self.poll_file_load(ctx);
        self.poll_file_save(ctx);
        self.poll_source_control(ctx);
        self.poll_issues(ctx, app_ctx);
        self.poll_issue_create(ctx);
        self.poll_menu_actions(ctx);
        self.poll_workspace(ctx, app_ctx);
        self.poll_project_watcher(ctx);
        self.schedule_background_poll(ctx);
        self.sync_status(app_ctx);

        TopBottomPanel::top("kiwi-title-bar")
            .exact_height(TITLE_BAR_HEIGHT)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(PALETTE.background_elevated)
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ctx, |ui| {
                title_bar(
                    ui,
                    &self.state,
                    &self.recent,
                    &mut self.menu,
                    self.workspace_error.as_deref(),
                );
            });

        SidePanel::left("kiwi-activity-bar")
            .exact_width(ACTIVITY_BAR_WIDTH)
            .resizable(false)
            .show_separator_line(true)
            .frame(
                Frame::new()
                    .fill(PALETTE.background_sidebar)
                    .inner_margin(egui::Margin::symmetric(0, 2)),
            )
            .show(ctx, |ui| {
                activity_bar(ui, &mut self.state.activity);
            });

        SidePanel::right("kiwi-ai-panel")
            .default_width(AI_PANEL_WIDTH)
            .resizable(true)
            .frame(
                Frame::new()
                    .fill(PALETTE.background_editor)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                ai_panel(
                    ui,
                    &mut self.state,
                    app_ctx,
                    &mut self.chat_pending,
                    &mut self.agent_pending,
                );
            });

        SidePanel::left("kiwi-sidebar-v3")
            .default_width(SIDEBAR_WIDTH)
            .width_range(200.0..=400.0)
            .resizable(true)
            .show_separator_line(true)
            .frame(
                Frame::new()
                    .fill(PALETTE.background_sidebar)
                    .inner_margin(egui::Margin::symmetric(8, 2)),
            )
            .show(ctx, |ui| {
                sidebar(
                    ui,
                    &mut self.state,
                    app_ctx,
                    &mut self.file_pending,
                );
            });

        bottom_panel::show_panel(ctx, &mut self.state);

        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(PALETTE.background_editor)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                central_panel(
                    ui,
                    ctx,
                    &mut self.state,
                    app_ctx,
                    &mut self.file_save_pending,
                    &mut self.issue_create_pending,
                );
            });

        self.poll_chat(ctx);
        self.poll_agent(ctx);
        self.poll_file_load(ctx);
        self.poll_file_save(ctx);
        self.poll_source_control(ctx);
        self.poll_issues(ctx, app_ctx);
        self.poll_issue_create(ctx);
        self.poll_menu_actions(ctx);
        self.poll_workspace(ctx, app_ctx);
        self.poll_project_watcher(ctx);
        if self.state.bottom_tab == BottomTab::Terminal
            && (self.state.terminal.poll() || self.state.terminal.is_active())
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        }
        self.schedule_background_poll(ctx);

        if self.state.activity == Activity::SourceControl
            && self.last_activity != Activity::SourceControl
        {
            self.state
                .source_control
                .request_refresh(&self.state.project.root);
            self.state
                .source_control
                .request_branch_list(&self.state.project.root);
        }
        if self.state.activity == Activity::Issues
            && self.last_activity != Activity::Issues
        {
            self.state.issues.sync_gh_auth();
            if let Ok(http) = app_ctx.service::<nest_http_client::HttpClientService>() {
                self.state
                    .issues
                    .request_list(&self.state.project.root, http.clone());
                self.state
                    .issues
                    .request_pr_list(&self.state.project.root, http.clone());
            }
        }
        self.last_activity = self.state.activity;

        self.show_comment_modal(ctx, app_ctx);
        self.show_metadata_modals(ctx, app_ctx);
        self.show_branch_create_modal(ctx);
        self.show_pr_merge_modal(ctx, app_ctx);

        Ok(())
    }
}

impl KiwiWorkbench {
    fn sync_ai_config(&mut self, app_ctx: &AppContext) {
        if self.ai_config_loaded {
            return;
        }
        let Ok(config) = app_ctx.service::<ConfigService>() else {
            return;
        };
        if let Ok(agent) = AgentSettings::from_config_service(&config) {
            self.state.agent = agent;
        } else if let Ok(Some(ollama)) = OllamaConfig::from_config_service(&config) {
            let (host, port) = OllamaConfig::host_port_from_base_url(&ollama.base_url);
            self.state.agent.host = host;
            self.state.agent.port = port.to_string();
            self.state.agent.model = ollama.model;
        }
        if let Ok(loop_cfg) = AgentLoopConfig::from_config_service(&config) {
            self.state.agent_mcp_path = loop_cfg.mcp_config_path.display().to_string();
            self.state.agent_mcp_servers = loop_cfg.mcp_servers;
            self.state.agent.disabled_mcp_servers = loop_cfg.disabled_mcp_servers;
            self.state.agent.allow_save_context = loop_cfg.allow_save_context;
            self.state.agent_mode = loop_cfg.agent_mode;
            if !loop_cfg.model.is_empty() {
                self.state.agent.mcp_status = Some(format!(
                    "Agent model: {} | MCP: {}",
                    loop_cfg.model,
                    self.state.agent_mcp_path
                ));
            }
        }
        self.state.sync_model_from_agent();
        if let Ok(shared) = app_ctx.service::<OllamaSharedConfig>() {
            self.state.agent.apply_runtime(&shared);
        }
        self.ai_config_loaded = true;
    }

    fn sync_github_auth(&mut self, app_ctx: &AppContext) {
        if self.github_auth_loaded {
            return;
        }
        if let Ok(config) = app_ctx.service::<ConfigService>() {
            if let Some(token) = load_token_from_config(&config) {
                self.state.issues.set_stored_token(Some(token));
            }
        }
        self.state.issues.sync_gh_auth();

        if self.state.issues.auth_login.is_none() && self.state.issues.token().is_some() {
            if let Ok(http) = app_ctx.service::<nest_http_client::HttpClientService>() {
                self.state.issues.request_verify(http.clone());
            }
        }
        self.github_auth_loaded = true;
    }

    fn sync_project_config(&mut self, app_ctx: &AppContext) {
        if self.project_loaded {
            return;
        }
        let Ok(config) = app_ctx.service::<ConfigService>() else {
            return;
        };
        if let Ok(project) = ProjectConfig::from_config_service(&config) {
            if !self.recent_loaded {
                self.recent = RecentProjects::load(config.path());
                self.recent_loaded = true;
            }
            let _ = self.recent.record(&project.root);

            let files = nest_file::FileService::with_config(
                nest_file::FileServiceConfig::scoped(project.root.clone()),
            )
            .unwrap_or_else(|error| {
                tracing::warn!(target: "kiwi", %error, "scoped file service failed at startup");
                self.state.files.clone()
            });

            self.state.project = project.clone();
            self.state.files = files;
            self.state.explorer = crate::workbench::explorer::ExplorerState::new(
                &project.root,
                &project.name,
                project.ignore.clone(),
            );
            self.project_watcher =
                ProjectWatcher::new(&project.root, project.ignore.clone()).ok();
            self.project_loaded = true;
            self.state.terminal.set_cwd(project.root.clone());
            self.state.source_control =
                crate::workbench::source_control::SourceControlState::new(project.root.clone());
            self.state
                .source_control
                .request_refresh(&project.root);
            self.state
                .source_control
                .request_branch_list(&project.root);
            self.source_control_loaded = true;
            tracing::info!(
                target: "kiwi",
                root = %project.root.display(),
                name = %project.name,
                "Project loaded"
            );
        }
    }

    fn poll_file_save(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.file_save_pending.as_ref() else {
            return;
        };
        match rx.try_recv() {
            Ok(event) => {
                apply_file_save(&mut self.state.editor, event);
                self.file_save_pending = None;
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.file_save_pending = None;
                ctx.request_repaint();
            }
        }
    }

    fn poll_project_watcher(&mut self, ctx: &egui::Context) {
        let Some(watcher) = self.project_watcher.as_mut() else {
            return;
        };
        if !watcher.poll() {
            return;
        }
        if let Err(error) = self.state.explorer.refresh(&self.state.files) {
            self.state.explorer.error = Some(error.to_string());
        }
        self.state
            .source_control
            .request_refresh(&self.state.project.root);
        ctx.request_repaint();
    }

    /// Schedules a single follow-up frame while background I/O is in flight.
    fn schedule_background_poll(&self, ctx: &egui::Context) {
        const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);

        let needs_poll = self.chat_pending.is_some()
            || self.agent_pending.is_some()
            || self.file_pending.is_some()
            || self.file_save_pending.is_some()
            || self.state.source_control.has_pending_io()
            || self.state.issues.busy()
            || self.issue_create_pending.is_some()
            || self.folder_dialog.is_some()
            || (self.state.bottom_tab == BottomTab::Terminal
                && self.state.terminal.is_active());

        if needs_poll {
            ctx.request_repaint_after(POLL_INTERVAL);
        }
    }

    fn poll_workspace(&mut self, ctx: &egui::Context, app_ctx: &AppContext) {
        if !self.recent_loaded {
            if let Ok(config) = app_ctx.service::<ConfigService>() {
                self.recent = RecentProjects::load(config.path());
            }
            self.recent_loaded = true;
        }

        if self.menu.open_folder_requested && self.folder_dialog.is_none() {
            self.menu.open_folder_requested = false;
            self.folder_dialog = Some(spawn_folder_dialog());
        }

        if let Some(rx) = self.folder_dialog.as_ref() {
            match rx.try_recv() {
                Ok(FolderDialogEvent::Selected(path)) => {
                    self.folder_dialog = None;
                    self.open_workspace_root(app_ctx, path);
                    ctx.request_repaint();
                }
                Ok(FolderDialogEvent::Cancelled) => {
                    self.folder_dialog = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.folder_dialog = None;
                    self.workspace_error = Some("Folder picker interrupted".into());
                    ctx.request_repaint();
                }
            }
        }

        if let Some(path) = self.menu.open_recent_path.take() {
            self.open_workspace_root(app_ctx, path);
            ctx.request_repaint();
        }
    }

    fn open_workspace_root(&mut self, app_ctx: &AppContext, root: PathBuf) {
        self.file_pending = None;
        self.file_save_pending = None;
        self.issue_create_pending = None;
        match switch_workspace(
            &mut self.state,
            &mut self.project_watcher,
            &mut self.recent,
            app_ctx,
            root,
        ) {
            Ok(()) => {
                self.workspace_error = None;
                self.source_control_loaded = true;
            }
            Err(error) => {
                self.workspace_error = Some(error.to_string());
            }
        }
    }

    fn poll_source_control(&mut self, ctx: &egui::Context) {
        let root = self.state.project.root.clone();
        if self.state.source_control.poll(&root) {
            ctx.request_repaint();
        }
        if self.state.source_control.take_focus_git_panel() {
            self.state.bottom_tab = BottomTab::Git;
            ctx.request_repaint();
        }
    }

    fn poll_issues(&mut self, ctx: &egui::Context, app_ctx: &AppContext) {
        if self.state.issues.poll() {
            ctx.request_repaint();
        }
        if let Some(number) = self.state.issues.comment_posted_on.take() {
            self.reload_open_issue_tab(number, app_ctx);
            ctx.request_repaint();
        }
        if let Some(number) = self.state.issues.issue_updated_on.take() {
            self.reload_open_issue_tab(number, app_ctx);
            ctx.request_repaint();
        }
        if let Some(issue) = self.state.issues.issue_sent_to_agent.take() {
            self.state
                .prompt
                .attach_issue(issue.number, &issue.title, &issue.content);
            self.state.agent_mode = true;
            ctx.request_repaint();
        }
        if let Some((tab_index, detail)) = self.state.issues.pr_loaded.take() {
            apply_pr_load(&mut self.state.editor, tab_index, &detail);
            ctx.request_repaint();
        }
        if let Some(number) = self.state.issues.pr_merged_on.take() {
            self.reload_open_pr_tab(number, app_ctx);
            self.state
                .source_control
                .request_refresh(&self.state.project.root);
            self.state
                .source_control
                .request_branch_list(&self.state.project.root);
            if let Ok(http) = app_ctx.service::<nest_http_client::HttpClientService>() {
                self.state
                    .issues
                    .request_pr_list(&self.state.project.root, http.clone());
            }
            ctx.request_repaint();
        }
    }

    fn poll_issue_create(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.issue_create_pending.as_ref() else {
            return;
        };
        match rx.try_recv() {
            Ok(event) => {
                apply_issue_created(&mut self.state.editor, event);
                self.issue_create_pending = None;
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.issue_create_pending = None;
                ctx.request_repaint();
            }
        }
    }

    fn poll_menu_actions(&mut self, ctx: &egui::Context) {
        if self.menu.new_comment_requested {
            self.menu.new_comment_requested = false;
            let issue_number = active_issue_number(&self.state.editor);
            self.state.issues.open_new_comment(issue_number);
            ctx.request_repaint();
        }

        if self.menu.manage_labels_requested {
            self.menu.manage_labels_requested = false;
            self.state.issues.open_manage_labels();
            ctx.request_repaint();
        }

        if self.menu.manage_milestones_requested {
            self.menu.manage_milestones_requested = false;
            self.state.issues.open_manage_milestones();
            ctx.request_repaint();
        }

        if self.menu.new_issue_requested {
            self.menu.new_issue_requested = false;

            let repo = self
                .state
                .issues
                .repo
                .clone()
                .or_else(|| read_github_repo(&self.state.project.root).ok());

            let Some((owner, repo)) = repo else {
                self.state.issues.error =
                    Some("Could not resolve GitHub repository from origin".into());
                ctx.request_repaint();
                return;
            };

            open_new_issue_tab(&mut self.state.editor, &owner, &repo);
            ctx.request_repaint();
        }
    }

    fn show_comment_modal(&mut self, ctx: &egui::Context, app_ctx: &AppContext) {
        let issues_list = self.state.issues.issues.clone();
        let Some(action) = comment_modal::show(
            ctx,
            &mut self.state.issues.comment_modal,
            &issues_list,
        ) else {
            return;
        };

        let CommentModalAction::Submit {
            issue_number,
            body,
        } = action;

        let repo = self
            .state
            .issues
            .repo
            .clone()
            .or_else(|| read_github_repo(&self.state.project.root).ok());

        let Some((owner, repo)) = repo else {
            self.state.issues.comment_modal.error =
                Some("Could not resolve GitHub repository from origin".into());
            ctx.request_repaint();
            return;
        };

        let Ok(http) = app_ctx.service::<nest_http_client::HttpClientService>() else {
            self.state.issues.comment_modal.error =
                Some("HTTP client is not configured".into());
            ctx.request_repaint();
            return;
        };

        self.state.issues.request_create_comment(
            &owner,
            &repo,
            issue_number,
            body,
            http.clone(),
        );
        ctx.request_repaint();
    }

    fn show_metadata_modals(&mut self, ctx: &egui::Context, app_ctx: &AppContext) {
        if let Some(action) = show_labels_modal(ctx, &mut self.state.issues.labels_modal) {
            self.handle_labels_modal_action(action, app_ctx, ctx);
        }
        if let Some(action) = show_milestones_modal(ctx, &mut self.state.issues.milestones_modal) {
            self.handle_milestones_modal_action(action, app_ctx, ctx);
        }
        if let Some(action) =
            show_issue_metadata_modal(ctx, &mut self.state.issues.issue_metadata_modal)
        {
            self.handle_issue_metadata_modal_action(action, app_ctx, ctx);
        }
    }

    fn github_repo(&self) -> Option<(String, String)> {
        self.state
            .issues
            .repo
            .clone()
            .or_else(|| read_github_repo(&self.state.project.root).ok())
    }

    fn github_http(&self, app_ctx: &AppContext) -> Option<nest_http_client::HttpClientService> {
        app_ctx
            .service::<nest_http_client::HttpClientService>()
            .ok()
            .cloned()
    }

    fn handle_labels_modal_action(
        &mut self,
        action: LabelsModalAction,
        app_ctx: &AppContext,
        ctx: &egui::Context,
    ) {
        let Some((owner, repo)) = self.github_repo() else {
            self.state.issues.labels_modal.fail(
                "Could not resolve GitHub repository from origin".into(),
            );
            ctx.request_repaint();
            return;
        };
        let Some(http) = self.github_http(app_ctx) else {
            self.state
                .issues
                .labels_modal
                .fail("HTTP client is not configured".into());
            ctx.request_repaint();
            return;
        };

        match action {
            LabelsModalAction::RequestList => {
                self.state
                    .issues
                    .request_list_labels(&owner, &repo, http);
            }
            LabelsModalAction::Create {
                name,
                color,
                description,
            } => {
                self.state.issues.request_create_label(
                    &owner, &repo, name, color, description, http,
                );
            }
            LabelsModalAction::Update {
                original_name,
                name,
                color,
                description,
            } => {
                self.state.issues.request_update_label(
                    &owner,
                    &repo,
                    original_name,
                    name,
                    color,
                    description,
                    http,
                );
            }
            LabelsModalAction::Delete { name } => {
                self.state
                    .issues
                    .request_delete_label(&owner, &repo, name, http);
            }
        }
        ctx.request_repaint();
    }

    fn handle_milestones_modal_action(
        &mut self,
        action: MilestonesModalAction,
        app_ctx: &AppContext,
        ctx: &egui::Context,
    ) {
        let Some((owner, repo)) = self.github_repo() else {
            self.state.issues.milestones_modal.fail(
                "Could not resolve GitHub repository from origin".into(),
            );
            ctx.request_repaint();
            return;
        };
        let Some(http) = self.github_http(app_ctx) else {
            self.state.issues.milestones_modal.fail(
                "HTTP client is not configured".into(),
            );
            ctx.request_repaint();
            return;
        };

        match action {
            MilestonesModalAction::RequestList => {
                self.state
                    .issues
                    .request_list_milestones(&owner, &repo, http);
            }
            MilestonesModalAction::Create {
                title,
                description,
                due_on,
            } => {
                self.state.issues.request_create_milestone(
                    &owner, &repo, title, description, due_on, http,
                );
            }
            MilestonesModalAction::Update {
                number,
                title,
                description,
                due_on,
                state,
            } => {
                self.state.issues.request_update_milestone(
                    &owner, &repo, number, title, description, due_on, state, http,
                );
            }
            MilestonesModalAction::Delete { number } => {
                self.state
                    .issues
                    .request_delete_milestone(&owner, &repo, number, http);
            }
        }
        ctx.request_repaint();
    }

    fn handle_issue_metadata_modal_action(
        &mut self,
        action: IssueMetadataModalAction,
        app_ctx: &AppContext,
        ctx: &egui::Context,
    ) {
        let Some((owner, repo)) = self.github_repo() else {
            self.state.issues.issue_metadata_modal.fail(
                "Could not resolve GitHub repository from origin".into(),
            );
            ctx.request_repaint();
            return;
        };
        let Some(http) = self.github_http(app_ctx) else {
            self.state.issues.issue_metadata_modal.fail(
                "HTTP client is not configured".into(),
            );
            ctx.request_repaint();
            return;
        };

        match action {
            IssueMetadataModalAction::RequestLoad => {
                self.state.issues.request_load_issue_metadata(
                    &owner,
                    &repo,
                    self.state.issues.issue_metadata_modal.issue_number,
                    http,
                );
            }
            IssueMetadataModalAction::Save {
                issue_number,
                labels,
                milestone,
            } => {
                self.state.issues.request_update_issue_metadata(
                    &owner, &repo, issue_number, labels, milestone, http,
                );
            }
        }
        ctx.request_repaint();
    }

    fn show_branch_create_modal(&mut self, ctx: &egui::Context) {
        let branches = self.state.source_control.branches.clone();
        let Some(action) = show_branch_create_modal(
            ctx,
            &mut self.state.source_control.branch_create_modal,
            &branches,
        ) else {
            return;
        };

        let BranchCreateModalAction::Create { name, start_branch } = action;
        self.state.source_control.create_branch(
            &self.state.project.root,
            name,
            start_branch,
        );
        ctx.request_repaint();
    }

    fn reload_open_issue_tab(&mut self, number: u64, app_ctx: &AppContext) {
        let repo = self
            .state
            .issues
            .repo
            .clone()
            .or_else(|| read_github_repo(&self.state.project.root).ok());
        let Some((owner, repo)) = repo else {
            return;
        };

        let rel_path = issue_tab_key(&owner, &repo, number);
        let view = EditorTabView::Issue { number };
        let Some(tab_index) = self.state.editor.tabs.iter().position(|tab| {
            tab.rel_path == rel_path && tab.view == view
        }) else {
            return;
        };

        self.state.editor.active_tab = tab_index;
        {
            let tab = &mut self.state.editor.tabs[tab_index];
            tab.loading = true;
            tab.error = None;
        }

        if self.file_pending.is_some() {
            return;
        }

        let Ok(http) = app_ctx.service::<nest_http_client::HttpClientService>() else {
            return;
        };

        self.file_pending = Some(self.state.issues.spawn_open_issue(
            &owner,
            &repo,
            number,
            tab_index,
            http.clone(),
        ));
    }

    fn reload_open_pr_tab(&mut self, number: u64, app_ctx: &AppContext) {
        let repo = self
            .state
            .issues
            .repo
            .clone()
            .or_else(|| read_github_repo(&self.state.project.root).ok());
        let Some((owner, repo)) = repo else {
            return;
        };

        let rel_path = pr_tab_key(&owner, &repo, number);
        let view = EditorTabView::PullRequest { number };
        let Some(tab_index) = self.state.editor.tabs.iter().position(|tab| {
            tab.rel_path == rel_path && tab.view == view
        }) else {
            return;
        };

        self.state.editor.active_tab = tab_index;
        {
            let tab = &mut self.state.editor.tabs[tab_index];
            tab.loading = true;
            tab.error = None;
        }

        let Ok(http) = app_ctx.service::<nest_http_client::HttpClientService>() else {
            return;
        };

        self.state.issues.request_open_pr(
            &self.state.project.root,
            number,
            tab_index,
            http.clone(),
        );
    }

    fn show_pr_merge_modal(&mut self, ctx: &egui::Context, app_ctx: &AppContext) {
        let Some(action) = show_pr_merge_modal(ctx, &mut self.state.issues.pr_merge_modal) else {
            return;
        };

        let PrMergeModalAction::Merge {
            number,
            merge_method,
        } = action;

        let Ok(http) = app_ctx.service::<nest_http_client::HttpClientService>() else {
            self.state
                .issues
                .pr_merge_modal
                .fail("HTTP client is not configured".into());
            ctx.request_repaint();
            return;
        };

        self.state.issues.request_merge_pr(
            &self.state.project.root,
            number,
            merge_method,
            http.clone(),
        );
        ctx.request_repaint();
    }

    fn poll_file_load(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.file_pending.as_ref() else {
            return;
        };
        match rx.try_recv() {
            Ok(event) => {
                apply_file_load(&mut self.state.editor, event);
                self.file_pending = None;
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.file_pending = None;
                ctx.request_repaint();
            }
        }
    }

    fn poll_chat(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.chat_pending.as_ref() else {
            return;
        };
        loop {
            match rx.try_recv() {
                Ok(ChatStreamEvent::Delta(text)) => {
                    if let Some(last) = self.state.chat_messages.last_mut() {
                        if last.role == ChatRole::Assistant {
                            last.content.push_str(&text);
                        }
                    }
                    ctx.request_repaint();
                }
                Ok(ChatStreamEvent::Finished { result: Ok(()), metrics }) => {
                    let empty_response = self.state.chat_messages.last().is_some_and(|last| {
                        last.role == ChatRole::Assistant && last.content.is_empty()
                    });
                    if empty_response {
                        self.state.chat_messages.pop();
                        self.state.chat_error =
                            Some("Model returned an empty response".into());
                        self.state.chat_metrics = None;
                    } else {
                        self.state.chat_error = None;
                        self.state.chat_metrics = metrics;
                    }
                    self.state.chat_busy = false;
                    self.chat_pending = None;
                    ctx.request_repaint();
                    break;
                }
                Ok(ChatStreamEvent::Finished { result: Err(error), .. }) => {
                    if let Some(last) = self.state.chat_messages.last() {
                        if last.role == ChatRole::Assistant && last.content.is_empty() {
                            self.state.chat_messages.pop();
                        }
                    }
                    self.state.chat_error = Some(chat::format_ai_error_message(&error.to_string()));
                    self.state.chat_metrics = None;
                    self.state.chat_busy = false;
                    self.chat_pending = None;
                    ctx.request_repaint();
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if self.state.chat_messages.last().is_some_and(|last| {
                        last.role == ChatRole::Assistant && last.content.is_empty()
                    }) {
                        self.state.chat_messages.pop();
                    }
                    self.state.chat_error = Some("Chat stream interrupted".into());
                    self.state.chat_metrics = None;
                    self.state.chat_busy = false;
                    self.chat_pending = None;
                    ctx.request_repaint();
                    break;
                }
            }
        }
    }

    fn poll_agent(&mut self, ctx: &egui::Context) {
        let mut events = Vec::new();
        let mut finished = None;
        let mut disconnected = false;

        if let Some(rx) = self.agent_pending.as_ref() {
            loop {
                match rx.try_recv() {
                    Ok(AgentRunEvent::Event(event)) => {
                        let failed = matches!(event, AgentEvent::Failed(_));
                        events.push(event);
                        if failed {
                            break;
                        }
                    }
                    Ok(AgentRunEvent::Finished(result)) => {
                        finished = Some(result);
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }

        for event in &events {
            self.handle_agent_event(event);
            ctx.request_repaint();
        }

        if events.iter().any(|event| matches!(event, AgentEvent::Failed(_))) {
            self.state.chat_busy = false;
            self.agent_pending = None;
            self.state.agent_step = None;
            ctx.request_repaint();
            return;
        }

        if let Some(result) = finished {
            if let Err(error) = result {
                if self.state.chat_messages.last().is_some_and(|last| {
                    last.role == ChatRole::Assistant && last.content.is_empty()
                }) {
                    self.state.chat_messages.pop();
                }
                self.state.chat_error =
                    Some(chat::format_ai_error_message(&error.to_string()));
            } else {
                self.state.chat_error = None;
            }
            self.state.chat_busy = false;
            self.agent_pending = None;
            self.state.agent_step = None;
            ctx.request_repaint();
        } else if disconnected {
            self.state.chat_error = Some("Agent run interrupted".into());
            self.state.chat_busy = false;
            self.agent_pending = None;
            self.state.agent_step = None;
            ctx.request_repaint();
        }
    }

    fn handle_agent_event(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::TextDelta(text) => {
                ensure_assistant_message(&mut self.state.chat_messages)
                    .content
                    .push_str(text);
            }
            AgentEvent::StepStarted { step } => {
                self.state.agent_step = Some(*step);
                if *step > 1 {
                    ensure_assistant_message(&mut self.state.chat_messages);
                }
            }
            AgentEvent::ToolCallStarted { tool, arguments } => {
                self.state.chat_messages.push(ChatEntry {
                    role: ChatRole::Tool,
                    content: bottom_panel::tool_activity::format_tool_call_summary(
                        &tool, &arguments,
                    ),
                });
                self.state.tool_activity.push(
                    bottom_panel::tool_activity::new_running_entry(
                        tool.clone(),
                        &arguments,
                        self.state.agent_step,
                    ),
                );
                self.state.bottom_tab = bottom_panel::BottomTab::ToolActivity;
            }
            AgentEvent::ToolCallFinished {
                tool,
                result,
                duration,
                ..
            } => {
                if let Some(entry) = self
                    .state
                    .tool_activity
                    .iter_mut()
                    .rev()
                    .find(|entry| entry.tool == *tool && entry.status == ToolActivityStatus::Running)
                {
                    entry.result = Some(result.clone());
                    entry.status = ToolActivityStatus::Success;
                    entry.duration_ms =
                        Some(bottom_panel::tool_activity::duration_to_ms(*duration));
                }
                if let Some(last) = self.state.chat_messages.last_mut() {
                    if last.role == ChatRole::Tool {
                        let preview =
                            bottom_panel::tool_activity::format_tool_result_chat_preview(result);
                        last.content.push_str(&format!("\n↳ {preview}"));
                    }
                }
                self.state.agent.mcp_tool_count = Some(self.state.tool_activity.len());
            }
            AgentEvent::ToolCallFailed { tool, error } => {
                if let Some(entry) = self
                    .state
                    .tool_activity
                    .iter_mut()
                    .rev()
                    .find(|entry| entry.tool == *tool && entry.status == ToolActivityStatus::Running)
                {
                    entry.error = Some(error.clone());
                    entry.status = ToolActivityStatus::Failed;
                }
            }
            AgentEvent::Finished { content, metrics, .. } => {
                let assistant = ensure_assistant_message(&mut self.state.chat_messages);
                if assistant.content.is_empty() {
                    assistant.content = content.clone();
                }
                self.state.chat_metrics = metrics.clone();
                self.state.agent_step = None;
            }
            AgentEvent::Failed(error) => {
                self.state.chat_error = Some(chat::format_ai_error_message(error));
                self.state.agent_step = None;
            }
        }
    }

    fn sync_status(&mut self, app_ctx: &AppContext) {
        self.state.sync_model_from_agent();
        let Ok(status) = app_ctx.service::<StatusBarService>() else {
            return;
        };

        let busy_label = if self.state.chat_busy {
            if self.state.agent_mode {
                "Agent…"
            } else {
                "Generating…"
            }
        } else {
            "Ready"
        };

        let base = format!(
            "main | Rust | {} | {} | Ln 42 Col 18 | UTF-8 | Spaces:4",
            busy_label,
            self.state.model
        );

        if self.state.chat_busy {
            status.loading(base);
            status.clear_right();
        } else {
            status.set(base);
            if let Some(metrics) = &self.state.chat_metrics {
                status.set_right_text(metrics.status_label());
            } else {
                status.clear_right();
            }
        }
    }
}

fn title_bar(
    ui: &mut Ui,
    state: &WorkbenchState,
    recent: &RecentProjects,
    menu: &mut MenuState,
    workspace_error: Option<&str>,
) {
    let weak = ui.visuals().weak_text_color();
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.painter()
        .hline(ui.max_rect().x_range(), ui.max_rect().bottom(), stroke);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        ui.add_space(8.0);
        ui.label(RichText::new("Kiwi").strong().size(14.0));
        ui.separator();
        menu::file_menu(ui, recent, menu);
        menu::git_menu(ui, menu);
        ui.separator();
        ui.label(
            RichText::new(&state.project.name)
                .size(12.0)
                .color(weak),
        )
        .on_hover_text(state.project.root.display().to_string());
        ui.label(
            RichText::new(&state.model)
                .size(12.0)
                .color(weak),
        );

        if let Some(error) = workspace_error {
            ui.label(
                RichText::new(error)
                    .size(11.0)
                    .color(ui.visuals().error_fg_color),
            );
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.add_space(12.0);
            ui.label(RichText::new("Settings").strong().size(14.0));
        });
    });
}

fn central_panel(
    ui: &mut Ui,
    ctx: &egui::Context,
    state: &mut WorkbenchState,
    app_ctx: &AppContext,
    file_save_pending: &mut Option<FileSavePending>,
    issue_create_pending: &mut Option<IssueCreatePending>,
) {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.set_min_height(ui.available_height());

    if let Some(request) = editor_panel(ui, ctx, &mut state.editor) {
        match request {
            EditorPanelRequest::SaveFile(tab_index) => {
                if file_save_pending.is_none() {
                    if let Some((rel_path, content)) = begin_file_save(&mut state.editor, tab_index)
                    {
                        *file_save_pending = Some(spawn_save_file(
                            state.files.clone(),
                            rel_path,
                            content,
                            tab_index,
                        ));
                    } else if let Some(tab) = state.editor.tabs.get_mut(tab_index) {
                        tab.saving = false;
                        tab.save_error = Some("File save failed".into());
                    }
                }
            }
            EditorPanelRequest::CreateIssue(tab_index) => {
                if issue_create_pending.is_some() {
                    return;
                }
                let Some(tab) = state.editor.tabs.get(tab_index) else {
                    return;
                };
                let Some((owner, repo)) = parse_issue_tab_repo(&tab.rel_path) else {
                    return;
                };
                let title = tab.issue_title.trim().to_string();
                if title.is_empty() {
                    return;
                };
                let body = tab.content.clone();
                let Ok(http) = app_ctx.service::<nest_http_client::HttpClientService>() else {
                    if let Some(tab) = state.editor.tabs.get_mut(tab_index) {
                        tab.save_error = Some("HTTP client is not configured".into());
                    }
                    return;
                };
                if let Some(tab) = state.editor.tabs.get_mut(tab_index) {
                    tab.saving = true;
                    tab.save_error = None;
                }
                *issue_create_pending = Some(state.issues.spawn_create_issue(
                    &owner,
                    &repo,
                    title,
                    body,
                    tab_index,
                    http.clone(),
                ));
            }
            EditorPanelRequest::EditIssueMetadata(tab_index) => {
                let Some(tab) = state.editor.tabs.get(tab_index) else {
                    return;
                };
                let EditorTabView::Issue { number } = tab.view else {
                    return;
                };
                state.issues.open_issue_metadata(number);
            }
            EditorPanelRequest::MergePullRequest(tab_index) => {
                let Some(tab) = state.editor.tabs.get(tab_index) else {
                    return;
                };
                let EditorTabView::PullRequest { number } = tab.view else {
                    return;
                };
                state.issues.open_merge_pr(number);
            }
        }
    }
}

fn ai_panel(
    ui: &mut Ui,
    state: &mut WorkbenchState,
    app_ctx: &AppContext,
    chat_pending: &mut Option<Receiver<ChatStreamEvent>>,
    agent_pending: &mut Option<Receiver<AgentRunEvent>>,
) {
    let (_, dropped) = ui.dnd_drop_zone::<EditorTabDragPayload, _>(
        Frame::new()
            .fill(PALETTE.background_editor)
            .inner_margin(egui::Margin::ZERO),
        |ui| {
            ai_panel_contents(ui, state, app_ctx, chat_pending, agent_pending);
        },
    );

    if let Some(payload) = dropped {
        let file = payload.as_ref();
        state.prompt.attach_file(&file.rel_path, &file.content);
        state.agent_mode = true;
    }
}

fn ai_panel_contents(
    ui: &mut Ui,
    state: &mut WorkbenchState,
    app_ctx: &AppContext,
    chat_pending: &mut Option<Receiver<ChatStreamEvent>>,
    agent_pending: &mut Option<Receiver<AgentRunEvent>>,
) {
    ui.spacing_mut().item_spacing.y = 0.0;

    let panel = ui.available_rect_before_wrap();
    let pending = chat_pending.is_some() || agent_pending.is_some();

    let header_rect = egui::Rect::from_min_max(
        panel.min,
        egui::pos2(panel.max.x, panel.min.y + AI_CHAT_HEADER_HEIGHT),
    );
    let prompt_rect = egui::Rect::from_min_max(
        egui::pos2(panel.min.x, panel.max.y - AI_PROMPT_SECTION_HEIGHT),
        panel.max,
    );
    let conversation_rect = egui::Rect::from_min_max(
        egui::pos2(panel.min.x, header_rect.max.y),
        egui::pos2(panel.max.x, prompt_rect.min.y),
    );

    ui.scope_builder(UiBuilder::new().max_rect(header_rect), |ui| {
        Frame::new()
            .fill(PALETTE.background_editor)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    section_heading(ui, "Chat");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let response = ui.checkbox(&mut state.agent_mode, "Agent");
                        if response.changed() {
                            let _ = try_persist_preferences(
                                &state.agent,
                                state.agent_mode,
                                app_ctx,
                            );
                        }
                    });
                });
            });
    });

    ui.scope_builder(UiBuilder::new().max_rect(conversation_rect), |ui| {
        Frame::new()
            .fill(PALETTE.background_editor)
            .inner_margin(egui::Margin::symmetric(8, 8))
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .id_salt("kiwi-ai-conversation")
                    .max_height(ui.available_height())
                    .auto_shrink([false; 2])
                    .stick_to_bottom(state.chat_busy || pending)
                    .show(ui, |ui| {
                        ai_conversation(ui, state, pending);
                    });
            });
    });

    ui.scope_builder(UiBuilder::new().max_rect(prompt_rect), |ui| {
        ai_prompt_section(ui, state, app_ctx, chat_pending, agent_pending);
    });
}

fn ai_conversation(ui: &mut Ui, state: &mut WorkbenchState, pending: bool) {
    if state.chat_messages.is_empty() {
        ui.label(
            RichText::new(
                "Ask questions about your project, generate plans, \
                 or apply code changes. Drag an editor tab here to \
                 attach file contents for the agent.",
            )
            .weak()
            .size(13.0),
        );
        return;
    }

    for (index, entry) in state.chat_messages.iter().enumerate() {
        let is_streaming = state.chat_busy
            && pending
            && entry.role == ChatRole::Assistant
            && state
                .chat_messages
                .iter()
                .rposition(|message| message.role == ChatRole::Assistant)
                == Some(index);

        match entry.role {
            ChatRole::User => user_message_bubble(ui, &entry.content),
            ChatRole::Tool => tool_call_block(ui, &entry.content),
            ChatRole::Assistant | ChatRole::System => {
                let (label, color) = match entry.role {
                    ChatRole::Assistant => ("Kiwi", ui.visuals().text_color()),
                    ChatRole::System => ("System", ui.visuals().weak_text_color()),
                    ChatRole::User | ChatRole::Tool => unreachable!(),
                };
                ui.label(RichText::new(label).strong().size(11.0).color(color));

                if is_streaming && entry.content.is_empty() {
                    ui.label(
                        RichText::new("Generating…")
                            .weak()
                            .italics()
                            .size(13.0),
                    );
                } else {
                    let mut display = entry.content.clone();
                    if is_streaming {
                        display.push('▍');
                    }
                    ui.label(RichText::new(display).size(13.0));
                }
            }
        }
        ui.add_space(10.0);
    }

    if let Some(error) = &state.chat_error {
        ui.add_space(4.0);
        ui.label(
            RichText::new(error)
                .color(ui.visuals().error_fg_color)
                .size(12.0),
        );
    }
}

fn ai_prompt_section(
    ui: &mut Ui,
    state: &mut WorkbenchState,
    app_ctx: &AppContext,
    chat_pending: &mut Option<Receiver<ChatStreamEvent>>,
    agent_pending: &mut Option<Receiver<AgentRunEvent>>,
) {
    Frame::new()
        .fill(PALETTE.background_editor)
        .inner_margin(egui::Margin::symmetric(8, 8))
        .show(ui, |ui| {
            Frame::new()
                .fill(PALETTE.background_panel)
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(8, 8))
                .show(ui, |ui| {
                    state.prompt.consume_large_pastes(ui);
                    ui.add_sized(
                        egui::vec2(ui.available_width(), PROMPT_INPUT_HEIGHT),
                        TextEdit::multiline(&mut state.prompt.visible)
                            .hint_text("Ask Kiwi…")
                            .desired_width(f32::INFINITY)
                            .desired_rows(3)
                            .frame(false),
                    );
                    state.prompt.collapse_oversized_visible();
                });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                ui.set_min_height(CHAT_CONTROL_HEIGHT);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    chat_model_selector(ui, state, app_ctx);
                    if ui
                        .add(
                            ActionButton::new(Icon::PAPERCLIP, "Attach")
                                .size(ButtonSize::Small)
                                .tooltip("Attach files"),
                        )
                        .clicked()
                    {}
                    if ui
                        .add(
                            ActionButton::new(Icon::PAPER_PLANE, "Send")
                                .size(ButtonSize::Small)
                                .fill(PALETTE.accent_primary)
                                .text_color(egui::Color32::WHITE)
                                .tooltip(if state.agent_mode {
                                    "Run agent (MCP tools)"
                                } else {
                                    "Send message"
                                }),
                        )
                        .clicked()
                    {
                        if state.agent_mode {
                            send_agent_message(state, app_ctx, agent_pending);
                        } else {
                            send_chat_message(state, app_ctx, chat_pending);
                        }
                    }
                });
            });
            ui.add_space(6.0);
            ui.set_min_height(14.0);
            if let Some(metrics) = &state.chat_metrics {
                ui.label(
                    RichText::new(metrics.detail_label())
                        .weak()
                        .size(11.0),
                );
            }
        });
}

fn tool_call_block(ui: &mut Ui, content: &str) {
    Frame::new()
        .fill(PALETTE.background_panel)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!("🔧 {content}"))
                    .monospace()
                    .size(12.0)
                    .color(PALETTE.text_secondary),
            );
        });
}

fn user_message_bubble(ui: &mut Ui, content: &str) {
    let row_width = ui.available_width();
    let bubble_width = (row_width * 0.88).max(120.0).min(row_width);

    ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
        ui.set_width(row_width);
        Frame::new()
            .fill(PALETTE.background_panel)
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_max_width(bubble_width);
                ui.label(RichText::new(content).size(13.0));
            });
    });
}

fn chat_model_selector(ui: &mut Ui, state: &mut WorkbenchState, app_ctx: &AppContext) {
    let models = if state.agent.models.is_empty() {
        vec![state.agent.model.clone()]
    } else {
        state.agent.models.clone()
    };

    let mut changed = false;

    ui.allocate_ui_with_layout(
        egui::vec2(CHAT_MODEL_WIDTH, CHAT_CONTROL_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().interact_size.y = CHAT_CONTROL_HEIGHT;
            ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);
            ComboBox::from_id_salt("kiwi-chat-model")
                .selected_text(RichText::new(&state.agent.model).size(13.0))
                .width(CHAT_MODEL_WIDTH)
                .truncate()
                .show_ui(ui, |ui| {
                    for model in models {
                        if ui
                            .selectable_value(
                                &mut state.agent.model,
                                model.clone(),
                                model.as_str(),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    }
                });
        },
    );

    if changed {
        state.sync_model_from_agent();
        if let Ok(shared) = app_ctx.service::<nest_ai_ollama::OllamaSharedConfig>() {
            state.agent.apply_runtime(&shared);
        }
        let _ = try_persist_preferences(&state.agent, state.agent_mode, app_ctx);
    }
}

fn send_chat_message(
    state: &mut WorkbenchState,
    app_ctx: &AppContext,
    chat_pending: &mut Option<Receiver<ChatStreamEvent>>,
) {
    if state.chat_busy || chat_pending.is_some() {
        return;
    }

    if state.prompt.is_empty() {
        return;
    }

    let prompt = state.prompt.resolve();

    let Ok(ai) = app_ctx.service::<AiService>() else {
        state.chat_error = Some("AI service is not configured".into());
        return;
    };

    state.chat_messages.push(ChatEntry {
        role: ChatRole::User,
        content: prompt.clone(),
    });
    state.prompt.clear();
    state.chat_error = None;
    state.chat_metrics = None;
    state.chat_busy = true;

    let history: Vec<ChatMessage> = state
        .chat_messages
        .iter()
        .map(|entry| match entry.role {
            ChatRole::User => ChatMessage::user(&entry.content),
            ChatRole::Assistant => ChatMessage::assistant(&entry.content),
            ChatRole::System => ChatMessage::system(&entry.content),
            ChatRole::Tool => ChatMessage::tool_result("tool", &entry.content),
        })
        .collect();

    state.chat_messages.push(ChatEntry {
        role: ChatRole::Assistant,
        content: String::new(),
    });

    let model = state.agent.model.clone();
    *chat_pending = Some(chat::spawn_stream_complete_messages(
        ai.clone(),
        history,
        Some(model),
    ));
}

fn send_agent_message(
    state: &mut WorkbenchState,
    app_ctx: &AppContext,
    agent_pending: &mut Option<Receiver<AgentRunEvent>>,
) {
    if state.chat_busy || agent_pending.is_some() {
        return;
    }

    if state.prompt.is_empty() {
        return;
    }

    let Ok(ai) = app_ctx.service::<AiService>() else {
        state.chat_error = Some("AI service is not configured".into());
        return;
    };

    let Ok(config) = app_ctx.service::<ConfigService>() else {
        state.chat_error = Some("Config service is not available".into());
        return;
    };

    let agent_cfg = match AgentLoopConfig::from_config_service(&config) {
        Ok(cfg) => cfg,
        Err(error) => {
            state.chat_error = Some(error.to_string());
            return;
        }
    };

    let prompt = state.prompt.resolve();
    state.chat_messages.push(ChatEntry {
        role: ChatRole::User,
        content: prompt.clone(),
    });
    state.prompt.clear();
    state.chat_error = None;
    state.chat_metrics = None;
    state.chat_busy = true;
    state.tool_activity.clear();

    let history: Vec<ChatMessage> = state
        .chat_messages
        .iter()
        .filter(|entry| entry.role != ChatRole::Tool)
        .map(|entry| match entry.role {
            ChatRole::User => ChatMessage::user(&entry.content),
            ChatRole::Assistant => ChatMessage::assistant(&entry.content),
            ChatRole::System => ChatMessage::system(&entry.content),
            ChatRole::Tool => ChatMessage::tool_result("tool", &entry.content),
        })
        .collect();

    let mcp_path = agent_cfg.mcp_config_path.clone();
    let mcp_servers = agent_cfg.enabled_mcp_servers();
    let agent_config = agent_cfg.agent_config();
    let model = agent_cfg.model;

    *agent_pending = Some(chat::spawn_agent_run(
        ai.clone(),
        history,
        Some(model),
        mcp_path,
        mcp_servers,
        agent_config,
    ));
}

/// Ensures the chat transcript ends with an assistant entry for agent streaming.
fn ensure_assistant_message(messages: &mut Vec<ChatEntry>) -> &mut ChatEntry {
    if messages
        .last()
        .is_some_and(|entry| entry.role == ChatRole::Assistant)
    {
        let index = messages.len() - 1;
        return &mut messages[index];
    }
    messages.push(ChatEntry {
        role: ChatRole::Assistant,
        content: String::new(),
    });
    let index = messages.len() - 1;
    &mut messages[index]
}
