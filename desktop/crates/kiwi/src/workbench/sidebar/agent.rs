//! Agent sidebar — agent tools and status.

use egui::{Align, Layout, RichText, TextEdit, Ui};
use nest_core::AppContext;
use nest_gui::{ActionButton, ButtonSize};
use nest_icon::Icon;

use crate::agent::{try_persist_preferences, AgentSettings};
use crate::workbench::state::WorkbenchState;
use crate::theme::PALETTE;
use super::SIDEBAR_INNER_WIDTH;
use nest_ai_ollama::OllamaSharedConfig;

/// Renders the agent configuration panel.
pub fn show(ui: &mut Ui, state: &mut WorkbenchState, app_ctx: &AppContext) {
    ui.set_width(SIDEBAR_INNER_WIDTH);

    ui.label(RichText::new("Agent endpoint").strong().size(12.0));
    ui.add_space(4.0);

    ui.label(RichText::new("Host").weak().size(11.0));
    ui.add(
        TextEdit::singleline(&mut state.agent.host)
            .hint_text("192.168.88.10")
            .desired_width(SIDEBAR_INNER_WIDTH),
    );

    ui.add_space(6.0);
    ui.label(RichText::new("Port").weak().size(11.0));
    ui.add(
        TextEdit::singleline(&mut state.agent.port)
            .hint_text("11434")
            .desired_width(96.0),
    );

    ui.add_space(12.0);
    ui.label(RichText::new("Models").strong().size(12.0));
    ui.add_space(4.0);

    let mut remove_index = None;
    let mut selected_model = None;
    let row_height = ui.spacing().interact_size.y;
    for (index, model) in state.agent.models.iter().enumerate() {
        let selected = state.agent.model == *model;
        ui.allocate_ui_with_layout(
            egui::vec2(SIDEBAR_INNER_WIDTH, row_height),
            Layout::left_to_right(Align::Center),
            |ui| {
                if ui.selectable_label(selected, model).clicked() {
                    selected_model = Some(model.clone());
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("Remove").clicked() {
                        remove_index = Some(index);
                    }
                });
            },
        );
    }
    if let Some(index) = remove_index {
        state.agent.remove_model(index);
        let _ = try_persist_preferences(&state.agent, state.agent_mode, app_ctx);
    }
    if let Some(model) = selected_model {
        state.agent.model = model;
        let _ = try_persist_preferences(&state.agent, state.agent_mode, app_ctx);
        apply_agent(&mut state.agent, app_ctx);
        state.sync_model_from_agent();
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.set_width(SIDEBAR_INNER_WIDTH);
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.add(
            TextEdit::singleline(&mut state.agent.new_model)
                .hint_text("Add model…")
                .desired_width(SIDEBAR_INNER_WIDTH - 80.0),
        );
        if ui
            .add(
                ActionButton::new(Icon::PLUS, "Add")
                    .size(ButtonSize::Small)
                    .tooltip("Add model to list"),
            )
            .clicked()
        {
            state.agent.add_model_from_draft();
            let _ = try_persist_preferences(&state.agent, state.agent_mode, app_ctx);
        }
    });

    ui.add_space(12.0);
    let mut apply_and_save = false;
    ui.horizontal(|ui| {
        ui.set_width(SIDEBAR_INNER_WIDTH);
        ui.spacing_mut().item_spacing.x = 8.0;
        if ui
            .add(
                ActionButton::new(Icon::CHECK, "Apply & Save")
                    .size(ButtonSize::Small)
                    .fill(PALETTE.accent_primary)
                    .text_color(egui::Color32::WHITE)
                    .tooltip("Apply settings and save to config.toml"),
            )
            .clicked()
        {
            apply_and_save = true;
        }
    });
    if apply_and_save {
        apply_and_save_agent(&mut state.agent, state.agent_mode, app_ctx);
        state.sync_model_from_agent();
    }

    if let Some(status) = &state.agent.status {
        ui.add_space(6.0);
        ui.label(RichText::new(status).weak().size(11.0));
    }

    ui.add_space(12.0);
    ui.label(RichText::new("MCP servers").strong().size(12.0));
    ui.add_space(4.0);

    if state.agent_mcp_servers.is_empty() {
        ui.label(
            RichText::new("Configure [agent].mcp_servers in config.toml")
                .weak()
                .size(11.0),
        );
    } else {
        let mut toggled: Option<(String, bool)> = None;
        for server in &state.agent_mcp_servers {
            let enabled = !state.agent.disabled_mcp_servers.contains(server);
            let mut checked = enabled;
            if ui.checkbox(&mut checked, server).changed() {
                toggled = Some((server.clone(), checked));
            }
        }
        if let Some((server, enabled)) = toggled {
            if enabled {
                state
                    .agent
                    .disabled_mcp_servers
                    .retain(|name| name != &server);
            } else if !state.agent.disabled_mcp_servers.contains(&server) {
                state.agent.disabled_mcp_servers.push(server);
            }
            let _ = try_persist_preferences(&state.agent, state.agent_mode, app_ctx);
        }
    }

    ui.add_space(6.0);
    if ui
        .checkbox(
            &mut state.agent.allow_save_context,
            "Allow save_context_memory",
        )
        .on_hover_text("When enabled, the agent may auto-run save_context_memory without asking")
        .changed()
    {
        let _ = try_persist_preferences(&state.agent, state.agent_mode, app_ctx);
    }

    if let Some(status) = &state.agent.mcp_status {
        ui.label(RichText::new(status).weak().size(11.0));
    } else {
        ui.label(
            RichText::new("Configure [agent] in config.toml")
                .weak()
                .size(11.0),
        );
    }

    if let Some(count) = state.agent.mcp_tool_count {
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("{count} tool calls this session"))
                .weak()
                .size(11.0),
        );
    }

    ui.add_space(4.0);
    ui.label(
        RichText::new("See Nest tools/MCP-SETUP.md")
            .weak()
            .size(11.0),
    );
}

fn apply_agent(agent: &mut AgentSettings, app_ctx: &AppContext) {
    let Ok(shared) = app_ctx.service::<OllamaSharedConfig>() else {
        agent.status = Some("Ollama service not available".into());
        return;
    };
    agent.apply_runtime(&shared);
}

fn apply_and_save_agent(agent: &mut AgentSettings, agent_mode: bool, app_ctx: &AppContext) {
    apply_agent(agent, app_ctx);
    match try_persist_preferences(agent, agent_mode, app_ctx) {
        Ok(path) => {
            agent.status = Some(format!(
                "Saved to {path} (model: {}, agent mode: {agent_mode})",
                agent.model
            ));
        }
        Err(error) => agent.status = Some(error.to_string()),
    }
}
