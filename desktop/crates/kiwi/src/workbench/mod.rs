//! Cursor-inspired IDE workbench layout for Kiwi.

mod activity;
mod bottom_panel;
mod editor;
mod editor_files;
mod editor_syntax;
mod explorer;
mod prompt;
mod sidebar;
mod state;
mod watcher;

use std::sync::mpsc::{Receiver, TryRecvError};

use egui::{Align, CentralPanel, ComboBox, Frame, Layout, RichText, ScrollArea, Separator, SidePanel, TextEdit, TopBottomPanel, Ui, UiBuilder};
use nest_agent::AgentEvent;
use nest_ai::{AiService, ChatMessage, ChatRole};
use nest_config::ConfigService;
use nest_core::AppContext;
use nest_error::NestResult;
use nest_gui::{ActionButton, ButtonSize, StatusBarService, WorkbenchView};
use nest_icon::{font, Icon};

pub use editor_files::{FileLoadEvent, FileSaveEvent};
pub use state::{ChatEntry, WorkbenchState};

use crate::agent::AgentLoopConfig;
use crate::agent::AgentSettings;
use crate::chat::{self, AgentRunEvent, ChatStreamEvent};
use crate::project::ProjectConfig;
use crate::theme::PALETTE;
use crate::workbench::editor_files::{apply_file_load, apply_file_save, begin_file_save, spawn_save_file};
use crate::workbench::watcher::ProjectWatcher;
use nest_ai_ollama::{OllamaConfig, OllamaSharedConfig};

/// Background file read channel for editor tabs.
pub type FileLoadPending = Receiver<FileLoadEvent>;

/// Background file write channel for editor tabs.
pub type FileSavePending = Receiver<FileSaveEvent>;

use activity::{activity_bar, ACTIVITY_BAR_WIDTH};
use bottom_panel::bottom_panel;
use editor::{editor_panel, EditorTabDragPayload};
use sidebar::{section_heading, sidebar};

use prompt::PROMPT_INPUT_HEIGHT;

