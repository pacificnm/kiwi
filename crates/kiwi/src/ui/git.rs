use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::git::{build_panel_rows, git_row_at_viewport, git_selected_row_index, GitPanelRow};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::scrollbar::{render_vertical_scrollbar, split_for_scrollbar};

const BRANCH_ROWS: u16 = 1;
const STATUS_ROWS: u16 = 1;

pub fn git_interaction_at(state: &AppState, area: Rect, column: u16, row: u16) -> Option<usize> {
    if area.width == 0 || area.height == 0 {
        return None;
    }

    if column < area.x
        || column >= area.x.saturating_add(area.width)
        || row < area.y
        || row >= area.y.saturating_add(area.height)
    {
        return None;
    }

    let list_area = git_list_area(pane_inner(area)?);
    if column < list_area.x
        || column >= list_area.x.saturating_add(list_area.width)
        || row < list_area.y
        || row >= list_area.y.saturating_add(list_area.height)
    {
        return None;
    }

    let viewport_index = usize::from(row.saturating_sub(list_area.y));
    let row_index = state.git.scroll_offset.saturating_add(viewport_index);
    let rows = build_panel_rows(&state.git.file_entries, state.config.git.show_untracked);
    let row = rows.get(row_index)?;
    match row {
        GitPanelRow::File { .. } => Some(row_index),
        GitPanelRow::Header { .. } => None,
    }
}

pub fn render_git_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let title = git_pane_title(state);
    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let Some(inner) = pane_inner(area) else {
        return;
    };

    let branch_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: BRANCH_ROWS.min(inner.height),
    };
    render_branch_line(frame, branch_area, state, theme);

    let status_y = inner
        .y
        .saturating_add(inner.height.saturating_sub(STATUS_ROWS));
    let status_area = Rect {
        x: inner.x,
        y: status_y,
        width: inner.width,
        height: STATUS_ROWS.min(inner.height),
    };

    let list_area = git_list_area(inner);
    if list_area.height > 0 && list_area.width > 0 {
        render_file_list(frame, list_area, focused, theme, state);
    }

    if status_area.height > 0 {
        render_status_line(frame, status_area, state, theme);
    }
}

fn git_pane_title(state: &AppState) -> String {
    if !state.workspace_meta.is_git_repo {
        return "Git".to_string();
    }

    match state.git.branch.as_deref() {
        Some(branch) => format!("Git: {branch}"),
        None => "Git".to_string(),
    }
}

