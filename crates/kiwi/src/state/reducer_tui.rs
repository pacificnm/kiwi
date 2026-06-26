use crate::clipboard::{pty_paste_bytes, resolve_copy_text, resolve_paste_target, PasteTarget};
use crate::commands::refresh_matches;
use crate::layout::compute_layout;
use crate::selection::{SelectionPane, TextPosition};
use crate::state::{AppCommand, AppEvent, AppState, SideEffect};
use crate::theme::ThemePalette;

use kiwi_core::navigation::{FocusTarget, LeftNavTab};
use kiwi_core::reducer as core;

pub fn reduce_terminal_resize(state: &mut AppState, width: u16, height: u16) -> Vec<SideEffect> {
    let Ok(layout) = compute_layout(width, height, state.config.app.left_width) else {
        return Vec::new();
    };

    if state.layout == layout {
        return Vec::new();
    }

    state.layout = layout;
    state.sync_viewport_from_layout();
    state.dirty = true;

    if !state.shell.running {
        return Vec::new();
    }

    let cols = state.viewport.shell_cols;
    let rows = state.viewport.shell_rows;
    if cols == state.shell.cols && rows == state.shell.rows {
        return Vec::new();
    }

    state.shell.apply_resize(cols, rows);
    vec![SideEffect::ResizeShell { cols, rows }]
}

pub fn reduce_clipboard_copy(state: &mut AppState) -> Vec<SideEffect> {
    let Some(text) = resolve_copy_text(state) else {
        state.notifications.show_toast("Nothing to copy");
        state.dirty = true;
        return Vec::new();
    };

    state.dirty = true;
    vec![SideEffect::CopyToClipboard(text)]
}

pub fn reduce_clipboard_cut(state: &mut AppState) -> Vec<SideEffect> {
    let Some(text) = resolve_copy_text(state) else {
        state.notifications.show_toast("Nothing to cut");
        state.dirty = true;
        return Vec::new();
    };

    apply_cut_mutation(state);
    state.dirty = true;
    vec![SideEffect::CopyToClipboard(text)]
}

pub fn reduce_clipboard_paste(state: &mut AppState) -> Vec<SideEffect> {
    state.dirty = true;
    vec![SideEffect::PasteFromClipboard]
}

pub fn reduce_paste_text(state: &mut AppState, text: String) -> Vec<SideEffect> {
    if text.is_empty() {
        return Vec::new();
    }

    state.dirty = true;
    match resolve_paste_target(state) {
        PasteTarget::PaletteInput => {
            state.palette.history_cursor = None;
            state.palette.input.push_str(&text);
            refresh_matches(state);
            Vec::new()
        }
        PasteTarget::SearchQuery => {
            let mut query = state.search.query.clone();
            query.push_str(&text);
            core::reduce_command(&mut state.reduce_view(), AppCommand::SearchSetQuery(query))
        }
        PasteTarget::ShellPty => vec![SideEffect::WriteShell(pty_paste_bytes(&text))],
        PasteTarget::AgentPty => vec![SideEffect::WriteAgent(pty_paste_bytes(&text))],
        PasteTarget::Unsupported => {
            state
                .notifications
                .show_toast("Paste is not supported in this pane");
            Vec::new()
        }
    }
}

pub fn reduce_selection_begin(
    state: &mut AppState,
    pane: SelectionPane,
    line: usize,
    col: usize,
) -> Vec<SideEffect> {
    state.text_selection.begin(pane, TextPosition { line, col });
    state.dirty = true;
    Vec::new()
}

pub fn reduce_selection_extend(state: &mut AppState, line: usize, col: usize) -> Vec<SideEffect> {
    if state.text_selection.dragging {
        state.text_selection.extend(TextPosition { line, col });
        state.dirty = true;
    }
    Vec::new()
}

pub fn reduce_selection_end(state: &mut AppState) -> Vec<SideEffect> {
    state.text_selection.end_drag();
    state.dirty = true;
    Vec::new()
}

pub fn reduce_selection_clear(state: &mut AppState) -> Vec<SideEffect> {
    if state.text_selection.pane.is_some() {
        state.text_selection.clear();
        state.dirty = true;
    }
    Vec::new()
}

pub fn sync_ui_theme(state: &mut AppState) {
    state.theme = ThemePalette::from_core(state.core_theme.clone());
}

pub fn post_reduce(state: &mut AppState, event: &AppEvent) {
    match event {
        AppEvent::GitHubIssueDetailLoaded { .. } if state.github.issue_detail.is_some() => {
            state.text_selection.clear();
        }
        AppEvent::GitHubPrDetailLoaded { .. } if state.github.pr_detail.is_some() => {
            state.text_selection.clear();
        }
        _ => {}
    }
}

fn apply_cut_mutation(state: &mut AppState) {
    if state.palette.open {
        state.palette.input.clear();
        refresh_matches(state);
        return;
    }

    if state.navigation.focus == FocusTarget::Left
        && state.navigation.left_tab == LeftNavTab::Search
        && state.search.results.is_empty()
    {
        state.search.query.clear();
    }
}
