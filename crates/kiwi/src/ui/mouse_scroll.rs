//! Mouse wheel hit-testing and scroll command mapping (SPEC-014 / issue #148).

use ratatui::layout::Rect;

use crate::github::GitHubLeftPane;
use crate::layout::{FocusTarget, LayoutRects};
use crate::navigation::{LeftNavTab, MainTab};
use crate::state::{AppCommand, AppState};

use super::mouse::mouse_interactions_enabled;

/// Lines scrolled per wheel tick (see docs/design/mouse-interaction.md).
pub const MOUSE_SCROLL_LINES: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollPane {
    Left,
    Main,
    Palette,
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelDirection {
    Up,
    Down,
    Left,
    Right,
}

impl WheelDirection {
    fn vertical_delta(self) -> i32 {
        match self {
            Self::Up => -MOUSE_SCROLL_LINES,
            Self::Down => MOUSE_SCROLL_LINES,
            Self::Left | Self::Right => 0,
        }
    }

    fn horizontal_delta(self) -> i32 {
        match self {
            Self::Left => -MOUSE_SCROLL_LINES,
            Self::Right => MOUSE_SCROLL_LINES,
            Self::Up | Self::Down => 0,
        }
    }
}

pub fn map_mouse_wheel(
    state: &AppState,
    column: u16,
    row: u16,
    direction: WheelDirection,
) -> Option<AppCommand> {
    if !mouse_interactions_enabled(&state.config.mouse) {
        return None;
    }

    if direction == WheelDirection::Left || direction == WheelDirection::Right {
        return map_horizontal_wheel(state, column, row, direction);
    }

    let delta = direction.vertical_delta();
    if delta == 0 {
        return None;
    }

    let pane = pane_at_point(&state.layout.rects, column, row)
        .filter(|pane| pane_is_scrollable(state, *pane))
        .or_else(|| focused_scroll_pane(state));

    map_wheel_to_command(state, pane?, delta)
}

fn map_horizontal_wheel(
    state: &AppState,
    column: u16,
    row: u16,
    direction: WheelDirection,
) -> Option<AppCommand> {
    let delta = direction.horizontal_delta();
    if delta == 0 || state.config.diff.word_wrap {
        return None;
    }

    let over_main = point_in_rect(column, row, state.layout.rects.main_content);
    let main_diff = state.navigation.main_tab == MainTab::Diff;
    let focused_diff =
        state.navigation.focus == FocusTarget::Main && state.navigation.main_tab == MainTab::Diff;

    if (over_main && main_diff) || (!over_main && focused_diff) {
        return Some(AppCommand::DiffHorizontalScroll(delta));
    }

    None
}

fn pane_at_point(rects: &LayoutRects, column: u16, row: u16) -> Option<ScrollPane> {
    if point_in_rect(column, row, rects.left_content) {
        Some(ScrollPane::Left)
    } else if point_in_rect(column, row, rects.main_content) {
        Some(ScrollPane::Main)
    } else if point_in_rect(column, row, rects.palette) {
        Some(ScrollPane::Palette)
    } else if point_in_rect(column, row, rects.shell) {
        Some(ScrollPane::Shell)
    } else {
        None
    }
}

fn focused_scroll_pane(state: &AppState) -> Option<ScrollPane> {
    match state.navigation.focus {
        FocusTarget::Left => Some(ScrollPane::Left),
        FocusTarget::Main => Some(ScrollPane::Main),
        FocusTarget::CommandPalette => Some(ScrollPane::Palette),
        FocusTarget::Shell => Some(ScrollPane::Shell),
    }
    .filter(|pane| pane_is_scrollable(state, *pane))
}

fn pane_is_scrollable(state: &AppState, pane: ScrollPane) -> bool {
    match pane {
        ScrollPane::Left => left_pane_scrollable(state),
        ScrollPane::Main => main_pane_scrollable(state),
        ScrollPane::Palette => palette_scrollable(state),
        ScrollPane::Shell => true,
    }
}

fn palette_scrollable(state: &AppState) -> bool {
    state.palette.open && state.palette.prompt.is_none() && !state.palette.matches.is_empty()
}

fn left_pane_scrollable(state: &AppState) -> bool {
    match state.navigation.left_tab {
        LeftNavTab::Files | LeftNavTab::Git | LeftNavTab::Search => true,
        LeftNavTab::Gh => matches!(
            state.github.left_pane,
            GitHubLeftPane::Issues | GitHubLeftPane::Prs
        ),
    }
}

fn main_pane_scrollable(state: &AppState) -> bool {
    matches!(
        state.navigation.main_tab,
        MainTab::Agent
            | MainTab::Issues
            | MainTab::Branches
            | MainTab::Prs
            | MainTab::Diff
            | MainTab::Preview
            | MainTab::Settings
    )
}

fn map_wheel_to_command(state: &AppState, pane: ScrollPane, delta: i32) -> Option<AppCommand> {
    match pane {
        ScrollPane::Left => left_wheel_command(state, delta),
        ScrollPane::Main => main_wheel_command(state, delta),
        ScrollPane::Palette => Some(AppCommand::PaletteMoveSelection(delta)),
        ScrollPane::Shell => Some(AppCommand::ShellScrollLines(delta)),
    }
}

fn left_wheel_command(state: &AppState, delta: i32) -> Option<AppCommand> {
    match state.navigation.left_tab {
        LeftNavTab::Files => Some(AppCommand::FileTreeMoveSelection(delta)),
        LeftNavTab::Git => Some(AppCommand::GitMoveSelection(delta)),
        LeftNavTab::Search => Some(AppCommand::SearchMoveSelection(delta)),
        LeftNavTab::Gh => match state.github.left_pane {
            GitHubLeftPane::Issues => Some(AppCommand::GitHubMoveIssueSelection(delta)),
            GitHubLeftPane::Prs => Some(AppCommand::GitHubMovePrSelection(delta)),
        },
    }
}

fn main_wheel_command(state: &AppState, delta: i32) -> Option<AppCommand> {
    match state.navigation.main_tab {
        MainTab::Agent => Some(AppCommand::AgentScrollLines(delta)),
        MainTab::Issues => Some(AppCommand::GitHubIssueDetailScroll(delta)),
        MainTab::Branches => Some(AppCommand::BranchMoveSelection(delta)),
        MainTab::Prs => Some(AppCommand::GitHubPrDetailScroll(delta)),
        MainTab::Diff => Some(AppCommand::DiffScroll(delta)),
        MainTab::Preview => Some(AppCommand::PreviewScroll(delta)),
        MainTab::Settings => Some(AppCommand::SettingsMoveSelection(delta)),
        MainTab::Logs => None,
    }
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
    use crate::file_tree::FileTreeState;
    use crate::layout::compute_layout;
    use crate::navigation::{LeftNavTab, MainTab};
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
    fn wheel_over_left_files_maps_file_tree_scroll() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Files;
        state.file_tree = FileTreeState::default();
        let rects = state.layout.rects;
        let cmd = map_mouse_wheel(
            &state,
            rects.left_content.x + 1,
            rects.left_content.y + 1,
            WheelDirection::Down,
        )
        .expect("command");
        assert_eq!(cmd, AppCommand::FileTreeMoveSelection(MOUSE_SCROLL_LINES));
    }

    #[test]
    fn wheel_over_shell_maps_shell_scroll_even_when_left_focused() {
        let state = test_state();
        let rects = state.layout.rects;
        let cmd = map_mouse_wheel(
            &state,
            rects.shell.x + 1,
            rects.shell.y + 1,
            WheelDirection::Up,
        )
        .expect("command");
        assert_eq!(cmd, AppCommand::ShellScrollLines(-MOUSE_SCROLL_LINES));
    }

    #[test]
    fn wheel_outside_panes_falls_back_to_focused_pane() {
        let mut state = test_state();
        state.navigation.focus = FocusTarget::Main;
        state.navigation.main_tab = MainTab::Preview;
        let cmd = map_mouse_wheel(&state, 0, 0, WheelDirection::Down).expect("command");
        assert_eq!(cmd, AppCommand::PreviewScroll(MOUSE_SCROLL_LINES));
    }

    #[test]
    fn wheel_disabled_when_mouse_off() {
        let mut state = test_state();
        state.config.mouse.enabled = false;
        let rects = state.layout.rects;
        assert!(map_mouse_wheel(
            &state,
            rects.left_content.x,
            rects.left_content.y,
            WheelDirection::Down,
        )
        .is_none());
    }

    #[test]
    fn closed_palette_falls_back_to_focus() {
        let mut state = test_state();
        state.palette.open = false;
        state.navigation.focus = FocusTarget::Main;
        state.navigation.main_tab = MainTab::Agent;
        let cmd = map_mouse_wheel(
            &state,
            state.layout.rects.palette.x,
            state.layout.rects.palette.y,
            WheelDirection::Down,
        )
        .expect("command");
        assert_eq!(cmd, AppCommand::AgentScrollLines(MOUSE_SCROLL_LINES));
    }
}
