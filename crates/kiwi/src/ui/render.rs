use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::layout::Region;
use crate::navigation::{LeftNavTab, MainTab, LEFT_TAB_LABELS, MAIN_TAB_LABELS};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::agent::render_agent_tab;
use super::branches::render_branches_pane;
use super::context_menu::render_github_context_menu;
use super::diff::render_diff_pane;
use super::file_tree::render_file_tree_pane;
use super::git::render_git_pane;
use super::github::{render_github_left_pane, render_issue_detail_pane, render_pr_detail_pane};
use super::label_picker::render_label_picker_overlay;
use super::logs::render_logs_pane;
use super::notifications::render_notifications;
use super::palette::render_palette_pane;
use super::preview::render_preview_pane;
use super::search::render_search_pane;
use super::settings::render_settings_pane;
use super::shell::render_shell_pane;
use super::status_bar::render_status_bar;
use super::tabs::tab_bar_line;

pub fn draw_frame(frame: &mut Frame<'_>, state: &AppState) {
    let chrome = chrome_style(&state.theme);
    let chrome_bg = chrome_background(&state.theme);
    frame.render_widget(Clear, frame.area());
    frame.render_widget(Paragraph::new("").style(chrome_bg), frame.area());

    render_tab_bar(
        frame,
        state.layout.rects.left_tabs,
        &LEFT_TAB_LABELS,
        state.navigation.left_tab.index(),
        &state.theme,
        chrome,
    );
    render_tab_bar(
        frame,
        state.layout.rects.main_tabs,
        &MAIN_TAB_LABELS,
        state.navigation.main_tab.index(),
        &state.theme,
        chrome,
    );

    if state.navigation.left_tab == LeftNavTab::Files {
        render_file_tree_pane(
            frame,
            state.layout.rects.left_content,
            state.navigation.focus.is_focused(Region::LeftContent),
            &state.theme,
            state,
        );
    } else if state.navigation.left_tab == LeftNavTab::Search {
        render_search_pane(
            frame,
            state.layout.rects.left_content,
            state.navigation.focus.is_focused(Region::LeftContent),
            &state.theme,
            state,
        );
    } else if state.navigation.left_tab == LeftNavTab::Git {
        render_git_pane(
            frame,
            state.layout.rects.left_content,
            state.navigation.focus.is_focused(Region::LeftContent),
            &state.theme,
            state,
        );
    } else if state.navigation.left_tab == LeftNavTab::Gh {
        render_github_left_pane(
            frame,
            state.layout.rects.left_content,
            state.navigation.focus.is_focused(Region::LeftContent),
            &state.theme,
            state,
        );
    }
    if state.navigation.main_tab == MainTab::Agent {
        render_agent_tab(
            frame,
            state.layout.rects.main_content,
            state.navigation.focus.is_focused(Region::MainContent),
            &state.theme,
            hint_style(&state.theme),
            chrome_style(&state.theme),
            state,
        );
    } else if state.navigation.main_tab == MainTab::Preview {
        render_preview_pane(
            frame,
            state.layout.rects.main_content,
            state.navigation.focus.is_focused(Region::MainContent),
            &state.theme,
            state,
        );
    } else if state.navigation.main_tab == MainTab::Diff {
        render_diff_pane(
            frame,
            state.layout.rects.main_content,
            state.navigation.focus.is_focused(Region::MainContent),
            &state.theme,
            state,
        );
    } else if state.navigation.main_tab == MainTab::Logs {
        render_logs_pane(
            frame,
            state.layout.rects.main_content,
            state.navigation.focus.is_focused(Region::MainContent),
            &state.theme,
            state,
        );
    } else if state.navigation.main_tab == MainTab::Settings {
        render_settings_pane(
            frame,
            state.layout.rects.main_content,
            state.navigation.focus.is_focused(Region::MainContent),
            &state.theme,
            state,
        );
    } else if state.navigation.main_tab == MainTab::Issues {
        render_issue_detail_pane(
            frame,
            state.layout.rects.main_content,
            state.navigation.focus.is_focused(Region::MainContent),
            &state.theme,
            state,
        );
    } else if state.navigation.main_tab == MainTab::Branches {
        render_branches_pane(
            frame,
            state.layout.rects.main_content,
            state.navigation.focus.is_focused(Region::MainContent),
            &state.theme,
            state,
        );
    } else if state.navigation.main_tab == MainTab::Prs {
        render_pr_detail_pane(
            frame,
            state.layout.rects.main_content,
            state.navigation.focus.is_focused(Region::MainContent),
            &state.theme,
            state,
        );
    } else {
        render_pane(
            frame,
            state.layout.rects.main_content,
            state.navigation.main_tab.label(),
            state.navigation.focus.is_focused(Region::MainContent),
            &state.theme,
            chrome,
            Some(main_pane_line(state)),
        );
    }
    if state.github.label_picker.is_some() {
        render_label_picker_overlay(frame, state.layout.rects.main_content, &state.theme, state);
    }
    if state.github.context_menu.is_some() {
        if let Some(menu) = &state.github.context_menu {
            render_github_context_menu(frame, state.layout.rects.left_content, &state.theme, menu);
        }
    }
    render_palette_pane(
        frame,
        state.layout.rects.palette,
        state.navigation.focus.is_focused(Region::Palette),
        &state.theme,
        state,
    );

    let shell_title = format!("Shell: {}", state.shell.shell_name);
    render_shell_pane(
        frame,
        state.layout.rects.shell,
        &shell_title,
        state.navigation.focus.is_focused(Region::Shell),
        &state.theme,
        hint_style(&state.theme),
        state,
    );

    render_status_bar(frame, state.layout.rects.status_bar, state);
    render_notifications(frame, state.layout.rects.status_bar, state);
}

