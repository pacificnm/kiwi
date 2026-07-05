//! Logs bottom panel.

use egui::{Align, Color32, Layout, RichText, Ui};
use nest_logging::{ui_buffer, LogLevel, LogRecord};

use crate::theme::PALETTE;

/// Renders captured application logs with level-based colors.
pub fn show(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Application logs").weak().size(11.0));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.small_button("Clear").clicked() {
                if let Some(buffer) = ui_buffer() {
                    buffer.clear();
                }
            }
        });
    });
    ui.add_space(4.0);

    let Some(buffer) = ui_buffer() else {
        ui.label(
            RichText::new("Log capture is not configured.")
                .weak()
                .size(12.0),
        );
        return;
    };

    let lines = buffer.snapshot();
    if lines.is_empty() {
        ui.label(
            RichText::new("No log entries yet.")
                .weak()
                .monospace()
                .size(12.0),
        );
        return;
    }

    for line in lines {
        log_line(ui, &line);
    }
}

fn log_line(ui: &mut Ui, record: &LogRecord) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(
            RichText::new(format!("[{}] ", record.timestamp))
                .monospace()
                .size(12.0)
                .color(PALETTE.text_muted),
        );
        ui.label(
            RichText::new(format!("{:5} ", level_tag(record.level)))
                .monospace()
                .size(12.0)
                .color(level_color(record.level)),
        );
        ui.label(
            RichText::new(format!("{}: ", record.target))
                .monospace()
                .size(12.0)
                .color(PALETTE.text_secondary),
        );
        ui.label(
            RichText::new(&record.message)
                .monospace()
                .size(12.0)
                .color(message_color(record.level)),
        );
    });
}

fn level_tag(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "TRACE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
    }
}

fn level_color(level: LogLevel) -> Color32 {
    match level {
        LogLevel::Trace => PALETTE.text_disabled,
        LogLevel::Debug => PALETTE.info,
        LogLevel::Info => PALETTE.success,
        LogLevel::Warn => PALETTE.warning,
        LogLevel::Error => PALETTE.error,
    }
}

fn message_color(level: LogLevel) -> Color32 {
    match level {
        LogLevel::Error => PALETTE.error,
        LogLevel::Warn => PALETTE.warning,
        LogLevel::Trace | LogLevel::Debug => PALETTE.text_secondary,
        LogLevel::Info => PALETTE.text_primary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_tags_are_fixed_width() {
        assert_eq!(level_tag(LogLevel::Info), "INFO");
        assert_eq!(level_tag(LogLevel::Error), "ERROR");
    }
}

