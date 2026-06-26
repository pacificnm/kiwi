use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::file_tree::{file_type_category, FileTreeState};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::scrollbar::{render_vertical_scrollbar, split_for_scrollbar};

pub const TREE_INDENT_CHARS: usize = 2;
pub const TREE_ICON_WIDTH: usize = 3;

const FOLDER_EXPANDED: &str = "[▾]";
const FOLDER_COLLAPSED: &str = "[▸]";
const FOLDER_LOADING: &str = "[…]";
const FOLDER_ERROR: &str = "[!]";
const FILE_ICON: &str = "   ";

pub fn tree_icon_x(depth: usize) -> usize {
    depth.saturating_mul(TREE_INDENT_CHARS)
}

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

    let (content, scrollbar) = split_for_scrollbar(inner);
    let viewport_rows = content.height as usize;
    let total_rows = state.file_tree.visible_rows().len();
    let mut lines = Vec::new();
    for viewport_index in 0..viewport_rows {
        let Some(line) = render_row_line(
            state,
            theme,
            viewport_index,
            content.width as usize,
            focused,
        ) else {
            break;
        };
        lines.push(line);
    }

    frame.render_widget(Paragraph::new(lines), content);
    if let Some(scrollbar_area) = scrollbar {
        render_vertical_scrollbar(
            frame,
            scrollbar_area,
            state.file_tree.scroll_offset,
            total_rows,
            viewport_rows,
            focused,
            theme,
        );
    }
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
    let icon_x = tree_icon_x(tree_row.depth);

    if node.is_dir && local_x >= icon_x && local_x < icon_x.saturating_add(TREE_ICON_WIDTH) {
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
    let chrome_style = row_chrome_style(theme, selected, focused);
    let name_style = row_name_style(theme, node, selected, focused);

    let indent = "  ".repeat(tree_row.depth);
    let glyph = row_glyph(&state.file_tree, node);
    let prefix = format!("{indent}{glyph}");

    if let Some(status) = node.git_status.filter(|_| !node.is_dir) {
        let status_style = if selected {
            name_style
        } else {
            theme.get(status.semantic_role())
        };
        let badge = format!(" {}", status.badge());
        let name_budget = max_width.saturating_sub(prefix.chars().count() + badge.chars().count());
        let name = truncate_line(&node.name, name_budget);

        return Some(Line::from(vec![
            Span::styled(prefix, chrome_style),
            Span::styled(name, status_style),
            Span::styled(badge, status_style),
        ]));
    }

    let name_budget = max_width.saturating_sub(prefix.chars().count());
    let name = truncate_line(&node.name, name_budget);

    Some(Line::from(vec![
        Span::styled(prefix, chrome_style),
        Span::styled(name, name_style),
    ]))
}

fn row_chrome_style(theme: &ThemePalette, selected: bool, focused: bool) -> ratatui::style::Style {
    if selected {
        return row_selection_style(theme, focused);
    }
    theme.get(SemanticRole::Fg)
}

fn row_name_style(
    theme: &ThemePalette,
    node: &crate::file_tree::FileNode,
    selected: bool,
    focused: bool,
) -> ratatui::style::Style {
    if selected {
        return row_selection_style(theme, focused);
    }
    if let Some(status) = node.git_status.filter(|_| !node.is_dir) {
        return theme.get(status.semantic_role());
    }
    theme.get(file_type_category(node).semantic_role())
}

fn row_selection_style(theme: &ThemePalette, focused: bool) -> ratatui::style::Style {
    let style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Selection)
    };
    style.add_modifier(Modifier::BOLD)
}

