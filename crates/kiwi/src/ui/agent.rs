use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::Frame;

use crate::selection::SelectionPane;
use crate::state::{AppCommand, AppState};
use crate::theme::ThemePalette;

use super::scrollback::{render_scrollback_pane, ScrollbackPane};
use super::tabs::{tab_bar_line_str, tab_index_at_x};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentPaneLayout {
    pub subtabs: Rect,
    pub pane: Rect,
}

#[must_use]
pub fn agent_subtabs_visible(state: &AppState) -> bool {
    state.agent_manager.session_count() > 1
}

pub fn split_agent_main_content(area: Rect, show_subtabs: bool) -> AgentPaneLayout {
    if !show_subtabs || area.height < 2 {
        return AgentPaneLayout {
            subtabs: Rect::default(),
            pane: area,
        };
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    AgentPaneLayout {
        subtabs: chunks[0],
        pane: chunks[1],
    }
}

#[must_use]
pub fn agent_scrollback_area(state: &AppState) -> Rect {
    split_agent_main_content(
        state.layout.rects.main_content,
        agent_subtabs_visible(state),
    )
    .pane
}

pub fn agent_session_tab_labels(state: &AppState) -> Vec<String> {
    state
        .agent_manager
        .sessions()
        .map(|session| {
            if session.pty.running {
                format!("{} •", session.label)
            } else {
                session.label.clone()
            }
        })
        .collect()
}

#[must_use]
pub fn active_agent_tab_index(state: &AppState) -> usize {
    let active = state.agent_manager.active_id();
    state
        .agent_manager
        .session_ids()
        .position(|id| id == active)
        .unwrap_or(0)
}

pub fn render_agent_tab(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    hint_style: Style,
    chrome: Style,
    state: &AppState,
) {
    let show_subtabs = agent_subtabs_visible(state);
    let layout = split_agent_main_content(area, show_subtabs);

    if show_subtabs {
        render_agent_subtabs(frame, layout.subtabs, theme, chrome, state);
    }

    let agent_title = format!(
        "Agent: {}",
        state.agent_manager.active_session().map(|s| s.label.as_str()).unwrap_or("Agent")
    );
    render_agent_pane(
        frame,
        layout.pane,
        &agent_title,
        focused,
        theme,
        hint_style,
        state,
    );
}

fn render_agent_subtabs(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &ThemePalette,
    chrome: Style,
    state: &AppState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let labels = agent_session_tab_labels(state);
    let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    let selected = active_agent_tab_index(state);
    let line = tab_bar_line_str(&refs, selected, theme);
    frame.render_widget(ratatui::widgets::Paragraph::new(line).style(chrome), area);
}

pub fn render_agent_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    focused: bool,
    theme: &ThemePalette,
    hint_style: Style,
    state: &AppState,
) {
    render_scrollback_pane(
        frame,
        area,
        title,
        focused,
        theme,
        hint_style,
        ScrollbackPane {
            scrollback: &state.active_agent().scrollback,
            follow_tail: state.active_agent().follow_tail,
            viewport_offset: state.active_agent().viewport_offset,
            spawn_error: None,
            idle_hint: None,
            footer: state.active_agent().restart_hint.as_deref(),
            selection_pane: Some(SelectionPane::Agent),
            show_pty_cursor: focused
                && state.active_agent().running
                && state.active_agent().follow_tail
                && state.pty_cursor_blink_on,
        },
        &state.text_selection,
    );
}

#[must_use]
pub fn map_agent_session_click(state: &AppState, column: u16, row: u16) -> Option<AppCommand> {
    use crate::navigation::MainTab;

    if state.navigation.main_tab != MainTab::Agent || !agent_subtabs_visible(state) {
        return None;
    }

    let layout = split_agent_main_content(state.layout.rects.main_content, true);
    if layout.subtabs.width == 0
        || layout.subtabs.height == 0
        || column < layout.subtabs.x
        || column >= layout.subtabs.x.saturating_add(layout.subtabs.width)
        || row != layout.subtabs.y
    {
        return None;
    }

    let labels = agent_session_tab_labels(state);
    let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    let local_x = column.saturating_sub(layout.subtabs.x);
    let index = tab_index_at_x(local_x, &refs)?;
    let id = state.agent_manager.session_id_at_index(index)?;
    Some(AppCommand::AgentSetActive(id))
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::agent::AgentId;
    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::navigation::MainTab;
    use crate::state::{AppCommand, AppState};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::super::tabs::{TAB_LEADING_PAD, TAB_SEPARATOR};
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
    fn subtabs_hidden_for_single_session() {
        let state = test_state();
        assert!(!agent_subtabs_visible(&state));
        let layout = split_agent_main_content(Rect::new(0, 0, 80, 20), false);
        assert_eq!(layout.pane.height, 20);
        assert_eq!(layout.subtabs.height, 0);
    }

    #[test]
    fn subtabs_split_reserves_one_row() {
        let layout = split_agent_main_content(Rect::new(0, 0, 80, 20), true);
        assert_eq!(layout.subtabs.height, 1);
        assert_eq!(layout.pane.height, 19);
    }

    #[test]
    fn session_click_selects_agent() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        state
            .agent_manager
            .create_agent(Some("second".to_string()), None)
            .expect("create");
        let second = state.agent_manager.active_id();
        state
            .agent_manager
            .set_active(AgentId::FIRST)
            .expect("switch");

        let layout = split_agent_main_content(state.layout.rects.main_content, true);
        let labels = agent_session_tab_labels(&state);
        let offset = TAB_LEADING_PAD.len() + labels[0].len() + TAB_SEPARATOR.len();
        let click_x = layout.subtabs.x + u16::try_from(offset).expect("offset");
        let command = map_agent_session_click(&state, click_x, layout.subtabs.y).expect("click");

        assert_eq!(command, AppCommand::AgentSetActive(second));
    }

    #[test]
    fn render_agent_tab_with_subtabs() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Agent;
        state
            .agent_manager
            .create_agent(Some("tests".to_string()), None)
            .expect("create");

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_agent_tab(
                    frame,
                    state.layout.rects.main_content,
                    true,
                    &state.theme,
                    Style::default(),
                    Style::default(),
                    &state,
                );
            })
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let text = buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("tests"));
    }
}
