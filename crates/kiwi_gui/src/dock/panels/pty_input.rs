//! PTY keyboard, paste, and scroll input for Terminal and Agent dock tabs (SPEC-010 / SPEC-011).

use std::time::{Duration, Instant};

use egui::{Context, Event, Key, Modifiers, Sense, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::navigation::{FocusTarget, MainTab, NavCommand};
use kiwi_core::shell::pty_paste_bytes;
use kiwi_core::state::AppState;

use crate::dock::tab::KiwiTab;

const SHELL_FORCE_QUIT_WINDOW: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyTarget {
    Shell,
    Agent,
}

pub struct PtyInputOutcome {
    pub commands: Vec<AppCommand>,
    pub copy_to_clipboard: Option<String>,
}

/// Sync TUI navigation state when a PTY dock tab is focused (lazy agent spawn, paste routing).
pub fn navigation_sync_commands(state: &AppState, tab: KiwiTab) -> Vec<AppCommand> {
    match tab {
        KiwiTab::Terminal if state.navigation.focus != FocusTarget::Shell => {
            vec![AppCommand::Navigation(NavCommand::SetFocus(
                FocusTarget::Shell,
            ))]
        }
        KiwiTab::Agent => {
            let mut commands = Vec::new();
            if state.navigation.main_tab != MainTab::Agent {
                commands.push(AppCommand::Navigation(NavCommand::SelectMainTabUnpaired(
                    MainTab::Agent,
                )));
            }
            if state.navigation.focus != FocusTarget::Main {
                commands.push(AppCommand::Navigation(NavCommand::SetFocus(
                    FocusTarget::Main,
                )));
            }
            commands
        }
        _ => Vec::new(),
    }
}

/// Capture click-to-focus for a PTY panel covering the tab viewport.
///
/// Returns `(has_keyboard_focus, clicked_this_frame)`.
#[must_use]
pub fn capture_pty_keyboard_focus(ui: &mut Ui, surface_id: &str) -> (bool, bool) {
    let rect = ui.clip_rect();
    let response = ui.interact(
        rect,
        ui.id().with(surface_id),
        Sense::click(),
    );
    if response.clicked() {
        response.request_focus();
    }
    (response.has_focus(), response.clicked())
}

/// Collect PTY input events for the focused dock tab.
pub fn collect_pty_input(
    ctx: &Context,
    state: &AppState,
    target: PtyTarget,
    last_shell_interrupt: &mut Option<Instant>,
    accept_keyboard: bool,
) -> PtyInputOutcome {
    let mut outcome = PtyInputOutcome {
        commands: Vec::new(),
        copy_to_clipboard: None,
    };

    if state.palette.open || !pty_running(state, target) {
        return outcome;
    }
    if !accept_keyboard && ctx.wants_keyboard_input() {
        return outcome;
    }

    let events: Vec<Event> = ctx.input(|input| input.events.clone());

    for event in events {
        match event {
            Event::Paste(text) if !text.is_empty() => {
                outcome
                    .commands
                    .push(write_command(target, pty_paste_bytes(&text)));
            }
            Event::Text(text) if !text.is_empty() => {
                push_write(
                    &mut outcome.commands,
                    target,
                    text.into_bytes(),
                    last_shell_interrupt,
                );
            }
            Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } => {
                if let Some(cmd) = key_to_command(key, modifiers, target, state, last_shell_interrupt)
                {
                    match cmd {
                        PtyKeyAction::Write(bytes) => {
                            push_write(&mut outcome.commands, target, bytes, last_shell_interrupt);
                        }
                        PtyKeyAction::Scroll(delta) => {
                            outcome.commands.push(scroll_command(target, delta));
                        }
                        PtyKeyAction::Restart => outcome.commands.push(AppCommand::AgentRestart),
                        PtyKeyAction::Copy(text) => outcome.copy_to_clipboard = Some(text),
                    }
                }
            }
            _ => {}
        }
    }

    outcome
}

