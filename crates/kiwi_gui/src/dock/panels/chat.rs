//! Native chat panel for API-based agents (Phase 4 — issue #332).
//!
//! Renders when `AgentState.chat` is `Some`. Falls back to the PTY path in `agent.rs`
//! when `chat` is `None`.

use egui::{Color32, CornerRadius, Frame, Key, Margin, Modifiers, RichText, ScrollArea, Stroke, Ui};
use kiwi_core::agent::{AgentId, AgentStatus, ChatMessage, ChatSession, ContentBlock, MessageRole, ToolUse};
use kiwi_core::events::AppCommand;
use kiwi_core::navigation::{FocusTarget, NavCommand};
use kiwi_core::theme::SemanticRole;

use crate::dock::context::PanelContext;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let agent_id = ctx.state.agent_manager.active_id();
    let mut commands: Vec<AppCommand> = Vec::new();

    render_chrome(ui, ctx, agent_id, &mut commands);

    // Reserve space for the input box at the bottom before sizing the scroll area,
    // so the input is always visible without scrolling.
    let input_reserve = 100.0_f32;
    let list_height = (ui.available_height() - input_reserve).max(60.0);

    // Message list — immutable borrow ends before input_box borrow begins.
    {
        let agent = ctx.state.active_agent();
        if let Some(chat) = &agent.chat {
            render_message_list(ui, ctx.theme, chat, agent_id, &mut commands, list_height);
        } else {
            ui.centered_and_justified(|ui| {
                ui.colored_label(ctx.theme.role(SemanticRole::Muted), "No chat session active.");
            });
        }
    }

    // Input box — needs mutable access to `input_draft`.
    {
        let agent = ctx.state.active_agent_mut();
        if let Some(chat) = &mut agent.chat {
            render_input_box(ui, ctx.theme, chat, agent_id, &mut commands);
        }
    }

    for cmd in commands {
        let _ = (ctx.dispatch)(cmd);
    }
}

// ---------------------------------------------------------------------------
// Chrome (status bar + session tabs)
// ---------------------------------------------------------------------------

fn render_chrome(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    agent_id: AgentId,
    commands: &mut Vec<AppCommand>,
) {
    let (status_text, role, model_label, error) = {
        let agent = ctx.state.active_agent();
        if let Some(chat) = &agent.chat {
            let (text, role) = status_label_role(&chat.status, chat.is_streaming);
            (text, role, chat.model.clone(), chat.error.clone())
        } else {
            ("no session", SemanticRole::Muted, String::new(), None)
        }
    };

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.colored_label(
            ctx.theme.role(role),
            RichText::new(format!("Agent · {status_text}")).strong(),
        );
        ui.colored_label(ctx.theme.role(SemanticRole::Muted), &model_label);

        // Session tabs when more than one session exists
        if ctx.state.agent_manager.session_count() > 1 {
            ui.separator();
            let sessions: Vec<_> = ctx
                .state
                .agent_manager
                .sessions()
                .map(|s| (s.id, s.label.clone()))
                .collect();
            let active_id = ctx.state.agent_manager.active_id();
            for (id, label) in sessions {
                let selected = id == active_id;
                if ui.selectable_label(selected, &label).clicked() && !selected {
                    commands.push(AppCommand::AgentSetActive(id));
                }
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("Clear").clicked() {
                commands.push(AppCommand::AgentClearHistory { agent_id });
            }
        });
    });

    if let Some(err) = error {
        Frame::new()
            .fill(ctx.theme.role(SemanticRole::AgentError).linear_multiply(0.15))
            .inner_margin(Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.colored_label(ctx.theme.role(SemanticRole::AgentError), &err);
            });
    }

    ui.separator();
}

fn status_label_role(status: &AgentStatus, is_streaming: bool) -> (&'static str, SemanticRole) {
    if is_streaming {
        return match status {
            AgentStatus::Executing => ("executing…", SemanticRole::AgentExecuting),
            _ => ("thinking…", SemanticRole::AgentThinking),
        };
    }
    match status {
        AgentStatus::Idle => ("idle", SemanticRole::Muted),
        AgentStatus::Thinking => ("thinking", SemanticRole::AgentThinking),
        AgentStatus::Executing => ("executing", SemanticRole::AgentExecuting),
        AgentStatus::Success => ("done", SemanticRole::AgentSuccess),
        AgentStatus::Error => ("error", SemanticRole::AgentError),
        AgentStatus::Warning => ("warning", SemanticRole::AgentWarning),
    }
}

// ---------------------------------------------------------------------------
// Message list
// ---------------------------------------------------------------------------

fn render_message_list(
    ui: &mut Ui,
    theme: &crate::theme::GuiTheme,
    chat: &ChatSession,
    agent_id: AgentId,
    commands: &mut Vec<AppCommand>,
    max_height: f32,
) {
    let is_streaming = chat.is_streaming;

    ScrollArea::vertical()
        .id_salt("chat_msg_scroll")
        .stick_to_bottom(is_streaming && chat.follow_tail)
        .max_height(max_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

            if chat.messages.is_empty() && !is_streaming {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.colored_label(
                        theme.role(SemanticRole::Muted),
                        "Start a conversation with your agent.",
                    );
                });
                return;
            }

            for (i, msg) in chat.messages.iter().enumerate() {
                render_message(ui, theme, msg, i, agent_id, commands);
                ui.add_space(4.0);
            }

            if is_streaming {
                if !chat.streaming_text.is_empty() {
                    let display = format!("{}▋", chat.streaming_text);
                    ui.label(&display);
                } else {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.colored_label(theme.role(SemanticRole::Muted), "▋");
                    });
                }
            }

            ui.add_space(8.0);
        });
}