fn main_pane_line(state: &AppState) -> Line<'_> {
    let slot = state.navigation.main_slot();
    Line::from(format!(
        "{} view (selection: {})",
        state.navigation.main_tab.label(),
        slot.selected_index
    ))
}

fn render_tab_bar(
    frame: &mut Frame<'_>,
    area: Rect,
    tabs: &[&'static str],
    selected: usize,
    theme: &ThemePalette,
    chrome: Style,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let line = tab_bar_line(tabs, selected, theme);
    frame.render_widget(Paragraph::new(line).style(chrome), area);
}

fn render_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    focused: bool,
    theme: &ThemePalette,
    chrome: Style,
    content: Option<Line<'_>>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(line) = content {
        frame.render_widget(Paragraph::new(line).style(chrome), inner);
    }
}

fn chrome_style(theme: &ThemePalette) -> Style {
    let mut style = Style::default();
    if let Some(bg) = theme.get(SemanticRole::Bg).bg {
        style = style.bg(bg);
    }
    if let Some(fg) = theme.get(SemanticRole::Fg).fg {
        style = style.fg(fg);
    }
    style
}

fn chrome_background(theme: &ThemePalette) -> Style {
    let mut style = Style::default();
    if let Some(bg) = theme.get(SemanticRole::Bg).bg {
        style = style.bg(bg);
    }
    style
}