enum PtyKeyAction {
    Write(Vec<u8>),
    Scroll(i32),
    Restart,
    Copy(String),
}

fn pty_running(state: &AppState, target: PtyTarget) -> bool {
    match target {
        PtyTarget::Shell => state.shell.running,
        PtyTarget::Agent => state.active_agent().running,
    }
}

fn write_command(target: PtyTarget, bytes: Vec<u8>) -> AppCommand {
    match target {
        PtyTarget::Shell => AppCommand::ShellWrite(bytes),
        PtyTarget::Agent => AppCommand::AgentWrite(bytes),
    }
}

fn push_write(
    commands: &mut Vec<AppCommand>,
    target: PtyTarget,
    bytes: Vec<u8>,
    last_shell_interrupt: &mut Option<Instant>,
) {
    if target == PtyTarget::Shell {
        if let Some(cmd) = shell_write_with_interrupt(bytes, last_shell_interrupt) {
            commands.push(cmd);
        }
    } else {
        commands.push(AppCommand::AgentWrite(bytes));
    }
}

fn shell_write_with_interrupt(
    bytes: Vec<u8>,
    last_shell_interrupt: &mut Option<Instant>,
) -> Option<AppCommand> {
    if bytes == [0x03] {
        let now = Instant::now();
        if last_shell_interrupt
            .is_some_and(|earlier| now.duration_since(earlier) <= SHELL_FORCE_QUIT_WINDOW)
        {
            return Some(AppCommand::Quit);
        }
        *last_shell_interrupt = Some(now);
    } else {
        *last_shell_interrupt = None;
    }
    Some(AppCommand::ShellWrite(bytes))
}

fn key_to_command(
    key: Key,
    modifiers: Modifiers,
    target: PtyTarget,
    state: &AppState,
    last_shell_interrupt: &mut Option<Instant>,
) -> Option<PtyKeyAction> {
    if modifiers.ctrl {
        return ctrl_key_command(key, modifiers, target, state, last_shell_interrupt);
    }

    if modifiers.any() && !modifiers.shift {
        return None;
    }

    match key {
        Key::PageUp => Some(PtyKeyAction::Scroll(-1)),
        Key::PageDown => Some(PtyKeyAction::Scroll(1)),
        _ => encode_egui_key(key).map(PtyKeyAction::Write),
    }
}

fn scroll_command(target: PtyTarget, delta: i32) -> AppCommand {
    match target {
        PtyTarget::Shell => AppCommand::ShellScroll(delta),
        PtyTarget::Agent => AppCommand::AgentScroll(delta),
    }
}

fn ctrl_key_command(
    key: Key,
    modifiers: Modifiers,
    target: PtyTarget,
    state: &AppState,
    _last_shell_interrupt: &mut Option<Instant>,
) -> Option<PtyKeyAction> {
    if modifiers.ctrl && modifiers.shift && key == Key::R && target == PtyTarget::Agent {
        return Some(PtyKeyAction::Restart);
    }

    if !modifiers.ctrl {
        return None;
    }

    match key {
        Key::C if target == PtyTarget::Shell => Some(PtyKeyAction::Write(vec![0x03])),
        Key::C if target == PtyTarget::Agent => copy_agent_scrollback(state).map(PtyKeyAction::Copy),
        Key::V => None,
        _ => encode_control_key(key).map(PtyKeyAction::Write),
    }
}

