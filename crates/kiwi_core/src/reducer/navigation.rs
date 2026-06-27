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

use crate::events::{AppCommand, AppEvent, GitHubEffect, SideEffect};

pub fn apply_navigation(state: &mut ReduceView<'_>, command: NavCommand) {
    let before = state.navigation.clone();
    state.navigation.apply(command);
    if *state.navigation != before {
        state.set_dirty();
    }
    if state.navigation.left_tab == LeftNavTab::Gh
        || matches!(state.navigation.main_tab, MainTab::Issues | MainTab::Prs)
    {
        sync_github_left_pane_from_main_tab(state);
    }
    if state.navigation.left_tab == LeftNavTab::Files {
        state.file_tree.ensure_selection();
    }
    if state.navigation.left_tab == LeftNavTab::Git {
        let git_rows = build_panel_rows(&state.git.file_entries, state.config.git.show_untracked);
        ensure_git_selection(state.git, &git_rows);
    }
    if state.navigation.main_tab == MainTab::Branches {
        ensure_branch_selection(state.branches);
    }
    if state.navigation.main_tab == MainTab::Settings && before.main_tab != MainTab::Settings {
        let viewport_rows = state.viewport.settings_rows;
        ensure_settings_selection(state.settings, &state.config.theme.name, viewport_rows);
    }
}

pub(super) fn sync_github_left_pane_from_main_tab(state: &mut ReduceView<'_>) {
    state.github.left_pane = match state.navigation.main_tab {
        MainTab::Issues => GitHubLeftPane::Issues,
        MainTab::Prs => GitHubLeftPane::Prs,
        _ => state.github.left_pane,
    };
}

pub(super) fn reduce_palette_open(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.palette.open_with_focus(state.navigation.focus);
    state.navigation.focus = FocusTarget::CommandPalette;
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_palette_close(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.navigation.focus = state.palette.close();
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_palette_append_char(state: &mut ReduceView<'_>, ch: char) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_cursor = None;
    state.palette.input.push(ch);
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_palette_backspace(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    state.palette.history_cursor = None;
    state.palette.input.pop();
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_palette_move_selection(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if !state.palette.open || state.palette.prompt.is_some() {
        return Vec::new();
    }

    state.palette.move_selection(delta as isize);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_palette_history_up(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    if let Some(command_id) = state.palette.history_up() {
        state.palette.input = history_input_for_id(state, &command_id);
    }
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_palette_history_down(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    match state.palette.history_down() {
        None => {}
        Some(None) => state.palette.input.clear(),
        Some(Some(command_id)) => state.palette.input = history_input_for_id(state, &command_id),
    }
    refresh_matches(state);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_palette_execute_selected(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.palette.open {
        return Vec::new();
    }

    if let Some(prompt) = state.palette.prompt.clone() {
        return reduce_palette_prompt_submit(state, prompt);
    }

    let Some(registry_index) = state.palette.matches.get(state.palette.selected).copied() else {
        return Vec::new();
    };

    execute_command(state, registry_index)
}

pub(super) fn reduce_palette_prompt_submit(
    state: &mut ReduceView<'_>,
    prompt: PalettePrompt,
) -> Vec<SideEffect> {
    match prompt {
        PalettePrompt::GitHubIssueComment { number } => {
            let body = state.palette.input.trim().to_string();
            if body.is_empty() {
                state.notifications.show_toast("Comment cannot be empty");
                state.set_dirty();
                return Vec::new();
            }

            state.navigation.focus = state.palette.close();
            state.set_dirty();
            vec![SideEffect::GitHub(GitHubEffect::SpawnIssueComment { number, body })]
        }
        PalettePrompt::GitHubPrCreate(prompt) => {
            let input = state.palette.input.clone();
            match advance_pr_create_prompt(prompt, &input) {
                Ok(PrCreatePromptAdvance::Continue(next)) => {
                    state.palette.input.clear();
                    state.palette.prompt = Some(PalettePrompt::GitHubPrCreate(next));
                    state.set_dirty();
                    Vec::new()
                }
                Ok(PrCreatePromptAdvance::Submit(request)) => {
                    state.navigation.focus = state.palette.close();
                    state.set_dirty();
                    vec![SideEffect::GitHub(GitHubEffect::SpawnPrCreate { request })]
                }
                Err(message) => {
                    state.notifications.show_toast(message);
                    state.set_dirty();
                    Vec::new()
                }
            }
        }
    }
}

pub(super) fn reduce_palette_execute_match(state: &mut ReduceView<'_>, match_index: usize) -> Vec<SideEffect> {
    if !state.palette.open || state.palette.prompt.is_some() {
        return Vec::new();
    }

    let Some(registry_index) = state.palette.matches.get(match_index).copied() else {
        return Vec::new();
    };

    execute_command(state, registry_index)
}

pub(super) fn reduce_modal_dismiss(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.notifications.modal.is_some() {
        state.notifications.dismiss_modal();
        state.set_dirty();
    }
    Vec::new()
}
