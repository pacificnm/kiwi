//! Cursor-inspired IDE workbench layout for Kiwi.

mod activity;
mod bottom_panel;
mod editor;
mod editor_diff;
mod editor_files;
mod editor_syntax;
mod explorer;
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
use crate::workbench::editor_files::{apply_file_load, apply_file_save, begin_file_save, spawn_save_file};
use crate::workbench::watcher::ProjectWatcher;
use nest_ai_ollama::{OllamaConfig, OllamaSharedConfig};

/// Background file read channel for editor tabs.
pub type FileLoadPending = Receiver<FileLoadEvent>;

/// Background file write channel for editor tabs.
pub type FileSavePending = Receiver<FileSaveEvent>;

/// Native folder picker result channel.
type FolderDialogPending = Receiver<FolderDialogEvent>;

use activity::{activity_bar, ACTIVITY_BAR_WIDTH};
use editor::{editor_panel, EditorTabDragPayload};
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
    chat_pending: Option<Receiver<ChatStreamEvent>>,
    agent_pending: Option<Receiver<AgentRunEvent>>,
    file_pending: Option<FileLoadPending>,
    file_save_pending: Option<FileSavePending>,
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
            chat_pending: None,
            agent_pending: None,
            file_pending: None,
            file_save_pending: None,
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
        self.poll_chat(ctx);
        self.poll_agent(ctx);
        self.poll_file_load(ctx);
        self.poll_file_save(ctx);
        self.poll_source_control(ctx);
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
                central_panel(ui, ctx, &mut self.state, &mut self.file_save_pending);
            });

        self.poll_chat(ctx);
        self.poll_agent(ctx);
        self.poll_file_load(ctx);
        self.poll_file_save(ctx);
        self.poll_source_control(ctx);
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
        }
        self.last_activity = self.state.activity;

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
        menu::file_menu(ui, recent, menu);
        ui.separator();
        ui.label(RichText::new("Kiwi").strong().size(14.0));
        ui.label(
            RichText::new(format!("Project: {}", state.project.name))
                .size(12.0)
                .color(weak),
        )
        .on_hover_text(state.project.root.display().to_string());
        ui.label(
            RichText::new(format!("Model: {}", state.model))
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
    file_save_pending: &mut Option<FileSavePending>,
) {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.set_min_height(ui.available_height());

    if let Some(tab_index) = editor_panel(ui, ctx, &mut state.editor) {
        if file_save_pending.is_none() {
            if let Some((rel_path, content)) = begin_file_save(&mut state.editor, tab_index) {
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