fn copy_agent_scrollback(state: &AppState) -> Option<String> {
    let agent = state.active_agent();
    let visible = usize::from(state.viewport.agent_rows.max(1));
    let width = usize::from(state.viewport.agent_cols.max(80));
    let start = agent
        .scrollback
        .viewport_start(visible, agent.follow_tail, agent.viewport_offset);
    let lines = agent
        .scrollback
        .viewport_lines(start, visible, width, agent.follow_tail);
    if lines.is_empty() {
        return None;
    }
    Some(
        lines
            .into_iter()
            .map(|line| kiwi_core::ansi::strip_ansi(&line))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

#[must_use]
pub fn encode_egui_key(key: Key) -> Option<Vec<u8>> {
    match key {
        Key::Enter => Some(b"\r".to_vec()),
        Key::Backspace => Some(vec![0x7f]),
        Key::Tab => Some(b"\t".to_vec()),
        Key::Escape => Some(vec![0x1b]),
        Key::ArrowUp => Some(b"\x1b[A".to_vec()),
        Key::ArrowDown => Some(b"\x1b[B".to_vec()),
        Key::ArrowRight => Some(b"\x1b[C".to_vec()),
        Key::ArrowLeft => Some(b"\x1b[D".to_vec()),
        Key::Home => Some(b"\x1b[H".to_vec()),
        Key::End => Some(b"\x1b[F".to_vec()),
        Key::Delete => Some(b"\x1b[3~".to_vec()),
        Key::Insert => Some(b"\x1b[2~".to_vec()),
        Key::F1 => Some(b"\x1bOP".to_vec()),
        Key::F2 => Some(b"\x1bOQ".to_vec()),
        Key::F3 => Some(b"\x1bOR".to_vec()),
        Key::F4 => Some(b"\x1bOS".to_vec()),
        Key::F5 => Some(b"\x1b[15~".to_vec()),
        Key::F6 => Some(b"\x1b[17~".to_vec()),
        Key::F7 => Some(b"\x1b[18~".to_vec()),
        Key::F8 => Some(b"\x1b[19~".to_vec()),
        Key::F9 => Some(b"\x1b[20~".to_vec()),
        Key::F10 => Some(b"\x1b[21~".to_vec()),
        Key::F11 => Some(b"\x1b[23~".to_vec()),
        Key::F12 => Some(b"\x1b[24~".to_vec()),
        _ => None,
    }
}

fn encode_control_key(key: Key) -> Option<Vec<u8>> {
    match key {
        Key::C => Some(vec![0x03]),
        Key::D => Some(vec![0x04]),
        Key::Z => Some(vec![0x1a]),
        Key::A => Some(vec![0x01]),
        Key::B => Some(vec![0x02]),
        Key::E => Some(vec![0x05]),
        Key::F => Some(vec![0x06]),
        Key::K => Some(vec![0x0b]),
        Key::L => Some(vec![0x0c]),
        Key::N => Some(vec![0x0e]),
        Key::P => Some(vec![0x10]),
        Key::R => Some(vec![0x12]),
        Key::T => Some(vec![0x14]),
        Key::U => Some(vec![0x15]),
        Key::W => Some(vec![0x17]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_enter_and_backspace() {
        assert_eq!(encode_egui_key(Key::Enter), Some(b"\r".to_vec()));
        assert_eq!(encode_egui_key(Key::Backspace), Some(vec![0x7f]));
    }

    #[test]
    fn encodes_control_c_as_interrupt() {
        assert_eq!(encode_control_key(Key::C), Some(vec![0x03]));
    }

    #[test]
    fn encodes_arrow_keys() {
        assert_eq!(encode_egui_key(Key::ArrowUp), Some(b"\x1b[A".to_vec()));
    }

    #[test]
    fn navigation_sync_for_agent_tab() {
        use kiwi_core::config::ResolvedConfig;
        use kiwi_core::navigation::FocusTarget;
        use kiwi_core::state::ViewportMetrics;
        use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};
        use std::path::PathBuf;

        let mut state = AppState::from_startup(
            PathBuf::from("/tmp"),
            false,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        );
        state.navigation.main_tab = MainTab::Logs;
        state.navigation.focus = FocusTarget::Shell;
        let cmds = navigation_sync_commands(&state, KiwiTab::Agent);
        assert_eq!(cmds.len(), 2);
    }
}
