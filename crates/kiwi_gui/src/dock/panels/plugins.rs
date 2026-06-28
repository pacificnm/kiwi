//! Plugin Manager dock panel — lists installed plugins with status and details.

use egui::{Color32, RichText, ScrollArea, SelectableLabel, Ui};
use kiwi_core::state::{PluginEntry, PluginStatus};
use kiwi_core::theme::SemanticRole;

use crate::dock::context::PanelContext;

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let entries = ctx.state.plugins.entries.clone();

    if entries.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new("No plugins installed.")
                .color(ctx.theme.role(SemanticRole::Muted))
                .small(),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Use `kiwi plugin install <path>` in a terminal to add a plugin.")
                .color(ctx.theme.role(SemanticRole::Muted))
                .small(),
        );
        return;
    }

    let selected = ctx
        .state
        .plugins
        .selected_index
        .min(entries.len().saturating_sub(1));

    // Two-column layout: list on left, detail on right.
    let available = ui.available_width();
    let list_width = (available * 0.38).max(140.0);

    ui.horizontal_top(|ui| {
        // --- Plugin list ---
        ui.allocate_ui_with_layout(
            egui::vec2(list_width, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ScrollArea::vertical()
                    .id_salt("plugin_list_scroll")
                    .show(ui, |ui| {
                        for (i, entry) in entries.iter().enumerate() {
                            let (badge, badge_color) = status_badge(&entry.status, ctx);
                            let label = format!("{badge} {}", entry.display_name);
                            let rich = RichText::new(label).color(if entry.enabled {
                                ctx.theme.role(SemanticRole::Fg)
                            } else {
                                ctx.theme.role(SemanticRole::Muted)
                            });

                            let row = ui.add(SelectableLabel::new(i == selected, rich));
                            if row.clicked() {
                                ctx.state.plugins.selected_index = i;
                            }

                            // Show status badge color on the badge character
                            let _ = badge_color;
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

fn render_detail(ui: &mut Ui, entry: &PluginEntry, ctx: &PanelContext<'_>) {
    let accent = ctx.theme.role(SemanticRole::Accent);
    let muted = ctx.theme.role(SemanticRole::Muted);
    let fg = ctx.theme.role(SemanticRole::Fg);

    ui.add_space(4.0);

    // Name + version
    ui.horizontal(|ui| {
        ui.label(RichText::new(&entry.display_name).color(accent).strong().size(15.0));
        ui.label(RichText::new(format!("v{}", entry.version)).color(muted).small());
    });

    // Status badge
    let (badge, badge_color) = status_badge_color(&entry.status, ctx);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Status:").color(muted).small());
        ui.label(RichText::new(badge).color(badge_color).small());
        if !entry.enabled {
            ui.label(RichText::new("(disabled)").color(muted).small());
        }
    });

    // Author
    if !entry.author.is_empty() {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Author:").color(muted).small());
            ui.label(RichText::new(&entry.author).color(fg).small());
        });
    }

    // Description
    if !entry.description.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new(&entry.description).color(fg));
    }

    // Commands
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

    // Failure / incompatibility reason
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

    ui.add_space(12.0);
    ui.label(
        RichText::new("Run `kiwi plugin enable/disable <name>` and restart to toggle.")
            .color(muted)
            .small(),
    );
}

fn status_badge(status: &PluginStatus, _ctx: &PanelContext<'_>) -> (&'static str, Color32) {
    match status {
        PluginStatus::Loaded => ("●", Color32::from_rgb(100, 200, 100)),
        PluginStatus::Disabled => ("○", Color32::GRAY),
        PluginStatus::Failed(_) => ("✗", Color32::from_rgb(220, 80, 80)),
        PluginStatus::Incompatible(_) => ("⚠", Color32::from_rgb(220, 160, 60)),
        PluginStatus::Missing => ("?", Color32::from_rgb(220, 80, 80)),
    }
}

fn status_badge_color(status: &PluginStatus, ctx: &PanelContext<'_>) -> (String, Color32) {
    let (sym, color) = match status {
        PluginStatus::Loaded => ("● Loaded", Color32::from_rgb(100, 200, 100)),
        PluginStatus::Disabled => ("○ Disabled", Color32::GRAY),
        PluginStatus::Failed(_) => ("✗ Failed", Color32::from_rgb(220, 80, 80)),
        PluginStatus::Incompatible(_) => ("⚠ Incompatible", Color32::from_rgb(220, 160, 60)),
        PluginStatus::Missing => ("? Missing", Color32::from_rgb(220, 80, 80)),
    };
    let _ = ctx;
    (sym.to_string(), color)
}
