//! Plugin Manager dock panel — lists installed plugins with status, enable/disable, and install.

use egui::{Color32, RichText, ScrollArea, SelectableLabel, TextEdit, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::state::{PluginEntry, PluginStatus};
use kiwi_core::theme::SemanticRole;

use crate::dock::context::PanelContext;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let entries = ctx.state.plugins.entries.clone();

    // Install-from-directory section always shown at the bottom.
    // We split the panel vertically: list/detail takes remaining space, install footer is fixed.
    let install_height = 60.0;
    let main_height = (ui.available_height() - install_height - 8.0).max(80.0);

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), main_height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            if entries.is_empty() {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("No plugins installed.")
                        .color(ctx.theme.role(SemanticRole::Muted))
                        .small(),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Install a plugin using the form below.")
                        .color(ctx.theme.role(SemanticRole::Muted))
                        .small(),
                );
            } else {
                render_plugin_list(ui, ctx, &entries);
            }
        },
    );

    ui.separator();
    render_install_footer(ui, ctx);
}

fn render_plugin_list(ui: &mut Ui, ctx: &mut PanelContext<'_>, entries: &[PluginEntry]) {
    let selected = ctx
        .state
        .plugins
        .selected_index
        .min(entries.len().saturating_sub(1));

    let available = ui.available_width();
    let list_width = (available * 0.38).max(140.0);

    ui.horizontal_top(|ui| {
        // --- Plugin list (with enable/disable buttons) ---
        ui.allocate_ui_with_layout(
            egui::vec2(list_width, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ScrollArea::vertical()
                    .id_salt("plugin_list_scroll")
                    .show(ui, |ui| {
                        for (i, entry) in entries.iter().enumerate() {
                            let (badge, _) = status_badge(&entry.status);
                            let label = format!("{badge} {}", entry.display_name);
                            let rich = RichText::new(label).color(if entry.enabled {
                                ctx.theme.role(SemanticRole::Fg)
                            } else {
                                ctx.theme.role(SemanticRole::Muted)
                            });

                            ui.horizontal(|ui| {
                                let row = ui.add(SelectableLabel::new(i == selected, rich));
                                if row.clicked() {
                                    ctx.state.plugins.selected_index = i;
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let (btn_label, new_enabled) = if entry.enabled {
                                            ("Disable", false)
                                        } else {
                                            ("Enable", true)
                                        };
                                        let name = entry.name.clone();
                                        if ui.small_button(btn_label).clicked() {
                                            (ctx.dispatch)(AppCommand::PluginSetEnabled {
                                                name,
                                                enabled: new_enabled,
                                            });
                                        }
                                    },
                                );
                            });
                        }
                    });
            },
        );

        ui.separator();

        // --- Detail panel ---
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ScrollArea::vertical()
                    .id_salt("plugin_detail_scroll")
                    .show(ui, |ui| {
                        render_detail(ui, &entries[selected], ctx);
                    });
            },
        );
    });
}

fn render_install_footer(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let muted = ctx.theme.role(SemanticRole::Muted);

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Install from directory:").color(muted).small());
    });
    ui.horizontal(|ui| {
        let input = TextEdit::singleline(&mut ctx.state.plugins.install_path_input)
            .hint_text("/path/to/plugin")
            .desired_width(ui.available_width() - 72.0);
        ui.add(input);

        let has_path = !ctx.state.plugins.install_path_input.trim().is_empty();
        let btn = ui.add_enabled(has_path, egui::Button::new("Install"));
        if btn.clicked() {
            let src = std::path::PathBuf::from(ctx.state.plugins.install_path_input.trim());
            (ctx.dispatch)(AppCommand::PluginInstall { src_path: src });
        }
    });
}

fn render_detail(ui: &mut Ui, entry: &PluginEntry, ctx: &PanelContext<'_>) {
    let accent = ctx.theme.role(SemanticRole::Accent);
    let muted = ctx.theme.role(SemanticRole::Muted);
    let fg = ctx.theme.role(SemanticRole::Fg);

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new(&entry.display_name).color(accent).strong().size(15.0));
        ui.label(RichText::new(format!("v{}", entry.version)).color(muted).small());
    });

    let (badge, badge_color) = status_badge_color(&entry.status);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Status:").color(muted).small());
        ui.label(RichText::new(badge).color(badge_color).small());
        if !entry.enabled {
            ui.label(RichText::new("(disabled)").color(muted).small());
        }
    });

    if !entry.author.is_empty() {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Author:").color(muted).small());
            ui.label(RichText::new(&entry.author).color(fg).small());
        });
    }

    if !entry.description.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new(&entry.description).color(fg));
    }

    if !entry.command_ids.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new("Commands").color(muted).small());
        for id in &entry.command_ids {
            ui.horizontal(|ui| {
                ui.label(RichText::new("•").color(muted));
                ui.label(RichText::new(id).color(fg).small().monospace());
            });
        }
    }

    match &entry.status {
        PluginStatus::Failed(reason) | PluginStatus::Incompatible(reason) => {
            ui.add_space(8.0);
            ui.label(RichText::new("Error").color(muted).small());
            ui.label(
                RichText::new(reason)
                    .color(ctx.theme.role(SemanticRole::AgentError))
                    .small(),
            );
        }
        _ => {}
    }

    ui.add_space(8.0);
    ui.label(
        RichText::new("Enable/Disable changes take effect after restart.")
            .color(muted)
            .small(),
    );
}

fn status_badge(status: &PluginStatus) -> (&'static str, Color32) {
    match status {
        PluginStatus::Loaded => ("●", Color32::from_rgb(100, 200, 100)),
        PluginStatus::Disabled => ("○", Color32::GRAY),
        PluginStatus::Failed(_) => ("✗", Color32::from_rgb(220, 80, 80)),
        PluginStatus::Incompatible(_) => ("⚠", Color32::from_rgb(220, 160, 60)),
        PluginStatus::Missing => ("?", Color32::from_rgb(220, 80, 80)),
    }
}

fn status_badge_color(status: &PluginStatus) -> (String, Color32) {
    let (sym, color) = match status {
        PluginStatus::Loaded => ("● Loaded", Color32::from_rgb(100, 200, 100)),
        PluginStatus::Disabled => ("○ Disabled", Color32::GRAY),
        PluginStatus::Failed(_) => ("✗ Failed", Color32::from_rgb(220, 80, 80)),
        PluginStatus::Incompatible(_) => ("⚠ Incompatible", Color32::from_rgb(220, 160, 60)),
        PluginStatus::Missing => ("? Missing", Color32::from_rgb(220, 80, 80)),
    };
    (sym.to_string(), color)
}