const TITLE_BAR_HEIGHT: f32 = 36.0;
const SIDEBAR_WIDTH: f32 = 260.0;
const AI_PANEL_WIDTH: f32 = 360.0;
const BOTTOM_PANEL_HEIGHT: f32 = 200.0;
/// Chat header row (title + agent toggle).
const AI_CHAT_HEADER_HEIGHT: f32 = 36.0;
/// Prompt input, model selector, buttons, and token stats row.
const AI_PROMPT_SECTION_HEIGHT: f32 = 192.0;
/// Matches [`ButtonSize::Small`] action buttons in the chat toolbar.
const CHAT_CONTROL_HEIGHT: f32 = 28.0;
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
        if self.chat_pending.is_some()
            || self.agent_pending.is_some()
            || self.file_pending.is_some()
            || self.file_save_pending.is_some()
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        }
        self.sync_status(app_ctx);

        let sidebar_frame = Frame::new()
            .fill(PALETTE.background_sidebar)
            .inner_margin(egui::Margin::symmetric(8, 2));

        TopBottomPanel::top("kiwi-title-bar")
            .exact_height(TITLE_BAR_HEIGHT)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(PALETTE.background_elevated)
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ctx, |ui| {
                title_bar(ui, &self.state);
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

        SidePanel::left("kiwi-sidebar-v2")
            .default_width(SIDEBAR_WIDTH)
            .width_range(200.0..=320.0)
            .resizable(true)
            .frame(sidebar_frame)
            .show(ctx, |ui| {
                sidebar(
                    ui,
                    &mut self.state,
                    app_ctx,
                    &mut self.file_pending,
                );
            });

        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(PALETTE.background_editor)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                central_panel(ui, ctx, &mut self.state, app_ctx, &mut self.file_save_pending);
            });

        self.poll_chat(ctx);
        self.poll_agent(ctx);
        self.poll_file_load(ctx);
        self.poll_file_save(ctx);
        self.poll_project_watcher(ctx, app_ctx);
        if self.chat_pending.is_some()
            || self.agent_pending.is_some()
            || self.file_pending.is_some()
            || self.file_save_pending.is_some()
            || self.project_watcher.as_ref().is_some_and(|w| w.has_pending())
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        }

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
            self.state.project = project.clone();
            self.state.explorer = crate::workbench::explorer::ExplorerState::new(
                &project.root,
                &project.name,
                project.ignore.clone(),
            );
            self.project_watcher = ProjectWatcher::new(&project.root).ok();
            self.project_loaded = true;
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

    fn poll_project_watcher(&mut self, ctx: &egui::Context, app_ctx: &AppContext) {
        let Some(watcher) = self.project_watcher.as_mut() else {
            return;
        };
        if !watcher.poll() {
            return;
        }
        let Ok(files) = app_ctx.service::<nest_file::FileService>() else {
            return;
        };
        if let Err(error) = self.state.explorer.refresh(&files) {
            self.state.explorer.error = Some(error.to_string());
        }
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
                    self.state.chat_error = Some(error.to_string());
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
                self.state.chat_error = Some(error.to_string());
            } else {
                self.state.chat_error = None;
            }
            self.state.chat_busy = false;
            self.agent_pending = None;
            ctx.request_repaint();
        } else if disconnected {
            self.state.chat_error = Some("Agent run interrupted".into());
            self.state.chat_busy = false;
            self.agent_pending = None;
            ctx.request_repaint();
        }
    }

    fn handle_agent_event(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::TextDelta(text) => {
                if let Some(last) = self.state.chat_messages.last_mut() {
                    if last.role == ChatRole::Assistant {
                        last.content.push_str(text);
                    }
                }
            }
            AgentEvent::ToolCallStarted { tool, arguments } => {
                self.state.chat_messages.push(ChatEntry {
                    role: ChatRole::Tool,
                    content: format!("{tool}({arguments})"),
                });
                self.state.tool_activity.push(crate::workbench::state::ToolActivityEntry {
                    tool: tool.clone(),
                    detail: arguments.to_string(),
                    running: true,
                });
                self.state.bottom_tab = bottom_panel::BottomTab::ToolActivity;
            }
            AgentEvent::ToolCallFinished { tool, result, .. } => {
                if let Some(entry) = self
                    .state
                    .tool_activity
                    .iter_mut()
                    .rev()
                    .find(|entry| entry.tool == *tool && entry.running)
                {
                    entry.detail = result.clone();
                    entry.running = false;
                }
                if let Some(last) = self.state.chat_messages.last_mut() {
                    if last.role == ChatRole::Tool {
                        last.content.push_str(&format!("\n↳ {result}"));
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
                    .find(|entry| entry.tool == *tool && entry.running)
                {
                    entry.detail = error.clone();
                    entry.running = false;
                }
                self.state.chat_error = Some(format!("Tool {tool} failed: {error}"));
            }
            AgentEvent::Finished { content, metrics, .. } => {
                if let Some(last) = self.state.chat_messages.last_mut() {
                    if last.role == ChatRole::Assistant && last.content.is_empty() {
                        last.content = content.clone();
                    }
                }
                self.state.chat_metrics = metrics.clone();
            }
            AgentEvent::Failed(error) => {
                self.state.chat_error = Some(error.clone());
            }
            AgentEvent::StepStarted { .. } => {}
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

fn title_bar(ui: &mut Ui, state: &WorkbenchState) {
    let weak = ui.visuals().weak_text_color();
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.painter()
        .hline(ui.max_rect().x_range(), ui.max_rect().bottom(), stroke);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 16.0;
        ui.add_space(12.0);
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
) {
    ui.spacing_mut().item_spacing.y = 0.0;

    let height = ui.available_height();
    let bottom_height = BOTTOM_PANEL_HEIGHT.min(height * 0.45).max(80.0);
    let editor_height = (height - bottom_height).max(100.0);

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), editor_height),
        Layout::top_down(Align::LEFT),
        |ui| {
            ui.set_height(editor_height);
            if let Some(tab_index) = editor_panel(ui, ctx, &mut state.editor) {
                if file_save_pending.is_none() {
                    if let Some((rel_path, content)) = begin_file_save(&mut state.editor, tab_index)
                    {
                        if let Ok(files) = app_ctx.service::<nest_file::FileService>() {
                            *file_save_pending = Some(spawn_save_file(
                                files.clone(),
                                rel_path,
                                content,
                                tab_index,
                            ));
                        } else if let Some(tab) = state.editor.tabs.get_mut(tab_index) {
                            tab.saving = false;
                            tab.save_error = Some("File service unavailable".into());
                        }
                    }
                }
            }
        },
    );

    ui.add(Separator::default().spacing(1.0));

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), bottom_height),
        Layout::top_down(Align::LEFT),
        |ui| {
            ui.set_height(bottom_height);
            bottom_panel(ui, state);
        },
    );
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
                        ui.checkbox(&mut state.agent_mode, "Agent");
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
                        } else {
                            for (index, entry) in state.chat_messages.iter().enumerate() {
                                let is_streaming = state.chat_busy
                                    && pending
                                    && index + 1 == state.chat_messages.len()
                                    && entry.role == ChatRole::Assistant;

                                match entry.role {
                                    ChatRole::User => {
                                        user_message_bubble(ui, &entry.content);
                                    }
                                    ChatRole::Tool => {
                                        tool_call_block(ui, &entry.content);
                                    }
                                    ChatRole::Assistant | ChatRole::System => {
                                        let (label, color) = match entry.role {
                                            ChatRole::Assistant => {
                                                ("Kiwi", ui.visuals().text_color())
                                            }
                                            ChatRole::System => {
                                                ("System", ui.visuals().weak_text_color())
                                            }
                                            ChatRole::User | ChatRole::Tool => unreachable!(),
                                        };
                                        ui.label(
                                            RichText::new(label).strong().size(11.0).color(color),
                                        );

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
                        }

                        if let Some(error) = &state.chat_error {
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(error)
                                    .color(ui.visuals().error_fg_color)
                                    .size(12.0),
                            );
                        }
                    });
            });
    });

    ui.scope_builder(UiBuilder::new().max_rect(prompt_rect), |ui| {
        ai_prompt_section(ui, state, app_ctx, chat_pending, agent_pending);
    });
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
            ui.label(RichText::new("Prompt Input").weak().size(11.0));
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
                    chat_model_selector(ui, state);
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

fn chat_model_selector(ui: &mut Ui, state: &mut WorkbenchState) {
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

    state.chat_messages.push(ChatEntry {
        role: ChatRole::Assistant,
        content: String::new(),
    });

    *agent_pending = Some(chat::spawn_agent_run(
        ai.clone(),
        history,
        Some(agent_cfg.model),
        agent_cfg.mcp_config_path,
        agent_cfg.mcp_servers,
        agent_cfg.max_steps,
    ));
}
