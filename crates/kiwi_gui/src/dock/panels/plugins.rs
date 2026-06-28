//! Plugin Manager dock panel — lists available plugins with install/enable/disable actions.

use egui::{Color32, RichText, ScrollArea, SelectableLabel, TextEdit, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::state::AvailablePlugin;
use kiwi_core::theme::SemanticRole;

use crate::dock::context::PanelContext;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let available = ctx.state.plugins.available.clone();

    let install_footer_height = 60.0;
    let main_height = (ui.available_height() - install_footer_height - 8.0).max(80.0);

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), main_height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            if available.is_empty() {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("No plugins found in plugins/ directory.")
                        .color(ctx.theme.role(SemanticRole::Muted))
                        .small(),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Use the Install form below to install from a custom path.")
                        .color(ctx.theme.role(SemanticRole::Muted))
                        .small(),
                );
            } else {
                render_plugin_list(ui, ctx, &available);
            }
        },
    );

    ui.separator();
    render_install_footer(ui, ctx);
}

fn render_plugin_list(ui: &mut Ui, ctx: &mut PanelContext<'_>, available: &[AvailablePlugin]) {
    let selected = ctx
        .state
        .plugins
        .selected_index
        .min(available.len().saturating_sub(1));

    let total_width = ui.available_width();
    let list_width = (total_width * 0.40).max(150.0);

    ui.horizontal_top(|ui| {
        // --- Plugin list ---
        ui.allocate_ui_with_layout(
            egui::vec2(list_width, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ScrollArea::vertical()
                    .id_salt("plugin_list_scroll")
                    .show(ui, |ui| {
                        for (i, plugin) in available.iter().enumerate() {
                            let badge = status_badge(plugin);
                            let label = format!("{badge} {}", plugin.display_name);
                            let color = if plugin.installed && plugin.enabled {
                                ctx.theme.role(SemanticRole::Fg)
                            } else if plugin.installed {
                                ctx.theme.role(SemanticRole::Muted)
                            } else {
                                ctx.theme.role(SemanticRole::Accent)
                            };
                            let rich = RichText::new(label).color(color);

                            ui.horizontal(|ui| {
                                let row = ui.add(SelectableLabel::new(i == selected, rich));
                                if row.clicked() {
                                    ctx.state.plugins.selected_index = i;
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        render_action_button(ui, ctx, plugin);
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
                        render_detail(ui, &available[selected], ctx);
                    });
            },
        );
    });
}

fn render_action_button(ui: &mut Ui, ctx: &mut PanelContext<'_>, plugin: &AvailablePlugin) {
    if !plugin.installed {
        let src = plugin.source_path.clone();
        if ui.small_button("Install").clicked() {
            (ctx.dispatch)(AppCommand::PluginInstall { src_path: src });
        }
    } else if plugin.enabled {
        let name = plugin.name.clone();
        if ui.small_button("Disable").clicked() {
            (ctx.dispatch)(AppCommand::PluginSetEnabled { name, enabled: false });
        }
    } else {
        let name = plugin.name.clone();
        if ui.small_button("Enable").clicked() {
            (ctx.dispatch)(AppCommand::PluginSetEnabled { name, enabled: true });
        }
    }
}

fn render_install_footer(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let muted = ctx.theme.role(SemanticRole::Muted);
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Install from custom path:").color(muted).small());
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

fn render_detail(ui: &mut Ui, plugin: &AvailablePlugin, ctx: &PanelContext<'_>) {
    let accent = ctx.theme.role(SemanticRole::Accent);
    let muted = ctx.theme.role(SemanticRole::Muted);
    let fg = ctx.theme.role(SemanticRole::Fg);

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new(&plugin.display_name).color(accent).strong().size(15.0));
        ui.label(RichText::new(format!("v{}", plugin.version)).color(muted).small());
    });

    // Status
    let (status_text, status_color) = if !plugin.installed {
        ("Not installed", Color32::from_rgb(150, 150, 220))
    } else if plugin.enabled {
        ("Installed — enabled", Color32::from_rgb(100, 200, 100))
    } else {
        ("Installed — disabled", Color32::GRAY)
    };
    ui.horizontal(|ui| {
        ui.label(RichText::new("Status:").color(muted).small());
        ui.label(RichText::new(status_text).color(status_color).small());
    });

    if !plugin.author.is_empty() {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Author:").color(muted).small());
            ui.label(RichText::new(&plugin.author).color(fg).small());
        });
    }

    if !plugin.description.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new(&plugin.description).color(fg));
    }

    ui.add_space(6.0);
    ui.label(
        RichText::new(plugin.source_path.to_string_lossy().as_ref())
            .color(muted)
            .small()
            .monospace(),
    );

    if plugin.installed {
        ui.add_space(6.0);
        ui.label(
            RichText::new("Changes take effect after restart.")
                .color(muted)
                .small(),
        );
    }
}

fn status_badge(plugin: &AvailablePlugin) -> &'static str {
    if !plugin.installed {
        "○"
    } else if plugin.enabled {
        "●"
    } else {
        "◐"
    }
}
