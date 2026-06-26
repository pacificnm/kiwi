//! Copy/paste target resolution using domain state and [`ViewportMetrics`].

use crate::ansi::strip_ansi;
use crate::navigation::{FocusTarget, LeftNavTab, MainTab};
use crate::shell::ScrollbackBuffer;
use crate::state::ReduceView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteTarget {
    PaletteInput,
    SearchQuery,
    ShellPty,
    AgentPty,
    Unsupported,
}

pub fn resolve_copy_text_for_focus(state: &ReduceView<'_>, focus: FocusTarget) -> Option<String> {
    match focus {
        FocusTarget::CommandPalette => copy_palette_input(state),
        FocusTarget::Left => copy_left_pane(state),
        FocusTarget::Main => copy_main_pane(state),
        FocusTarget::Shell => copy_scrollback_pane(
            &state.shell.scrollback,
            state.shell.follow_tail,
            state.shell.viewport_offset,
            usize::from(state.viewport.shell_rows),
            usize::from(state.viewport.shell_cols),
        ),
    }
}

pub fn resolve_paste_target(state: &ReduceView<'_>) -> PasteTarget {
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
        && state.active_agent().running
    {
        return PasteTarget::AgentPty;
    }

    PasteTarget::Unsupported
}

fn copy_palette_input(state: &ReduceView<'_>) -> Option<String> {
    if state.palette.input.is_empty() {
        None
    } else {
        Some(state.palette.input.clone())
    }
}

fn copy_left_pane(state: &ReduceView<'_>) -> Option<String> {
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

fn copy_main_pane(state: &ReduceView<'_>) -> Option<String> {
    match state.navigation.main_tab {
        MainTab::Preview => copy_preview_line(state),
        MainTab::Agent => copy_scrollback_pane(
            &state.active_agent().scrollback,
            state.active_agent().follow_tail,
            state.active_agent().viewport_offset,
            usize::from(state.viewport.agent_rows),
            usize::from(state.viewport.agent_cols),
        ),
        MainTab::Logs => copy_logs(state),
        _ => None,
    }
}

fn copy_preview_line(state: &ReduceView<'_>) -> Option<String> {
    let line_index = state.preview.scroll_offset;
    state.preview.lines.get(line_index).cloned()
}

fn copy_search_query(state: &ReduceView<'_>) -> Option<String> {
    if state.search.query.is_empty() {
        None
    } else {
        Some(state.search.query.clone())
    }
}

fn copy_search_selection(state: &ReduceView<'_>) -> Option<String> {
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

fn copy_logs(state: &ReduceView<'_>) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::navigation::{NavCommand, NavigationState};
    use crate::search::{SearchMode, SearchResult, SearchState};
    use crate::state::{AppState, ViewportMetrics};
    use crate::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;

    fn test_view() -> (AppState, ViewportMetrics) {
        let mut state = AppState::from_startup(
            PathBuf::from("."),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        );
        state.viewport.shell_rows = 20;
        state.viewport.shell_cols = 80;
        state.viewport.agent_rows = 15;
        state.viewport.agent_cols = 100;
        let viewport = state.viewport;
        (state, viewport)
    }

    #[test]
    fn copy_preview_line_uses_scroll_offset() {
        let (mut app, _) = test_view();
        app.preview.lines = vec!["alpha".to_string(), "beta".to_string()];
        app.preview.scroll_offset = 1;
        app.navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));
        let mut view = ReduceView::from_app_state(&mut app);
        assert_eq!(
            resolve_copy_text_for_focus(&view, FocusTarget::Main),
            Some("beta".to_string())
        );
        let _ = &mut view;
    }

    #[test]
    fn copy_palette_input_when_open() {
        let (mut app, _) = test_view();
        app.palette.open = true;
        app.palette.input = "git ref".to_string();
        let view = ReduceView::from_app_state(&mut app);
        assert_eq!(
            resolve_copy_text_for_focus(&view, FocusTarget::CommandPalette),
            Some("git ref".to_string())
        );
    }

    #[test]
    fn paste_target_routes_shell_and_search() {
        let (mut app, _) = test_view();
        app.shell.running = true;
        app.navigation
            .apply(NavCommand::SetFocus(FocusTarget::Shell));
        let view = ReduceView::from_app_state(&mut app);
        assert_eq!(resolve_paste_target(&view), PasteTarget::ShellPty);

        app.navigation
            .apply(NavCommand::SetFocus(FocusTarget::Left));
        app.navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Search));
        let view = ReduceView::from_app_state(&mut app);
        assert_eq!(resolve_paste_target(&view), PasteTarget::SearchQuery);
    }

    #[test]
    fn copy_search_selection_includes_line_and_preview() {
        let (mut app, _) = test_view();
        app.search = SearchState {
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
        app.navigation
            .apply(NavCommand::SetFocus(FocusTarget::Left));
        app.navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Search));
        let view = ReduceView::from_app_state(&mut app);
        assert_eq!(
            resolve_copy_text_for_focus(&view, FocusTarget::Left),
            Some("src/main.rs:12\tfn main()".to_string())
        );
    }
}
