#[path = "reducer_tui.rs"]
mod reducer_tui;

use kiwi_core::reducer as core;

use crate::state::{AppCommand, AppEvent, AppState, SideEffect};

pub fn reduce(state: &mut AppState, event: AppEvent) -> Vec<SideEffect> {
    match event {
        AppEvent::TerminalResize { width, height } => {
            reducer_tui::reduce_terminal_resize(state, width, height)
        }
        AppEvent::Command(command) => reduce_command(state, command),
        other => {
            let effects = core::reduce(&mut state.reduce_view(), other.clone());
            reducer_tui::post_reduce(state, &other);
            effects
        }
    }
}

fn reduce_command(state: &mut AppState, command: AppCommand) -> Vec<SideEffect> {
    match command {
        AppCommand::ClipboardCopy => reducer_tui::reduce_clipboard_copy(state),
        AppCommand::ClipboardCut => reducer_tui::reduce_clipboard_cut(state),
        AppCommand::ClipboardPaste => reducer_tui::reduce_clipboard_paste(state),
        AppCommand::PasteText(text) => reducer_tui::reduce_paste_text(state, text),
        AppCommand::SelectionBegin { pane, line, col } => {
            reducer_tui::reduce_selection_begin(state, pane, line, col)
        }
        AppCommand::SelectionExtend { line, col } => {
            reducer_tui::reduce_selection_extend(state, line, col)
        }
        AppCommand::SelectionEnd => reducer_tui::reduce_selection_end(state),
        AppCommand::SelectionClear => reducer_tui::reduce_selection_clear(state),
        other => {
            let needs_theme = matches!(
                other,
                AppCommand::SetTheme(_) | AppCommand::SettingsApplyTheme
            );
            let effects = core::reduce_command(&mut state.reduce_view(), other);
            if needs_theme {
                reducer_tui::sync_ui_theme(state);
            }
            effects
        }
    }
}

pub fn agent_spawn_effects_if_needed(state: &mut AppState) -> Vec<SideEffect> {
    core::agent_spawn_effects_if_needed(&mut state.reduce_view())
}

pub fn file_tree_startup_effects(state: &mut AppState) -> Vec<SideEffect> {
    core::file_tree_startup_effects(&mut state.reduce_view())
}

pub fn workspace_restore_effects(state: &mut AppState) -> Vec<SideEffect> {
    core::workspace_restore_effects(&mut state.reduce_view())
}