fn render_message(
    ui: &mut Ui,
    theme: &crate::theme::GuiTheme,
    msg: &ChatMessage,
    msg_index: usize,
    agent_id: AgentId,
    commands: &mut Vec<AppCommand>,
) {
    match msg.role {
        MessageRole::User => {
            let text: String = msg
                .blocks
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::Text(t) => Some(t.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            if text.is_empty() {
                return;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                Frame::new()
                    .fill(theme.role(SemanticRole::Accent).linear_multiply(0.2))
                    .stroke(Stroke::new(
                        1.0,
                        theme.role(SemanticRole::Accent).linear_multiply(0.4),
                    ))
                    .inner_margin(Margin::symmetric(10, 6))
                    .corner_radius(CornerRadius::same(6))
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width() * 0.75);
                        ui.label(&text);
                    });
            });
        }
        MessageRole::Assistant => {
            ui.vertical(|ui| {
                ui.add_space(2.0);
                for block in &msg.blocks {
                    match block {
                        ContentBlock::Text(text) if !text.is_empty() => {
                            ui.label(text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            render_tool_widget(ui, theme, tool, agent_id, commands);
                        }
                        _ => {}
                    }
                    ui.add_space(2.0);
                }
            });
        }
    }
    let _ = msg_index; // reserved for future per-message id use
}

// ---------------------------------------------------------------------------
// Tool widget
// ---------------------------------------------------------------------------

fn render_tool_widget(
    ui: &mut Ui,
    theme: &crate::theme::GuiTheme,
    tool: &ToolUse,
    agent_id: AgentId,
    commands: &mut Vec<AppCommand>,
) {
    let is_bash = tool.name == "run_bash";
    let fill = if is_bash {
        theme.role(SemanticRole::AgentExecuting).linear_multiply(0.15)
    } else {
        theme.role(SemanticRole::Muted).linear_multiply(0.1)
    };

    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(
            1.0,
            theme.role(SemanticRole::Muted).linear_multiply(0.3),
        ))
        .inner_margin(Margin::symmetric(8, 4))
        .corner_radius(CornerRadius::same(4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let arrow = if tool.collapsed { "▶" } else { "▼" };
                let label = RichText::new(format!("{arrow} {}", tool.name))
                    .monospace()
                    .strong()
                    .color(theme.role(SemanticRole::Accent));
                if ui.label(label).clicked() {
                    commands.push(AppCommand::AgentToggleToolExpand {
                        agent_id,
                        tool_use_id: tool.id.clone(),
                    });
                }

                if is_bash {
                    ui.add_space(6.0);
                    if ui.small_button("→ Terminal").clicked() {
                        commands.push(AppCommand::Navigation(NavCommand::SetFocus(
                            FocusTarget::Shell,
                        )));
                    }
                }
            });

            if !tool.collapsed {
                ui.add_space(2.0);
                let pretty = serde_json::from_str::<serde_json::Value>(&tool.input_json)
                    .map(|v| {
                        serde_json::to_string_pretty(&v)
                            .unwrap_or_else(|_| tool.input_json.clone())
                    })
                    .unwrap_or_else(|_| tool.input_json.clone());
                Frame::new()
                    .fill(Color32::BLACK.linear_multiply(0.3))
                    .inner_margin(Margin::symmetric(6, 4))
                    .corner_radius(CornerRadius::same(3))
                    .show(ui, |ui| {
                        ui.label(RichText::new(&pretty).monospace().small());
                    });
            }
        });
}

// ---------------------------------------------------------------------------
// Input box
// ---------------------------------------------------------------------------

fn render_input_box(
    ui: &mut Ui,
    _theme: &crate::theme::GuiTheme,
    chat: &mut ChatSession,
    agent_id: AgentId,
    commands: &mut Vec<AppCommand>,
) {
    let is_streaming = chat.is_streaming;

    ui.separator();
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.add_space(4.0);

        let hint = if is_streaming {
            "Waiting for response…"
        } else {
            "Type a message…  (Ctrl+Enter to send)"
        };

        let text_edit = egui::TextEdit::multiline(&mut chat.input_draft)
            .hint_text(hint)
            .desired_rows(3)
            .desired_width(ui.available_width() - 70.0)
            .frame(true);

        let response = ui.add_enabled(!is_streaming, text_edit);

        // Ctrl+Enter → send
        if !is_streaming
            && response.has_focus()
            && ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Enter))
        {
            let text = chat.input_draft.trim().to_string();
            if !text.is_empty() {
                chat.input_draft.clear();
                commands.push(AppCommand::AgentUserSend { agent_id, text });
            }
        }

        // Escape → clear draft
        if response.has_focus()
            && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape))
        {
            chat.input_draft.clear();
        }

        ui.vertical(|ui| {
            let can_send = !is_streaming && !chat.input_draft.trim().is_empty();
            if ui.add_enabled(can_send, egui::Button::new("Send")).clicked() {
                let text = chat.input_draft.trim().to_string();
                if !text.is_empty() {
                    chat.input_draft.clear();
                    commands.push(AppCommand::AgentUserSend { agent_id, text });
                }
            }
        });
    });

    ui.add_space(4.0);
}
