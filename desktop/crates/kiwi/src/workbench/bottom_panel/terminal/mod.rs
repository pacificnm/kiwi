//! Integrated terminal tab — PTY shell with ANSI color rendering.

mod input;
mod pty;

use std::path::PathBuf;

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, Event, FontId, RichText, Sense, Ui};
use nest_error::NestResult;
use vt100::Color;

use crate::theme::PALETTE;
use crate::workbench::state::WorkbenchState;

use self::input::key_to_bytes;
use self::pty::PtySession;

const FONT_SIZE: f32 = 12.0;
const LINE_HEIGHT: f32 = 16.0;
const CHAR_WIDTH: f32 = 7.1;
const STATUS_ROW_HEIGHT: f32 = 18.0;
const MIN_ROWS: u16 = 4;
const MAX_ROWS: u16 = 120;
const SCROLLBACK_LINES: usize = 10_000;

/// Terminal emulator state for the bottom panel.
pub struct TerminalState {
    cwd: PathBuf,
    parser: vt100::Parser,
    session: Option<PtySession>,
    grid_cols: u16,
    grid_rows: u16,
}

impl TerminalState {
    /// Creates terminal state rooted at `cwd`.
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            cwd,
            parser: vt100::Parser::new(24, 80, SCROLLBACK_LINES),
            session: None,
            grid_cols: 80,
            grid_rows: 24,
        }
    }

    /// Updates the shell working directory for the next spawn.
    pub fn set_cwd(&mut self, cwd: PathBuf) {
        if self.cwd != cwd {
            self.cwd = cwd;
            self.session = None;
        }
    }

    /// Drains PTY output into the vt100 screen buffer. Returns true when new output arrived.
    pub fn poll(&mut self) -> bool {
        let Some(session) = self.session.as_ref() else {
            return false;
        };
        let mut updated = false;
        for chunk in session.drain_output() {
            if !chunk.is_empty() {
                self.parser.process(&chunk);
                updated = true;
            }
        }
        updated
    }

    /// Returns true when a shell session is running.
    pub fn is_active(&self) -> bool {
        self.session.is_some()
    }

    fn ensure_running(&mut self, rows: u16, cols: u16) -> NestResult<()> {
        if self.session.is_none() {
            self.grid_rows = rows;
            self.grid_cols = cols;
            self.parser = vt100::Parser::new(rows, cols, SCROLLBACK_LINES);
            self.session = Some(PtySession::spawn(&self.cwd, rows, cols)?);
            tracing::info!(
                target: "kiwi",
                cwd = %self.cwd.display(),
                rows,
                cols,
                "Terminal session started"
            );
            return Ok(());
        }

        if rows != self.grid_rows || cols != self.grid_cols {
            self.grid_rows = rows;
            self.grid_cols = cols;
            self.parser.set_size(rows, cols);
            if let Some(session) = self.session.as_ref() {
                let _ = session.resize(rows, cols);
            }
        }
        Ok(())
    }

    fn write(&mut self, bytes: &[u8]) {
        if let Some(session) = self.session.as_ref() {
            let _ = session.write(bytes);
        }
    }
}

impl std::fmt::Debug for TerminalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalState")
            .field("cwd", &self.cwd)
            .field("grid_cols", &self.grid_cols)
            .field("grid_rows", &self.grid_rows)
            .field("active", &self.session.is_some())
            .finish()
    }
}

impl Clone for TerminalState {
    fn clone(&self) -> Self {
        Self::new(self.cwd.clone())
    }
}

