use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::file_tree::FileTreeState;
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTreeMouseAction {
    Select(std::path::PathBuf),
    Expand(std::path::PathBuf),
    Collapse(std::path::PathBuf),
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn file_tree_viewport_rows(area: Rect) -> usize {
    let block = Block::default().borders(Borders::ALL);
    block.inner(area).height.saturating_sub(0) as usize
}

pub fn render_file_tree_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
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
        .title("Files")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let viewport_rows = inner.height as usize;
    let mut lines = Vec::new();
    for viewport_index in 0..viewport_rows {
        let Some(line) =
            render_row_line(state, theme, viewport_index, inner.width as usize, focused)
        else {
            break;
        };
        lines.push(line);
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn file_tree_interaction_at(
    state: &AppState,
    area: Rect,
    column: u16,
    row: u16,
) -> Option<FileTreeMouseAction> {
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

    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    if inner.width == 0 || inner.height == 0 {
        return None;
    }

    if column < inner.x
        || column >= inner.x.saturating_add(inner.width)
        || row < inner.y
        || row >= inner.y.saturating_add(inner.height)
    {
        return None;
    }

    let viewport_index = usize::from(row.saturating_sub(inner.y));
    let tree_row = state.file_tree.row_at_viewport_index(viewport_index)?;
    let node = state.file_tree.nodes.get(&tree_row.path)?;
    let local_x = usize::from(column.saturating_sub(inner.x));
    let chevron_x = tree_row.depth.saturating_mul(2);

    if node.is_dir && local_x == chevron_x {
        if node.expanded {
            return Some(FileTreeMouseAction::Collapse(tree_row.path.clone()));
        }
        return Some(FileTreeMouseAction::Expand(tree_row.path.clone()));
    }

    Some(FileTreeMouseAction::Select(tree_row.path.clone()))
}

fn render_row_line(
    state: &AppState,
    theme: &ThemePalette,
    viewport_index: usize,
    max_width: usize,
    focused: bool,
) -> Option<Line<'static>> {
    let tree_row = state.file_tree.row_at_viewport_index(viewport_index)?;
    let node = state.file_tree.nodes.get(&tree_row.path)?;
    let selected = state.file_tree.selected.as_ref() == Some(&tree_row.path);

    let mut style = theme.get(SemanticRole::Fg);
    if selected {
        style = if focused {
            theme.get(SemanticRole::Accent)
        } else {
            theme.get(SemanticRole::Fg)
        };
        style = style.add_modifier(Modifier::BOLD);
    }

    let indent = "  ".repeat(tree_row.depth);
    let glyph = row_glyph(&state.file_tree, node);

    if node.is_dir || node.git_status.is_none() {
        let label = truncate_line(&format!("{indent}{glyph}{}", node.name), max_width);
        return Some(Line::from(Span::styled(label, style)));
    }

    let status = node.git_status.expect("checked above");
    let status_style = theme.get(status.semantic_role());
    let prefix = format!("{indent}{glyph}");
    let badge = format!(" {}", status.badge());
    let name_budget = max_width.saturating_sub(prefix.chars().count() + badge.chars().count());
    let name = truncate_line(&node.name, name_budget);

    Some(Line::from(vec![
        Span::styled(prefix, style),
        Span::styled(name, status_style),
        Span::styled(badge, status_style),
    ]))
}

fn row_glyph(tree: &FileTreeState, node: &crate::file_tree::FileNode) -> &'static str {
    if node.load_error.is_some() {
        return "! ";
    }
    if node.is_dir {
        if tree.loading.contains(&node.path) {
            return "… ";
        }
        if node.expanded {
            return "▾ ";
        }
        return "▸ ";
    }
    "  "
}

fn truncate_line(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if text.chars().count() <= max_width {
        return text.to_string();
    }
    if max_width == 1 {
        return "…".to_string();
    }
    text.chars().take(max_width - 1).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::ResolvedConfig;
    use crate::file_tree::DirectoryEntry;
    use crate::git::GitFileStatus;
    use crate::layout::compute_layout;
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state_with_tree() -> AppState {
        let mut state = AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            false,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        );
        let root = state.file_tree.root.clone();
        state.file_tree.expand(&root).expect("expand");
        state.file_tree.apply_children_loaded(
            &root,
            vec![
                DirectoryEntry {
                    path: root.join("src"),
                    name: "src".to_string(),
                    is_dir: true,
                },
                DirectoryEntry {
                    path: root.join("README.md"),
                    name: "README.md".to_string(),
                    is_dir: false,
                },
            ],
            None,
        );
        state.file_tree.collapse(&root);
        state.file_tree.ensure_selection();
        state
    }

    #[test]
    fn modified_file_renders_git_badge() {
        let mut state = test_state_with_tree();
        let root = state.file_tree.root.clone();
        state.file_tree.expand(&root).expect("expand");
        state
            .file_tree
            .nodes
            .get_mut(&root.join("README.md"))
            .expect("readme")
            .git_status = Some(GitFileStatus::Modified);

        let area = Rect::new(0, 0, 40, 8);
        let block = Block::default().title("Files").borders(Borders::ALL);
        let inner = block.inner(area);
        let line = render_row_line(&state, &state.theme, 2, inner.width as usize, true)
            .expect("readme row");
        assert_eq!(line.spans.len(), 3);
        assert!(line.spans[2].content.contains('M'));
    }

    #[test]
    fn interaction_on_chevron_expands_directory() {
        let state = test_state_with_tree();
        let area = Rect::new(0, 0, 30, 8);
        let action = file_tree_interaction_at(&state, area, area.x + 1, area.y + 1);
        assert_eq!(
            action,
            Some(FileTreeMouseAction::Expand(state.file_tree.root.clone()))
        );
    }

    #[test]
    fn interaction_on_name_selects_entry() {
        let mut state = test_state_with_tree();
        let root = state.file_tree.root.clone();
        state.file_tree.expand(&root).expect("expand");
        let area = Rect::new(0, 0, 30, 8);
        let child_row = area.y + 2;
        let action = file_tree_interaction_at(&state, area, area.x + 4, child_row);
        assert_eq!(action, Some(FileTreeMouseAction::Select(root.join("src"))));
    }
}