fn hint_style(theme: &ThemePalette) -> Style {
    theme.get(SemanticRole::Muted)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ratatui::backend::TestBackend;
    use ratatui::style::Modifier;
    use ratatui::Terminal;

    use crate::config::ResolvedConfig;
    use crate::git::GitFileStatus;
    use crate::layout::{compute_layout, FocusTarget};
    use crate::navigation::{LeftNavTab, MainTab, NavCommand};
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use crate::state::{reduce, AppEvent};
    use crate::ui::status_bar::compute_status_bar;
    use kiwi_core::status_bar::{display_width, format_status_line};

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("."),
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
    fn draw_frame_renders_tab_labels_and_pane_titles() {
        let state = test_state();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let content = buffer_content(buffer);

        for label in LEFT_TAB_LABELS {
            assert!(content.contains(label), "missing left tab {label}");
        }
        for label in MAIN_TAB_LABELS {
            assert!(content.contains(label), "missing main tab {label}");
        }
        assert!(content.contains("Shell: bash"));
        assert!(content.contains("Ctrl+P for commands"));
        assert!(content.contains("▸") || content.contains("▾"));
        assert!(content.contains("Agent: agent"));
        assert!(content.contains("Kiwi |"));
        assert!(content.contains("Agent Idle"));
        assert!(content.contains("Clean"));
    }

    #[test]
    fn draw_frame_status_bar_reflects_git_and_agent_updates() {
        let mut state = test_state();
        state.git.branch = Some("main".to_string());
        state.git.file_entries = vec![crate::git::GitFileEntry {
            path: "src/lib.rs".to_string(),
            status: GitFileStatus::Modified,
        }];
        state.active_agent_mut().running = true;
        state.github.selected_issue = Some(7);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let content = buffer_content(terminal.backend().buffer());
        assert!(content.contains("Kiwi |"));
        assert!(content.contains("main"));
        assert!(content.contains("Agent Running"));
        assert!(content.contains("1 Modified"));
        assert!(content.contains("#7"));
    }

    #[test]
    fn draw_frame_status_bar_truncates_on_narrow_terminal() {
        let mut state = test_state();
        state.status_bar.root_name = "cityartwalks".to_string();
        state.git.branch = Some("feature/very-long-branch-name".to_string());
        state.git.file_entries = vec![
            crate::git::GitFileEntry {
                path: "a.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            crate::git::GitFileEntry {
                path: "b.rs".to_string(),
                status: GitFileStatus::Modified,
            },
        ];
        state.active_agent_mut().running = true;
        state.github.selected_issue = Some(99);

        let snapshot = compute_status_bar(&state);
        let formatted = format_status_line(&snapshot, 80);
        assert!(display_width(&formatted) <= 80);
        assert!(!formatted.contains("#99"));

        state.layout = compute_layout(80, 40, 30).expect("layout");
        let backend = TestBackend::new(80, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let content = buffer_content(terminal.backend().buffer());
        assert!(content.contains("Kiwi |"));
        assert!(content.contains("Root: cityartwalks"));
    }

    #[test]
    fn draw_frame_status_bar_updates_after_git_refresh_event() {
        let mut state = test_state();
        reduce(
            &mut state,
            AppEvent::GitStatusUpdated {
                branch: None,
                remote_repo: None,
                ahead: 0,
                behind: 0,
                file_entries: vec![
                    crate::git::GitFileEntry {
                        path: "src/main.rs".to_string(),
                        status: GitFileStatus::Modified,
                    },
                    crate::git::GitFileEntry {
                        path: "src/lib.rs".to_string(),
                        status: GitFileStatus::Modified,
                    },
                ],
                error: None,
            },
        );

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let content = buffer_content(terminal.backend().buffer());
        assert!(content.contains("2 Modified"));
    }

    #[test]
    fn draw_frame_reflects_orthogonal_tab_selection() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = true;
        state.git.branch = Some("main".to_string());
        state.git.file_entries = vec![crate::git::GitFileEntry {
            path: "src/main.rs".to_string(),
            status: GitFileStatus::Modified,
        }];
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Git));
        state
            .navigation
            .apply(NavCommand::SelectLeftTab(LeftNavTab::Gh));
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.issues = vec![crate::github::Issue {
            number: 1,
            title: "Sample".to_string(),
            state: crate::github::IssueState::Open,
            labels: Vec::new(),
            assignees: Vec::new(),
        }];
        state.github.selected_issue = Some(1);
        state.github.issues_loaded_at = Some(std::time::SystemTime::now());

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let content = buffer_content(terminal.backend().buffer());
        assert!(content.contains("Modified"));
        assert!(content.contains("#1"));
        assert!(content.contains("Sample"));
    }

    #[test]
    fn draw_frame_renders_shell_scrollback() {
        let mut state = test_state();
        state.shell.scrollback.append_bytes(b"hello kiwi\n");

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let content = buffer_content(terminal.backend().buffer());
        assert!(content.contains("hello kiwi"));
    }

    #[test]
    fn draw_frame_renders_shell_prompt_without_trailing_newline() {
        let mut state = test_state();
        state.shell.scrollback.append_bytes(b"user@host:~/kiwi$ ");

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let content = buffer_content(terminal.backend().buffer());
        assert!(content.contains("user@host:~/kiwi$"));
    }

    #[test]
    fn draw_frame_renders_shell_cursor_when_focused() {
        let mut state = test_state();
        state.navigation.focus = FocusTarget::Shell;
        state.shell.running = true;
        state.pty_cursor_blink_on = true;
        state.shell.scrollback.append_bytes(b"user@host:~/kiwi$ ");

        let rects = state.layout.rects;
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let shell = rects.shell;
        let mut found_cursor = false;
        for y in shell.y + 1..shell.y + shell.height - 1 {
            for x in shell.x + 1..shell.x + shell.width - 1 {
                let cell = &buffer[(x, y)];
                if cell.modifier.contains(Modifier::REVERSED) {
                    found_cursor = true;
                    break;
                }
            }
        }
        assert!(found_cursor, "expected reversed cursor cell in shell pane");
    }

    #[test]
    fn draw_frame_renders_agent_cursor_when_focused() {
        let mut state = test_state();
        state.navigation.focus = FocusTarget::Main;
        state.navigation.main_tab = MainTab::Agent;
        state.active_agent_mut().running = true;
        state.pty_cursor_blink_on = true;
        state.active_agent_mut().scrollback.append_bytes(b"agent> ");

        let rects = state.layout.rects;
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let main = rects.main_content;
        let mut found_cursor = false;
        for y in main.y + 1..main.y + main.height - 1 {
            for x in main.x + 1..main.x + main.width - 1 {
                let cell = &buffer[(x, y)];
                if cell.modifier.contains(Modifier::REVERSED) {
                    found_cursor = true;
                    break;
                }
            }
        }
        assert!(found_cursor, "expected reversed cursor cell in agent pane");
    }

    #[test]
    fn draw_frame_renders_agent_cursor_after_hide_cursor_sequence() {
        let mut state = test_state();
        state.navigation.focus = FocusTarget::Main;
        state.navigation.main_tab = MainTab::Agent;
        state.active_agent_mut().running = true;
        state.pty_cursor_blink_on = true;
        state
            .active_agent_mut()
            .scrollback
            .append_bytes(b"\x1b[?25lagent> ");

        let rects = state.layout.rects;
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let main = rects.main_content;
        let mut found_cursor = false;
        for y in main.y + 1..main.y + main.height - 1 {
            for x in main.x + 1..main.x + main.width - 1 {
                let cell = &buffer[(x, y)];
                if cell.modifier.contains(Modifier::REVERSED) {
                    found_cursor = true;
                    break;
                }
            }
        }
        assert!(
            found_cursor,
            "expected overlay cursor even when PTY sent ?25l"
        );
    }

    #[test]
    fn draw_frame_hides_shell_cursor_when_not_following_tail() {
        let mut state = test_state();
        state.navigation.focus = FocusTarget::Shell;
        state.shell.running = true;
        state.pty_cursor_blink_on = true;
        state.shell.follow_tail = false;
        state.shell.scrollback.append_bytes(b"user@host:~/kiwi$ ");

        let rects = state.layout.rects;
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let shell = rects.shell;
        for y in shell.y + 1..shell.y + shell.height - 1 {
            for x in shell.x + 1..shell.x + shell.width - 1 {
                let cell = &buffer[(x, y)];
                assert!(
                    !cell.modifier.contains(Modifier::REVERSED),
                    "cursor should not render when follow_tail is false"
                );
            }
        }
    }

    #[test]
    fn draw_frame_renders_agent_scrollback() {
        let mut state = test_state();
        state
            .active_agent_mut()
            .scrollback
            .append_bytes(b"agent output\n");

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let content = buffer_content(terminal.backend().buffer());
        assert!(content.contains("agent output"));
    }

    #[test]
    fn draw_frame_keeps_shell_output_inside_shell_pane() {
        let mut state = test_state();
        state.shell.scrollback.append_bytes(b"SHELL_ONLY_OUTPUT\n");

        let rects = state.layout.rects;
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| draw_frame(frame, &state))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let palette_text = region_text(buffer, rects.palette);
        assert!(!palette_text.contains("SHELL_ONLY_OUTPUT"));

        let shell_text = region_text(buffer, rects.shell);
        assert!(shell_text.contains("SHELL_ONLY_OUTPUT"));
    }

    fn region_text(buffer: &ratatui::buffer::Buffer, area: ratatui::layout::Rect) -> String {
        let mut out = String::new();
        for y in area.y..area.y.saturating_add(area.height) {
            for x in area.x..area.x.saturating_add(area.width) {
                out.push_str(buffer[(x, y)].symbol());
            }
        }
        out
    }

    fn buffer_content(buffer: &ratatui::buffer::Buffer) -> String {
        buffer.content.iter().map(|cell| cell.symbol()).collect()
    }
}
