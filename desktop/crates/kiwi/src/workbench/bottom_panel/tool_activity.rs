//! Tool Activity bottom panel — MCP agent tool invocations.

use std::time::Duration;

use egui::{Align, Color32, Frame, Layout, RichText, Ui};
use nest_agent::looks_like_schema_arguments;
use serde_json::Value;

use crate::theme::PALETTE;
use crate::workbench::state::{ToolActivityEntry, ToolActivityStatus, WorkbenchState};

/// Renders agent tool activity from the current session.
pub fn show(ui: &mut Ui, state: &mut WorkbenchState) {
    header(ui, state);
    ui.add_space(4.0);

    if state.tool_activity.is_empty() {
        empty_state(ui, state);
        return;
    }

    let running = state
        .tool_activity
        .iter()
        .any(|entry| entry.status == ToolActivityStatus::Running);

    if running {
        if let Some(step) = state.agent_step {
            ui.label(
                RichText::new(format!("Agent step {step} — waiting for tool…"))
                    .color(PALETTE.info)
                    .size(11.0),
            );
            ui.add_space(4.0);
        }
    }

    for (index, entry) in state.tool_activity.iter().enumerate() {
        render_entry(ui, index, entry);
        ui.add_space(6.0);
    }
}

fn header(ui: &mut Ui, state: &mut WorkbenchState) {
    let total = state.tool_activity.len();
    let failed = state
        .tool_activity
        .iter()
        .filter(|entry| entry.status == ToolActivityStatus::Failed)
        .count();

    ui.horizontal(|ui| {
        let summary = if total == 0 {
            "MCP tool invocations".to_string()
        } else if failed > 0 {
            format!("{total} calls · {failed} failed")
        } else {
            format!("{total} calls")
        };
        ui.label(RichText::new(summary).weak().size(11.0));

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.small_button("Clear").clicked() {
                state.tool_activity.clear();
                state.agent_step = None;
            }
        });
    });
}

fn empty_state(ui: &mut Ui, state: &WorkbenchState) {
    let hint = if state.agent_mode {
        "Enable Agent mode and send a prompt to see MCP tool calls here."
    } else {
        "No tool activity. Turn on Agent mode in the chat panel to run MCP tools."
    };
    ui.label(
        RichText::new(hint)
            .weak()
            .monospace()
            .size(12.0),
    );
}

fn render_entry(ui: &mut Ui, index: usize, entry: &ToolActivityEntry) {
    let expand_id = ui.id().with(("tool-activity-expand", index));
    let mut expanded = ui
        .data_mut(|data| data.get_temp_mut_or_default::<bool>(expand_id).to_owned());

    Frame::new()
        .fill(PALETTE.background_editor)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(status_label(entry.status))
                        .monospace()
                        .size(11.0)
                        .color(status_color(entry.status)),
                );
                ui.label(
                    RichText::new(display_tool_name(&entry.tool))
                        .strong()
                        .monospace()
                        .size(12.0)
                        .color(PALETTE.text_primary),
                );
                if let Some(step) = entry.step {
                    ui.label(
                        RichText::new(format!("step {step}"))
                            .weak()
                            .size(10.0),
                    );
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(ms) = entry.duration_ms {
                        ui.label(
                            RichText::new(format_duration_ms(ms))
                                .weak()
                                .size(10.0),
                        );
                    }
                    if ui.small_button(if expanded { "▾" } else { "▸" }).clicked() {
                        expanded = !expanded;
                    }
                });
            });

            ui.label(
                RichText::new(format!("args: {}", entry.arguments))
                    .monospace()
                    .size(11.0)
                    .color(PALETTE.text_secondary),
            );

            if expanded {
                ui.add_space(4.0);
                if let Some(result) = &entry.result {
                    section_block(ui, "Result", result, PALETTE.success);
                }
                if let Some(error) = &entry.error {
                    section_block(ui, "Error", error, PALETTE.error);
                }
            } else if let Some(error) = &entry.error {
                ui.label(
                    RichText::new(format!("error: {error}"))
                        .monospace()
                        .size(11.0)
                        .color(PALETTE.error),
                );
            } else if let Some(result) = &entry.result {
                ui.label(
                    RichText::new(format!("↳ {}", preview_line(result)))
                        .monospace()
                        .size(11.0)
                        .color(PALETTE.text_secondary),
                );
            }
        });

    ui.data_mut(|data| {
        *data.get_temp_mut_or_default::<bool>(expand_id) = expanded;
    });
}

