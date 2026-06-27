use std::path::PathBuf;
use std::time::SystemTime;

use crate::agent::infer_status_from_scrollback;
use crate::commands::{execute_command, history_input_for_id, refresh_matches};
use crate::config::{project_has_theme_override, ThemeSettings};
use crate::file_tree::ExpandAction;
use crate::git::{
    adjacent_changed_file, branch_move_selection, branch_select_row, branch_selected_name,
    build_panel_rows, changed_file_paths, clamp_git_scroll, ensure_branch_selection,
    ensure_git_selection, git_move_selection, git_select_row, patch_git_file_entries,
    row_for_path, BranchEntry, GitFileEntry,
};
use crate::github::{
    advance_pr_create_prompt, apply_label_picker_load, ensure_issue_selection, ensure_pr_selection,
    format_issue_agent_prompt, format_pr_agent_prompt, issue_body_excerpt_from_detail, issue_move_selection, issue_select_row,
    missing_browser_target_message, page_scroll_issue_detail, pr_move_selection, pr_select_row,
    pull_request_is_mergeable, resolve_browser_target, scroll_issue_detail, selected_pull_request,
    GhContextMenuAction, GhContextMenuState, GhContextTarget, GitHubLeftPane,
    IssueDetailLoadResult, IssueListLoadResult, LabelPickerState, PrCreatePromptAdvance,
    PrDetailLoadResult, PrListLoadResult, ISSUE_LIST_CACHE_SECS, PR_LIST_CACHE_SECS,
};
use crate::navigation::{FocusTarget, LeftNavTab, MainTab, NavCommand};
use crate::settings::{ensure_settings_selection, settings_move_selection, settings_select_row};
use crate::state::{PalettePrompt, ReduceView};
use crate::theme::load_theme_with_capabilities;

use crate::events::{AppCommand, AppEvent, FsEffect, SideEffect};

pub(super) fn reduce_open_editor(
    state: &mut ReduceView<'_>,
    path: PathBuf,
    line: Option<u32>,
) -> Vec<SideEffect> {
    state.set_dirty();
    vec![SideEffect::Fs(FsEffect::LaunchEditor { path, line })]
}

pub(super) fn reduce_editor_launched(
    state: &mut ReduceView<'_>,
    path: PathBuf,
    command: String,
) -> Vec<SideEffect> {
    state.logs.push_info(format!(
        "Launched editor `{command}` for {}",
        path.display()
    ));
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_editor_launch_failed(
    state: &mut ReduceView<'_>,
    path: PathBuf,
    error: String,
    show_modal: bool,
) -> Vec<SideEffect> {
    state.logs.push_error(format!(
        "Editor launch failed for {}: {error}",
        path.display()
    ));
    if show_modal {
        state.notifications.show_modal(
            "Editor command not found",
            format!("{error}\n\nPress Esc to dismiss."),
        );
    } else {
        state.notifications.show_toast(error);
    }
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_settings_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    let viewport_rows = state.viewport.settings_rows;
    settings_move_selection(state.settings, delta, viewport_rows);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_settings_select(state: &mut ReduceView<'_>, index: usize) -> Vec<SideEffect> {
    let viewport_rows = state.viewport.settings_rows;
    settings_select_row(state.settings, index, viewport_rows);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_settings_apply_theme(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    let Some(name) = crate::theme::BUILTIN_THEME_NAMES
        .get(state.settings.selected_index)
        .copied()
    else {
        return Vec::new();
    };

    reduce_set_theme(state, name.to_string())
}

pub(super) fn reduce_set_theme(state: &mut ReduceView<'_>, name: String) -> Vec<SideEffect> {
    if name == state.config.theme.name && state.config.theme.custom.is_none() {
        return Vec::new();
    }

    let settings = ThemeSettings {
        name: name.clone(),
        custom: None,
    };

    let palette = match load_theme_with_capabilities(&settings, *state.terminal_capabilities) {
        Ok(palette) => palette,
        Err(err) => {
            state
                .notifications
                .show_toast(format!("Failed to load theme: {err}"));
            return Vec::new();
        }
    };

    *state.theme = palette;
    state.config.theme = settings;
    state.set_dirty();
    let viewport_rows = state.viewport.settings_rows;
    ensure_settings_selection(state.settings, &name, viewport_rows);

    let mut message = format!("Theme set to {name}");
    if project_has_theme_override(state.repo_root) {
        message.push_str("; saved to user config (project .kiwi.toml may override on restart)");
    } else {
        message.push_str("; saved to user config");
    }
    state.notifications.show_toast(message);

    vec![SideEffect::PersistUserTheme { name }]
}