fn render_branch_line(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &ThemePalette) {
    let text = if !state.workspace_meta.is_git_repo {
        "Not a git repository".to_string()
    } else if state.git.loading {
        "Refreshing…".to_string()
    } else if let Some(error) = &state.git.error {
        error.clone()
    } else {
        let mut parts = Vec::new();
        if let Some(branch) = &state.git.branch {
            parts.push(branch.clone());
        }
        if state.git.ahead > 0 || state.git.behind > 0 {
            parts.push(format!("↑{} ↓{}", state.git.ahead, state.git.behind));
        }
        if parts.is_empty() {
            "No branch".to_string()
        } else {
            parts.join(" · ")
        }
    };

    frame.render_widget(
        Paragraph::new(truncate_line(&text, area.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn render_status_line(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &ThemePalette) {
    let status = if !state.workspace_meta.is_git_repo {
        "Git features disabled"
    } else if state.git.loading {
        "Refreshing…"
    } else if state.git.file_entries.is_empty() {
        "Clean · R refresh · Enter diff"
    } else {
        "j/k move · Enter diff · R refresh"
    };

    frame.render_widget(
        Paragraph::new(truncate_line(status, area.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn render_file_list(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let (content, scrollbar) = split_for_scrollbar(area);
    let viewport_rows = content.height as usize;
    let max_width = content.width as usize;
    let show_untracked = state.config.git.show_untracked;
    let rows = build_panel_rows(&state.git.file_entries, show_untracked);
    let total_rows = rows.len();
    let selected_row = git_selected_row_index(&state.git, &rows);
    let mut lines = Vec::new();

    for viewport_index in 0..viewport_rows {
        let Some(row) = git_row_at_viewport(&state.git, &rows, viewport_index) else {
            break;
        };
        let row_index = state.git.scroll_offset + viewport_index;
        lines.push(render_row_line(
            &row,
            row_index,
            selected_row,
            max_width,
            focused,
            theme,
        ));
    }

    if lines.is_empty() {
        let hint = if !state.workspace_meta.is_git_repo {
            "Open a git repository"
        } else if state.git.loading {
            "Refreshing…"
        } else {
            "No changes"
        };
        lines.push(Line::from(Span::styled(
            truncate_line(hint, max_width),
            theme.get(SemanticRole::Muted),
        )));
    }

    frame.render_widget(Clear, content);
    frame.render_widget(Paragraph::new(lines).style(chrome_style(theme)), content);
    if let Some(scrollbar_area) = scrollbar {
        render_vertical_scrollbar(
            frame,
            scrollbar_area,
            state.git.scroll_offset,
            total_rows,
            viewport_rows,
            focused,
            theme,
        );
    }
}

fn render_row_line(
    row: &GitPanelRow,
    row_index: usize,
    selected_row: Option<usize>,
    max_width: usize,
    focused: bool,
    theme: &ThemePalette,
) -> Line<'static> {
    match row {
        GitPanelRow::Header { label, count } => Line::from(vec![Span::styled(
            truncate_line(&format!("{label} ({count})"), max_width),
            theme.get(SemanticRole::Muted).add_modifier(Modifier::BOLD),
        )]),
        GitPanelRow::File { path, status } => {
            let selected = selected_row == Some(row_index);
            let mut style = theme.get(status.semantic_role());
            if selected {
                style = if focused {
                    theme.get(SemanticRole::Accent)
                } else {
                    theme.get(SemanticRole::Selection)
                };
                style = style.add_modifier(Modifier::BOLD);
            }

            let prefix = if selected { "▸ " } else { "  " };
            let badge = status.badge();
            Line::from(Span::styled(
                truncate_line(&format!("{prefix}{badge} {path}"), max_width),
                style,
            ))
        }
    }
}

fn pane_inner(area: Rect) -> Option<Rect> {
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    if inner.width == 0 || inner.height == 0 {
        None
    } else {
        Some(inner)
    }
}

fn git_list_area(inner: Rect) -> Rect {
    let list_top = inner.y.saturating_add(BRANCH_ROWS);
    let list_height =
        inner
            .height
            .saturating_sub(BRANCH_ROWS)
            .saturating_sub(if inner.height > BRANCH_ROWS {
                STATUS_ROWS
            } else {
                0
            });
    Rect {
        x: inner.x,
        y: list_top,
        width: inner.width,
        height: list_height,
    }
}

fn truncate_line(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_width {
        return text.to_string();
    }

    if max_width <= 1 {
        return "…".to_string();
    }

    chars[..max_width - 1].iter().collect::<String>() + "…"
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::config::ResolvedConfig;
    use crate::git::{GitFileEntry, GitFileStatus};
    use crate::layout::compute_layout;
    use crate::state::AppState;
    use crate::state::GitState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state_with_git() -> AppState {
        let mut state = AppState::from_startup(
            PathBuf::from("."),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        );
        state.workspace_meta.is_git_repo = true;
        state.git = GitState {
            branch: Some("main".to_string()),
            ahead: 1,
            behind: 0,
            file_entries: vec![
                GitFileEntry {
                    path: "src/main.rs".to_string(),
                    status: GitFileStatus::Modified,
                },
                GitFileEntry {
                    path: "src/new.rs".to_string(),
                    status: GitFileStatus::Added,
                },
            ],
            selected_path: Some("src/main.rs".to_string()),
            ..GitState::default()
        };
        state
    }

    #[test]
    fn render_git_pane_lists_grouped_files() {
        let state = test_state_with_git();
        let backend = TestBackend::new(80, 14);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_git_pane(frame, Rect::new(0, 0, 80, 14), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("Modified"));
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("Added"));
        assert!(content.contains("src/new.rs"));
    }

    #[test]
    fn git_interaction_at_selects_file_row() {
        let state = test_state_with_git();
        let area = Rect::new(0, 0, 80, 14);
        let inner = pane_inner(area).expect("inner");
        let list = git_list_area(inner);
        let row =
            git_interaction_at(&state, area, list.x, list.y.saturating_add(1)).expect("file row");
        assert!(row > 0);
    }
}