/// Renders the interactive terminal panel.
pub fn show(ui: &mut Ui, state: &mut WorkbenchState) {
    let cols = grid_cols(ui);
    // Reserve space for the status row before sizing the PTY grid.
    let body_top = ui.cursor().top();
    let panel_bottom = ui.max_rect().bottom();
    let body_height = (panel_bottom - body_top - STATUS_ROW_HEIGHT).max(LINE_HEIGHT);
    let rows = ((body_height / LINE_HEIGHT).floor() as u16).clamp(MIN_ROWS, MAX_ROWS);

    if let Err(error) = state.terminal.ensure_running(rows, cols) {
        ui.label(
            RichText::new(format!("Failed to start terminal: {error}"))
                .color(PALETTE.error)
                .size(12.0),
        );
        return;
    }

    state.terminal.poll();

    let screen = state.terminal.parser.screen();
    let (screen_rows, screen_cols) = screen.size();
    let focus_id = ui.id().with("terminal-surface");
    let was_focused = ui.memory(|mem| mem.has_focus(focus_id));

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{}  {}×{}",
                state.terminal.cwd.display(),
                screen_cols,
                screen_rows
            ))
            .weak()
            .size(11.0),
        );
        if !was_focused {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Click terminal to focus")
                        .weak()
                        .size(10.0),
                );
            });
        }
    });

    let terminal_height = screen_rows as f32 * LINE_HEIGHT;
    let mut focused = false;

    egui::Frame::new()
        .fill(PALETTE.background_editor)
        .inner_margin(egui::Margin::symmetric(4, 2))
        .show(ui, |ui| {
            ui.set_height(terminal_height);
            ui.set_width(ui.available_width());

            let size = egui::vec2(ui.available_width(), terminal_height);
            let rect = egui::Rect::from_min_size(ui.cursor().min, size);
            let response = ui.interact(rect, focus_id, Sense::click());
            if response.clicked() {
                response.request_focus();
            }
            focused = response.has_focus();

            let clip = response.rect;
            ui.set_clip_rect(clip);

            ui.scope_builder(egui::UiBuilder::new().max_rect(clip), |ui| {
                ui.set_width(clip.width());
                for row in 0..screen_rows {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.set_min_height(LINE_HEIGHT);
                        ui.set_max_height(LINE_HEIGHT);
                        render_row(ui, screen, row, screen_cols);
                    });
                }
            });
        });

    if focused {
        handle_input(ui, state);
    }
}

fn handle_input(ui: &mut Ui, state: &mut WorkbenchState) {
    let events: Vec<Event> = ui.input_mut(|input| {
        let mut terminal_events = Vec::new();
        input.events.retain(|event| {
            let capture = matches!(
                event,
                Event::Key { .. } | Event::Text(_) | Event::Paste(_)
            );
            if capture {
                terminal_events.push(event.clone());
            }
            !capture
        });
        terminal_events
    });

    for event in events {
        match event {
            Event::Copy => {}
            Event::Text(text) => state.terminal.write(text.as_bytes()),
            Event::Paste(text) => state.terminal.write(text.as_bytes()),
            Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => {
                if let Some(bytes) = key_to_bytes(key, modifiers) {
                    state.terminal.write(&bytes);
                }
            }
            _ => {}
        }
    }
}

fn grid_cols(ui: &Ui) -> u16 {
    ((ui.available_width() / CHAR_WIDTH).floor() as u16).clamp(20, 240)
}

fn render_row(ui: &mut Ui, screen: &vt100::Screen, row: u16, cols: u16) {
    let mut job = LayoutJob::default();
    let mut has_text = false;

    for col in 0..cols {
        let Some(cell) = screen.cell(row, col) else {
            continue;
        };
        if !cell.has_contents() {
            continue;
        }
        let text = cell.contents();
        if text.is_empty() {
            continue;
        }
        has_text = true;
        job.append(
            &text,
            0.0,
            TextFormat {
                font_id: FontId::monospace(FONT_SIZE),
                color: cell_color(cell.fgcolor(), cell.bold(), true),
                ..Default::default()
            },
        );
    }

    if has_text {
        ui.label(job);
    } else {
        ui.add_space(LINE_HEIGHT);
    }
}

fn cell_color(color: Color, bold: bool, foreground: bool) -> Color32 {
    match color {
        Color::Default => {
            if foreground {
                if bold {
                    PALETTE.text_primary
                } else {
                    PALETTE.text_primary
                }
            } else {
                PALETTE.background_editor
            }
        }
        Color::Idx(index) => ansi_index_color(index, bold),
        Color::Rgb(r, g, b) => Color32::from_rgb(r, g, b),
    }
}

fn ansi_index_color(index: u8, bold: bool) -> Color32 {
    match index {
        0 => PALETTE.text_primary,
        1 => if bold { Color32::from_rgb(255, 100, 100) } else { PALETTE.error },
        2 => if bold { Color32::from_rgb(120, 230, 140) } else { PALETTE.success },
        3 => if bold { Color32::from_rgb(255, 220, 120) } else { PALETTE.warning },
        4 => if bold { Color32::from_rgb(140, 180, 255) } else { PALETTE.info },
        5 => Color32::from_rgb(220, 140, 255),
        6 => Color32::from_rgb(120, 220, 220),
        7 => PALETTE.text_primary,
        8 => PALETTE.text_secondary,
        9 => PALETTE.error,
        10 => PALETTE.success,
        11 => PALETTE.warning,
        12 => PALETTE.info,
        13 => Color32::from_rgb(200, 120, 255),
        14 => Color32::from_rgb(100, 210, 210),
        15 => Color32::from_rgb(255, 255, 255),
        _ => PALETTE.text_primary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_state_starts_without_session() {
        let state = TerminalState::new(PathBuf::from("."));
        assert!(state.session.is_none());
    }
}