fn section_block(ui: &mut Ui, title: &str, body: &str, color: Color32) {
    ui.label(
        RichText::new(title)
            .strong()
            .size(10.0)
            .color(color),
    );
    ui.label(
        RichText::new(body)
            .monospace()
            .size(11.0)
            .color(PALETTE.text_primary),
    );
}

fn status_label(status: ToolActivityStatus) -> &'static str {
    match status {
        ToolActivityStatus::Running => "RUN  ",
        ToolActivityStatus::Success => "OK   ",
        ToolActivityStatus::Failed => "FAIL ",
    }
}

fn status_color(status: ToolActivityStatus) -> Color32 {
    match status {
        ToolActivityStatus::Running => PALETTE.info,
        ToolActivityStatus::Success => PALETTE.success,
        ToolActivityStatus::Failed => PALETTE.error,
    }
}

/// Formats a model-visible MCP tool name for display.
pub fn display_tool_name(tool: &str) -> String {
    if let Some((server, name)) = tool.split_once("__") {
        format!("{server}/{name}")
    } else {
        tool.to_string()
    }
}

/// Formats tool arguments for display in chat and the activity panel.
pub fn format_tool_arguments(arguments: &Value) -> String {
    if looks_like_schema_arguments(arguments) {
        return "(invalid — model returned JSON Schema)".into();
    }

    let Some(object) = arguments.as_object() else {
        return arguments.to_string();
    };

    if object.is_empty() {
        return "(none)".into();
    }

    object
        .iter()
        .map(|(key, value)| format!("{key}={}", summarize_json_value(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Builds a one-line chat summary for a tool invocation.
pub fn format_tool_call_summary(tool: &str, arguments: &Value) -> String {
    format!("{}({})", display_tool_name(tool), format_tool_arguments(arguments))
}

/// Short tool-result line for the chat panel (full text lives in Tool Activity).
pub fn format_tool_result_chat_preview(result: &str) -> String {
    const LIMIT: usize = 160;
    let header = result.lines().next().unwrap_or(result).trim();
    if header.is_empty() {
        return "OK (see Tool Activity for details)".into();
    }
    let mut preview = if header.len() <= LIMIT {
        header.to_string()
    } else {
        format!("{}…", &header[..LIMIT])
    };
    if result.lines().count() > 1 {
        preview.push_str(" — see Tool Activity for full results");
    }
    preview
}

/// Creates a new running tool activity entry.
pub fn new_running_entry(tool: String, arguments: &Value, step: Option<u32>) -> ToolActivityEntry {
    ToolActivityEntry {
        tool,
        arguments: format_tool_arguments(arguments),
        result: None,
        error: None,
        status: ToolActivityStatus::Running,
        duration_ms: None,
        step,
    }
}

pub fn duration_to_ms(duration: Duration) -> u64 {
    duration.as_millis().min(u64::MAX as u128) as u64
}

fn summarize_json_value(value: &Value) -> String {
    match value {
        Value::String(text) => {
            if text.len() > 64 {
                format!("\"{}…\"", &text[..61])
            } else {
                format!("\"{text}\"")
            }
        }
        other => other.to_string(),
    }
}

fn preview_line(text: &str) -> String {
    const LIMIT: usize = 120;
    let line = text.lines().next().unwrap_or(text);
    if line.len() <= LIMIT {
        line.to_string()
    } else {
        format!("{}…", &line[..LIMIT])
    }
}

fn format_duration_ms(ms: u64) -> String {
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{ms}ms")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn display_name_splits_server_prefix() {
        assert_eq!(
            display_tool_name("nest_memory__search_project_memory"),
            "nest_memory/search_project_memory"
        );
    }

    #[test]
    fn formats_object_arguments() {
        let formatted = format_tool_arguments(&json!({"query": "nest-core", "limit": 3}));
        assert!(formatted.contains("query=\"nest-core\""));
        assert!(formatted.contains("limit=3"));
    }
}
