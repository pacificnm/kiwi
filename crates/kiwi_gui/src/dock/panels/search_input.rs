//! Search navigation sync and keyboard input (#192 / SPEC-007).

use egui::{Context, Key, Modifiers};
use kiwi_core::events::AppCommand;
use kiwi_core::navigation::{FocusTarget, LeftNavTab, NavCommand};
use kiwi_core::search::SearchMode;
use kiwi_core::state::AppState;

/// Collect keyboard commands when the Search dock tab is focused.
pub fn collect_search_keyboard(ctx: &Context, state: &AppState) -> Vec<AppCommand> {
    if state.palette.open || ctx.wants_keyboard_input() {
        return Vec::new();
    }

    let Some(action) = ctx.input(detect_search_key_action) else {
        return Vec::new();
    };

    match action {
        SearchKeyAction::MoveSelection(delta) => vec![AppCommand::SearchMoveSelection(delta)],
        SearchKeyAction::OpenPreview => preview_commands(state),
        SearchKeyAction::OpenEditor => open_editor_command(state).into_iter().collect(),
        SearchKeyAction::Clear => vec![AppCommand::SearchClear],
        SearchKeyAction::ToggleMode => vec![AppCommand::SearchSetMode(toggle_mode(
            state.search.mode,
        ))],
        SearchKeyAction::Refresh => {
            if state.search.query.is_empty() {
                Vec::new()
            } else {
                vec![AppCommand::SearchExecute]
            }
        }
    }
}

enum SearchKeyAction {
    MoveSelection(i32),
    OpenPreview,
    OpenEditor,
    Clear,
    ToggleMode,
    Refresh,
}

fn detect_search_key_action(input: &egui::InputState) -> Option<SearchKeyAction> {
    if input.modifiers == Modifiers::CTRL | Modifiers::SHIFT {
        return None;
    }

    if input.modifiers == Modifiers::CTRL {
        if input.key_pressed(Key::M) {
            return Some(SearchKeyAction::ToggleMode);
        }
        return None;
    }

    if input.modifiers.any() {
        return None;
    }

    if input.key_pressed(Key::ArrowDown) {
        return Some(SearchKeyAction::MoveSelection(1));
    }
    if input.key_pressed(Key::ArrowUp) {
        return Some(SearchKeyAction::MoveSelection(-1));
    }
    if input.key_pressed(Key::Enter) {
        return Some(SearchKeyAction::OpenPreview);
    }
    if input.key_pressed(Key::E) {
        return Some(SearchKeyAction::OpenEditor);
    }
    if input.key_pressed(Key::Escape) {
        return Some(SearchKeyAction::Clear);
    }
    if input.key_pressed(Key::R) || input.key_pressed(Key::F5) {
        return Some(SearchKeyAction::Refresh);
    }

    None
}

fn preview_command(state: &AppState) -> Option<AppCommand> {
    let result = state.search.results.get(state.search.selected)?;
    Some(AppCommand::PreviewFile {
        path: result.path.clone(),
        line: result.line,
    })
}

pub fn preview_commands(state: &AppState) -> Vec<AppCommand> {
    let Some(preview) = preview_command(state) else {
        return Vec::new();
    };
    vec![
        preview,
        AppCommand::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Search)),
    ]
}

fn open_editor_command(state: &AppState) -> Option<AppCommand> {
    let result = state.search.results.get(state.search.selected)?;
    Some(AppCommand::OpenEditor {
        path: result.path.clone(),
        line: result.line,
    })
}

const fn toggle_mode(mode: SearchMode) -> SearchMode {
    match mode {
        SearchMode::Files => SearchMode::Content,
        SearchMode::Content => SearchMode::Files,
    }
}

/// Global shortcut to focus the Search tab (Ctrl+Shift+F).
pub fn global_search_focus_commands() -> Vec<AppCommand> {
    vec![
        AppCommand::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Search)),
        AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Left)),
    ]
}

#[must_use]
pub fn global_search_focus_pressed(ctx: &Context) -> bool {
    ctx.input(|input| {
        input.key_pressed(Key::F) && input.modifiers.command && input.modifiers.shift
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::navigation::{FocusTarget, LeftNavTab};
    use kiwi_core::search::{SearchMode, SearchResult, SearchState};
    use kiwi_core::state::{AppState, ViewportMetrics};
    use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        )
    }

    #[test]
    fn toggle_mode_switches_files_and_content() {
        assert_eq!(toggle_mode(SearchMode::Files), SearchMode::Content);
        assert_eq!(toggle_mode(SearchMode::Content), SearchMode::Files);
    }

    #[test]
    fn preview_command_includes_content_line() {
        let mut state = test_state();
        state.search = SearchState {
            results: vec![SearchResult::content(
                PathBuf::from("/tmp/a.rs"),
                42,
                "fn main".to_string(),
            )],
            selected: 0,
            ..SearchState::default()
        };
        let cmd = preview_command(&state).expect("preview");
        assert!(matches!(
            cmd,
            AppCommand::PreviewFile { line: Some(42), .. }
        ));
    }
}
