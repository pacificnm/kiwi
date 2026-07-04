//! Agent sidebar — agent tools and status.

use egui::{Align, Layout, RichText, TextEdit, Ui};
use nest_config::ConfigService;
use nest_core::AppContext;
use nest_gui::{ActionButton, ButtonSize};
use nest_icon::Icon;

use crate::agent::AgentSettings;
use crate::theme::PALETTE;
use super::SIDEBAR_INNER_WIDTH;
use nest_ai_ollama::OllamaSharedConfig;

/// Renders the agent configuration panel.
pub fn show(ui: &mut Ui, agent: &mut AgentSettings, app_ctx: &AppContext) {
    ui.set_width(SIDEBAR_INNER_WIDTH);

    ui.label(RichText::new("Agent endpoint").strong().size(12.0));
    ui.add_space(4.0);

    ui.label(RichText::new("Host").weak().size(11.0));
    ui.add(
        TextEdit::singleline(&mut agent.host)
            .hint_text("192.168.88.10")
            .desired_width(SIDEBAR_INNER_WIDTH),
    );

    ui.add_space(6.0);
    ui.label(RichText::new("Port").weak().size(11.0));
    ui.add(
        TextEdit::singleline(&mut agent.port)
            .hint_text("11434")
            .desired_width(96.0),
    );

    ui.add_space(12.0);
    ui.label(RichText::new("Models").strong().size(12.0));
    ui.add_space(4.0);

    let mut remove_index = None;
    let row_height = ui.spacing().interact_size.y;
    for (index, model) in agent.models.iter().enumerate() {
        let selected = agent.model == *model;
        ui.allocate_ui_with_layout(
            egui::vec2(SIDEBAR_INNER_WIDTH, row_height),
            Layout::left_to_right(Align::Center),
            |ui| {
                if ui.selectable_label(selected, model).clicked() {
                    agent.model = model.clone();
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
        agent.remove_model(index);
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.set_width(SIDEBAR_INNER_WIDTH);
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.add(
            TextEdit::singleline(&mut agent.new_model)
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
            agent.add_model_from_draft();
        }
    });

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.set_width(SIDEBAR_INNER_WIDTH);
        ui.spacing_mut().item_spacing.x = 8.0;
        if ui
            .add(
                ActionButton::new(Icon::CHECK, "Apply")
                    .size(ButtonSize::Small)
                    .fill(PALETTE.accent_primary)
                    .text_color(egui::Color32::WHITE)
                    .tooltip("Apply agent settings"),
            )
            .clicked()
        {
            apply_agent(agent, app_ctx);
        }
        if ui
            .add(
                ActionButton::new(Icon::CHECK, "Save")
                    .size(ButtonSize::Small)
                    .tooltip("Save agent settings to config"),
            )
            .clicked()
        {
            save_agent(agent, app_ctx);
        }
    });

    if let Some(status) = &agent.status {
        ui.add_space(6.0);
        ui.label(RichText::new(status).weak().size(11.0));
    }

    ui.add_space(12.0);
    ui.label(RichText::new("MCP servers").strong().size(12.0));
    ui.add_space(4.0);

    if let Some(status) = &agent.mcp_status {
        ui.label(RichText::new(status).weak().size(11.0));
    } else {
        ui.label(
            RichText::new("Configure [agent] in config.toml")
                .weak()
                .size(11.0),
        );
    }

    if let Some(count) = agent.mcp_tool_count {
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
    agent.status = Some(format!(
        "Connected to {} (model: {})",
        agent.base_url(),
        agent.model
    ));
}

fn save_agent(agent: &mut AgentSettings, app_ctx: &AppContext) {
    let Ok(config) = app_ctx.service::<ConfigService>() else {
        agent.status = Some("Config service not available".into());
        return;
    };
    let Some(path) = config.path() else {
        agent.status = Some("No config file path — use --config".into());
        return;
    };

    match agent.save_to_config_path(path) {
        Ok(()) => {
            apply_agent(agent, app_ctx);
            agent.status = Some(format!("Saved to {}", path.display()));
        }
        Err(error) => agent.status = Some(error.to_string()),
    }
}
