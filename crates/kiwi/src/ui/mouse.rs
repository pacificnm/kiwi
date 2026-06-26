//! Mouse hit-testing for basic tab interactions per SPEC-014 / ADR-015.

use ratatui::layout::Rect;

use crate::config::{MouseMode, MouseSettings};
use crate::layout::{FocusTarget, LayoutRects};
use crate::navigation::{LeftNavTab, MainTab, NavCommand, LEFT_TAB_LABELS, MAIN_TAB_LABELS};
use crate::state::AppState;

use super::tabs::tab_index_at_x;

pub fn map_mouse_click(state: &AppState, column: u16, row: u16) -> Vec<NavCommand> {
    if !mouse_interactions_enabled(&state.config.mouse) {
        return Vec::new();
    }

    map_mouse_click_in_layout(&state.layout.rects, state.navigation.main_tab, column, row)
}

pub fn map_mouse_click_in_layout(
    rects: &LayoutRects,
    main_tab: MainTab,
    column: u16,
    row: u16,
) -> Vec<NavCommand> {
    if let Some(command) = map_tab_click_in_layout(rects, column, row) {
        let focus = match &command {
            NavCommand::SelectLeftTab(_) => FocusTarget::Left,
            NavCommand::SelectMainTab(_) => FocusTarget::Main,
            _ => return vec![command],
        };
        return vec![command, NavCommand::SetFocus(focus)];
    }

    if matches!(main_tab, MainTab::Agent | MainTab::Diff | MainTab::Preview)
        && point_in_rect(column, row, rects.main_content)
    {
        return vec![NavCommand::SetFocus(FocusTarget::Main)];
    }

    if point_in_rect(column, row, rects.shell) {
        return vec![NavCommand::SetFocus(FocusTarget::Shell)];
    }

    Vec::new()
}

pub fn map_tab_click_in_layout(rects: &LayoutRects, column: u16, row: u16) -> Option<NavCommand> {
    if point_in_rect(column, row, rects.left_tabs) {
        let local_x = column.saturating_sub(rects.left_tabs.x);
        return tab_index_at_x(local_x, &LEFT_TAB_LABELS)
            .and_then(LeftNavTab::from_index)
            .map(NavCommand::SelectLeftTab);
    }

    if point_in_rect(column, row, rects.main_tabs) {
        let local_x = column.saturating_sub(rects.main_tabs.x);
        return tab_index_at_x(local_x, &MAIN_TAB_LABELS)
            .and_then(MainTab::from_index)
            .map(NavCommand::SelectMainTab);
    }

    None
}

pub fn mouse_interactions_enabled(mouse: &MouseSettings) -> bool {
    mouse.enabled && mouse.mode == MouseMode::Hybrid
}

fn point_in_rect(column: u16, row: u16, area: Rect) -> bool {
    column >= area.x
        && column < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}

#[cfg(test)]
mod tests {
    use crate::config::ResolvedConfig;
    use crate::layout::{compute_layout, FocusTarget};
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            std::path::PathBuf::from("."),
            false,
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
    fn click_on_left_tab_label_selects_tab() {
        let state = test_state();
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.left_tabs.x + 10, rects.left_tabs.y),
            vec![
                NavCommand::SelectLeftTab(LeftNavTab::Git),
                NavCommand::SetFocus(FocusTarget::Left),
            ]
        );
    }

    #[test]
    fn click_on_main_tab_label_selects_tab() {
        let state = test_state();
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.main_tabs.x + 10, rects.main_tabs.y),
            vec![
                NavCommand::SelectMainTab(MainTab::Issues),
                NavCommand::SetFocus(FocusTarget::Main),
            ]
        );
    }

    #[test]
    fn click_outside_tab_bars_is_ignored() {
        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Git));
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.left_content.x + 1, rects.left_content.y + 1),
            Vec::<NavCommand>::new()
        );
    }

    #[test]
    fn click_on_shell_pane_focuses_shell() {
        let state = test_state();
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.shell.x + 2, rects.shell.y + 2),
            vec![NavCommand::SetFocus(FocusTarget::Shell)]
        );
    }

    #[test]
    fn click_on_agent_pane_focuses_main() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.main_content.x + 2, rects.main_content.y + 2),
            vec![NavCommand::SetFocus(FocusTarget::Main)]
        );
    }

    #[test]
    fn click_on_diff_pane_focuses_main() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Diff;
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.main_content.x + 2, rects.main_content.y + 2),
            vec![NavCommand::SetFocus(FocusTarget::Main)]
        );
    }

    #[test]
    fn click_on_preview_pane_focuses_main() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Preview;
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.main_content.x + 2, rects.main_content.y + 2),
            vec![NavCommand::SetFocus(FocusTarget::Main)]
        );
    }

    #[test]
    fn click_on_main_content_ignored_on_non_interactive_main_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.main_content.x + 2, rects.main_content.y + 2),
            Vec::<NavCommand>::new()
        );
    }

    #[test]
    fn mouse_disabled_skips_tab_click_mapping() {
        let mut state = test_state();
        state.config.mouse.enabled = false;
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.left_tabs.x, rects.left_tabs.y),
            Vec::<NavCommand>::new()
        );
    }

    #[test]
    fn mouse_mode_disabled_skips_tab_click_mapping() {
        let mut state = test_state();
        state.config.mouse.mode = MouseMode::Disabled;
        let rects = state.layout.rects;
        assert_eq!(
            map_mouse_click(&state, rects.main_tabs.x, rects.main_tabs.y),
            Vec::<NavCommand>::new()
        );
    }
}
