//! Settings dock panel — displays and edits adjustable app settings.

use egui::{DragValue, RichText, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::theme::SemanticRole;

use crate::dock::context::PanelContext;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let muted = ctx.theme.role(SemanticRole::Muted);
    let accent = ctx.theme.role(SemanticRole::Accent);

    ui.add_space(4.0);
    ui.label(
        RichText::new("Changes apply immediately. Edit kiwi.toml to persist across restarts.")
            .color(muted)
            .small(),
    );
    ui.add_space(8.0);

    egui::CollapsingHeader::new(RichText::new("Appearance").color(accent))
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Font size");
                ui.add(
                    DragValue::new(&mut ctx.state.config.gui.font_size)
                        .speed(0.25)
                        .range(8.0_f32..=32.0),
                );
                ui.label("pt");
            });
            let theme_name = ctx.state.config.theme.name.clone();
            read_only_row(ui, muted, "Theme", &theme_name);
        });

    ui.add_space(4.0);

    egui::CollapsingHeader::new(RichText::new("Preview").color(accent))
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(&mut ctx.state.config.preview.line_numbers, "Line numbers");
            ui.checkbox(&mut ctx.state.config.preview.wrap, "Word wrap");
        });

    ui.add_space(4.0);

    egui::CollapsingHeader::new(RichText::new("Diff").color(accent))
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(&mut ctx.state.config.diff.word_wrap, "Word wrap");
            ui.horizontal(|ui| {
                ui.label("Context lines");
                ui.add(DragValue::new(&mut ctx.state.config.diff.context_lines).range(0_u32..=20));
            });
        });

    ui.add_space(4.0);

    egui::CollapsingHeader::new(RichText::new("Git").color(accent))
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(&mut ctx.state.config.git.show_untracked, "Show untracked files");
        });

    ui.add_space(4.0);

    egui::CollapsingHeader::new(RichText::new("Status Bar").color(accent))
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(
                &mut ctx.state.config.status_bar.show_issue,
                "Show current issue number",
            );
        });

    ui.add_space(4.0);

    egui::CollapsingHeader::new(RichText::new("Workspace").color(accent))
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(&mut ctx.state.config.workspace.persist, "Persist layout on exit");
            ui.horizontal(|ui| {
                ui.label("Auto-save interval");
                ui.add(
                    DragValue::new(&mut ctx.state.config.workspace.save_interval_secs)
                        .range(5_u64..=3600),
                );
                ui.label("s");
            });
        });

    ui.add_space(4.0);

    egui::CollapsingHeader::new(RichText::new("Search").color(accent))
        .default_open(false)
        .show(ui, |ui| {
            let search_cmd = ctx.state.config.search.command.clone();
            ui.horizontal(|ui| {
                ui.label("Debounce");
                ui.add(
                    DragValue::new(&mut ctx.state.config.search.debounce_ms).range(0_u64..=2000),
                );
                ui.label("ms");
            });
            read_only_row(ui, muted, "Command", &search_cmd);
        });

    ui.add_space(4.0);

    // Agents section — dropdown of agent plugins with an explicit Apply button.
    {
        let agent_plugins: Vec<_> = ctx
            .state
            .plugins
            .available
            .iter()
            .filter(|p| p.agent_command.is_some())
            .cloned()
            .collect();

        if !agent_plugins.is_empty() {
            egui::CollapsingHeader::new(RichText::new("Agents").color(accent))
                .default_open(true)
                .show(ui, |ui| {
                    let current_cmd = ctx.state.config.agent.command.clone();

                    // Persistent pending selection — survives across frames without
                    // changing the live config until the user clicks Apply.
                    let pending_id = egui::Id::new("agent_pending_selection");
                    let mut pending: (String, Vec<String>) = ui
                        .ctx()
                        .data_mut(|d| d.get_temp::<(String, Vec<String>)>(pending_id))
                        .unwrap_or_else(|| (current_cmd.clone(), ctx.state.config.agent.args.clone()));

                    // Determine display label for the current pending selection.
                    let pending_label = agent_plugins
                        .iter()
                        .find(|p| p.agent_command.as_deref() == Some(&pending.0))
                        .map(|p| p.display_name.clone())
                        .unwrap_or_else(|| format!("Custom ({})", pending.0));

                    egui::ComboBox::from_id_salt("agent_select")
                        .selected_text(&pending_label)
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            for plugin in &agent_plugins {
                                let cmd = plugin.agent_command.as_deref().unwrap_or("").to_string();
                                let args = plugin.agent_args.clone();
                                if ui
                                    .selectable_label(pending.0 == cmd, &plugin.display_name)
                                    .clicked()
                                {
                                    pending = (cmd, args);
                                }
                            }
                        });

                    // Persist the pending selection across frames.
                    ui.ctx().data_mut(|d| d.insert_temp(pending_id, pending.clone()));

                    ui.add_space(4.0);

                    let changed = pending.0 != current_cmd;
                    let btn = ui.add_enabled(changed, egui::Button::new("Apply & Restart Agent"));
                    if btn.clicked() {
                        (ctx.dispatch)(AppCommand::SetAgent {
                            command: pending.0.clone(),
                            args: pending.1.clone(),
                        });
                    }

                    if !changed {
                        ui.add_space(2.0);
                        let active_label = agent_plugins
                            .iter()
                            .find(|p| p.agent_command.as_deref() == Some(&current_cmd))
                            .map(|p| p.display_name.as_str())
                            .unwrap_or("Custom");
                        ui.label(
                            RichText::new(format!("Active: {active_label}"))
                                .color(muted)
                                .small(),
                        );
                    }
                });

            ui.add_space(4.0);
        }
    }

    egui::CollapsingHeader::new(RichText::new("Commands").color(accent))
        .default_open(false)
        .show(ui, |ui| {
            let editor = ctx
                .state
                .config
                .editor
                .configured_command
                .clone()
                .unwrap_or_else(|| "$VISUAL / $EDITOR / nano".to_string());
            let shell = ctx.state.config.shell.command.clone();
            let agent = ctx.state.config.agent.command.clone();
            let gh = ctx.state.config.github.command.clone();
            ui.label(RichText::new("Set in kiwi.toml — restart required to take effect.").color(muted).small());
            read_only_row(ui, muted, "Editor", &editor);
            read_only_row(ui, muted, "Shell", &shell);
            read_only_row(ui, muted, "Agent", &agent);
            read_only_row(ui, muted, "GitHub CLI", &gh);
        });
}

fn read_only_row(ui: &mut Ui, muted: egui::Color32, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.colored_label(muted, value);
    });
}