fn row_glyph(tree: &FileTreeState, node: &crate::file_tree::FileNode) -> &'static str {
    if node.load_error.is_some() {
        return FOLDER_ERROR;
    }
    if node.is_dir {
        if tree.loading.contains(&node.path) {
            return FOLDER_LOADING;
        }
        if node.expanded {
            return FOLDER_EXPANDED;
        }
        return FOLDER_COLLAPSED;
    }
    FILE_ICON
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
    use crate::theme::SemanticRole;
    use ratatui::style::Color;

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
                DirectoryEntry {
                    path: root.join("main.rs"),
                    name: "main.rs".to_string(),
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
    fn interaction_on_icon_expands_directory() {
        let state = test_state_with_tree();
        let area = Rect::new(0, 0, 30, 8);
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);
        let root = state.file_tree.root.clone();
        for offset in 0..TREE_ICON_WIDTH {
            let column = inner.x + u16::try_from(offset).expect("column");
            let action = file_tree_interaction_at(&state, area, column, inner.y);
            assert_eq!(action, Some(FileTreeMouseAction::Expand(root.clone())));
        }
    }

    #[test]
    fn interaction_on_name_selects_entry() {
        let mut state = test_state_with_tree();
        let root = state.file_tree.root.clone();
        state.file_tree.expand(&root).expect("expand");
        let area = Rect::new(0, 0, 30, 8);
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);
        let name_column =
            inner.x + u16::try_from(tree_icon_x(1) + TREE_ICON_WIDTH).expect("name column");
        let action = file_tree_interaction_at(&state, area, name_column, inner.y + 1);
        assert_eq!(action, Some(FileTreeMouseAction::Select(root.join("src"))));
    }

    #[test]
    fn source_file_uses_theme_file_type_color() {
        let mut state = test_state_with_tree();
        let root = state.file_tree.root.clone();
        state.file_tree.expand(&root).expect("expand");

        let area = Rect::new(0, 0, 40, 8);
        let block = Block::default().title("Files").borders(Borders::ALL);
        let inner = block.inner(area);
        let line = render_row_line(&state, &state.theme, 3, inner.width as usize, true)
            .expect("main.rs row");

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[1].content, "main.rs");
        assert_eq!(
            line.spans[1].style.fg,
            state.theme.get(SemanticRole::FileSource).fg
        );
        assert_eq!(
            state.theme.get(SemanticRole::FileSource).fg,
            Some(Color::Rgb(158, 206, 106))
        );
    }

    #[test]
    fn git_status_color_overrides_file_type_color() {
        let mut state = test_state_with_tree();
        let root = state.file_tree.root.clone();
        state.file_tree.expand(&root).expect("expand");
        state
            .file_tree
            .nodes
            .get_mut(&root.join("main.rs"))
            .expect("main.rs")
            .git_status = Some(GitFileStatus::Modified);

        let area = Rect::new(0, 0, 40, 8);
        let block = Block::default().title("Files").borders(Borders::ALL);
        let inner = block.inner(area);
        let line = render_row_line(&state, &state.theme, 3, inner.width as usize, true)
            .expect("main.rs row");

        assert_eq!(line.spans.len(), 3);
        assert_eq!(
            line.spans[1].style.fg,
            state.theme.get(SemanticRole::GitModified).fg
        );
    }

    #[test]
    fn selected_file_row_uses_accent_highlight() {
        let mut state = test_state_with_tree();
        let root = state.file_tree.root.clone();
        state.file_tree.expand(&root).expect("expand");
        state.file_tree.select(root.join("main.rs"));

        let area = Rect::new(0, 0, 40, 8);
        let block = Block::default().title("Files").borders(Borders::ALL);
        let inner = block.inner(area);
        let line = render_row_line(&state, &state.theme, 3, inner.width as usize, true)
            .expect("main.rs row");

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[1].content, "main.rs");
        assert_eq!(
            line.spans[1].style.fg,
            state.theme.get(SemanticRole::Accent).fg
        );
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn row_glyph_uses_bracketed_folder_icons() {
        let mut state = test_state_with_tree();
        let root = state.file_tree.root.clone();
        let area = Rect::new(0, 0, 40, 8);
        let block = Block::default().title("Files").borders(Borders::ALL);
        let inner = block.inner(area);

        let collapsed =
            render_row_line(&state, &state.theme, 0, inner.width as usize, true).expect("root row");
        assert!(collapsed.spans[0].content.contains("[▸]"));

        state.file_tree.expand(&root).expect("expand");
        let expanded = render_row_line(&state, &state.theme, 0, inner.width as usize, true)
            .expect("expanded root row");
        assert!(expanded.spans[0].content.contains("[▾]"));
    }
}
