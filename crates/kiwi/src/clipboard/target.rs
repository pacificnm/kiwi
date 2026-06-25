use crate::ansi::strip_ansi;
use crate::layout::FocusTarget;
use crate::navigation::{LeftNavTab, MainTab};
use crate::shell::ScrollbackBuffer;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteTarget {
    PaletteInput,
    SearchQuery,
    ShellPty,
    AgentPty,
    Unsupported,
}

pub fn resolve_copy_text(state: &AppState) -> Option<String> {
    if let Some(text) = state.text_selection.extract_text(state) {
        if !text.is_empty() {
            return Some(text);
        }
    }

    if state.palette.open {
        if !state.palette.input.is_empty() {
            return Some(state.palette.input.clone());
        }
        return resolve_copy_text_for_focus(state, state.palette.focus_before_open);
    }

    resolve_copy_text_for_focus(state, state.navigation.focus)
}

pub fn resolve_copy_text_for_focus(state: &AppState, focus: FocusTarget) -> Option<String> {
    match focus {
        FocusTarget::CommandPalette => copy_palette_input(state),
        FocusTarget::Left => copy_left_pane(state),
        FocusTarget::Main => copy_main_pane(state),
        FocusTarget::Shell => copy_scrollback_pane(
            &state.shell.scrollback,
            state.shell.follow_tail,
            state.shell.viewport_offset,
            shell_visible_rows(state),
            shell_visible_cols(state),
        ),
    }
}

pub fn resolve_paste_target(state: &AppState) -> PasteTarget {
    if state.palette.open || state.navigation.focus == FocusTarget::CommandPalette {
        return PasteTarget::PaletteInput;
    }

    if state.navigation.focus == FocusTarget::Left
        && state.navigation.left_tab == LeftNavTab::Search
    {
        return PasteTarget::SearchQuery;
    }

    if state.navigation.focus == FocusTarget::Shell && state.shell.running {
        return PasteTarget::ShellPty;
    }

    if state.navigation.focus == FocusTarget::Main
        && state.navigation.main_tab == MainTab::Agent
        && state.agent.running
    {
        return PasteTarget::AgentPty;
    }

    PasteTarget::Unsupported
}

fn copy_palette_input(state: &AppState) -> Option<String> {
    if state.palette.input.is_empty() {
        None
    } else {
        Some(state.palette.input.clone())
    }
}

fn copy_left_pane(state: &AppState) -> Option<String> {
    match state.navigation.left_tab {
        LeftNavTab::Files => state
            .file_tree
            .selected
            .as_ref()
            .map(|path| path.display().to_string()),
        LeftNavTab::Search => copy_search_selection(state).or_else(|| copy_search_query(state)),
        _ => None,
    }
}

fn copy_main_pane(state: &AppState) -> Option<String> {
    match state.navigation.main_tab {
        MainTab::Preview => copy_preview_line(state),
        MainTab::Agent => copy_scrollback_pane(
            &state.agent.scrollback,
            state.agent.follow_tail,
            state.agent.viewport_offset,
            agent_visible_rows(state),
            agent_visible_cols(state),
        ),
        MainTab::Logs => copy_logs(state),
        _ => None,
    }
}

fn copy_preview_line(state: &AppState) -> Option<String> {
    let line_index = state.preview.scroll_offset;
    state.preview.lines.get(line_index).cloned()
}

fn copy_search_query(state: &AppState) -> Option<String> {
    if state.search.query.is_empty() {
        None
    } else {
        Some(state.search.query.clone())
    }
}

fn copy_search_selection(state: &AppState) -> Option<String> {
    let result = state.search.results.get(state.search.selected)?;
    let mut text = result.path.display().to_string();
    if let Some(line) = result.line {
        text.push(':');
        text.push_str(&line.to_string());
    }
    if !result.preview.is_empty() {
        text.push('\t');
        text.push_str(&result.preview);
    }
    Some(text)
}

fn copy_logs(state: &AppState) -> Option<String> {
    if state.logs.entries.is_empty() {
        return None;
    }

    Some(
        state
            .logs
            .entries
            .iter()
            .map(|entry| entry.message.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn copy_scrollback_pane(
    scrollback: &ScrollbackBuffer,
    follow_tail: bool,
    viewport_offset: usize,
    visible_height: usize,
    max_width: usize,
) -> Option<String> {
    if visible_height == 0 || max_width == 0 {
        return None;
    }

    let start = scrollback.viewport_start(visible_height, follow_tail, viewport_offset);
    let lines = scrollback.viewport_lines(start, visible_height, max_width, follow_tail);
    if lines.is_empty() {
        return None;
    }

    Some(
        lines
            .into_iter()
            .map(|line| strip_ansi(&line))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn shell_visible_rows(state: &AppState) -> usize {
    state.layout.rects.shell.height.saturating_sub(2) as usize
}

fn shell_visible_cols(state: &AppState) -> usize {
    state.layout.rects.shell.width.saturating_sub(2) as usize
}

fn agent_visible_rows(state: &AppState) -> usize {
    state.layout.rects.main_content.height.saturating_sub(4) as usize
}

fn agent_visible_cols(state: &AppState) -> usize {
    state.layout.rects.main_content.width.saturating_sub(2) as usize
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::layout::FocusTarget;
    use crate::navigation::{LeftNavTab, MainTab, NavCommand};
    use crate::search::{SearchMode, SearchResult, SearchState};
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("."),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        )
    }

    #[test]
    fn copy_preview_line_uses_scroll_offset() {
        let mut state = test_state();
        state.preview.lines = vec!["alpha".to_string(), "beta".to_string()];
        state.preview.scroll_offset = 1;
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));
        assert_eq!(resolve_copy_text(&state), Some("beta".to_string()));
    }

    #[test]
    fn copy_palette_input_when_open() {
        let mut state = test_state();
        state.palette.open = true;
        state.palette.input = "git ref".to_string();
        assert_eq!(resolve_copy_text(&state), Some("git ref".to_string()));
    }

    #[test]
    fn paste_target_routes_shell_and_search() {
        let mut state = test_state();
        state.shell.running = true;
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Shell));
        assert_eq!(resolve_paste_target(&state), PasteTarget::ShellPty);

        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Left));
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Search));
        assert_eq!(resolve_paste_target(&state), PasteTarget::SearchQuery);
    }

    #[test]
    fn copy_search_selection_includes_line_and_preview() {
        let mut state = test_state();
        state.search = SearchState {
            mode: SearchMode::Content,
            query: "fn".to_string(),
            results: vec![SearchResult::content(
                PathBuf::from("src/main.rs"),
                12,
                "fn main()".to_string(),
            )],
            selected: 0,
            ..SearchState::default()
        };
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Left));
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Search));
        assert_eq!(
            resolve_copy_text(&state),
            Some("src/main.rs:12\tfn main()".to_string())
        );
    }
}
