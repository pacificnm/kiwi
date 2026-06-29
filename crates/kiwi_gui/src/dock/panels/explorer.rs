//! File explorer dock panel (SPEC-005 / SPEC-022).

use std::path::{Path, PathBuf};

use egui::{Color32, Response, RichText, Ui};
use kiwi_core::events::AppCommand;
use kiwi_core::file_tree::{file_type_category, FileNode, FileTreeState, VisibleTreeRow};
use kiwi_core::theme::SemanticRole;

use super::layout::{render_virtual_rows, selectable_label};
use crate::dock::context::PanelContext;

const ROW_HEIGHT: f32 = 18.0;
const TREE_INDENT: f32 = 16.0;
const FOLDER_EXPANDED: &str = "[▾]";
const FOLDER_COLLAPSED: &str = "[▸]";
const FOLDER_LOADING: &str = "[…]";
const FOLDER_ERROR: &str = "[!]";
const FILE_ICON: &str = "   ";

pub fn render(ui: &mut Ui, ctx: &mut PanelContext<'_>) {
    let visible = ctx.state.file_tree.visible_rows();
    if visible.is_empty() {
        ui.label(
            RichText::new("Open a repository to browse files")
                .color(ctx.theme.role(SemanticRole::Muted)),
        );
        return;
    }

    let rows: Vec<(VisibleTreeRow, FileNode)> = visible
        .into_iter()
        .filter_map(|row| {
            let node = ctx.state.file_tree.nodes.get(&row.path)?.clone();
            Some((row, node))
        })
        .collect();

    let mut scroll_offset = ctx.state.file_tree.scroll_offset;
    let layout = render_virtual_rows(
        ui,
        ROW_HEIGHT,
        rows.len(),
        &mut scroll_offset,
        |ui, row_index| {
            let (row, node) = &rows[row_index];
            render_row(ui, ctx, &row.path, row.depth, node);
        },
    );
    ctx.state.file_tree.scroll_offset = scroll_offset;
    ctx.state.viewport.file_tree_rows = layout.viewport_rows;
    ctx.state
        .file_tree
        .clamp_scroll_to_viewport(layout.max_start);
}

/// Keyboard shortcuts when the Explorer dock tab is focused ([`gui-keyboard-shortcuts.md`]).
pub fn keyboard_action(
    ctx: &egui::Context,
    state: &kiwi_core::state::AppState,
) -> Option<AppCommand> {
    if ctx.wants_keyboard_input() {
        return None;
    }

    if ctx.input(|input| input.modifiers.any()) {
        return None;
    }

    if ctx.input(|input| input.key_pressed(egui::Key::ArrowDown)) {
        return Some(AppCommand::FileTreeMoveSelection(1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::ArrowUp)) {
        return Some(AppCommand::FileTreeMoveSelection(-1));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::ArrowRight)) {
        let path = state.file_tree.selected.clone()?;
        return Some(AppCommand::FileTreeExpand(path));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::ArrowLeft)) {
        let path = state.file_tree.selected.clone()?;
        return Some(AppCommand::FileTreeCollapse(path));
    }
    if ctx.input(|input| input.key_pressed(egui::Key::R)) {
        return Some(AppCommand::FileTreeRefresh);
    }
    if ctx.input(|input| input.key_pressed(egui::Key::Space)) {
        return preview_selected_file(state);
    }
    if ctx.input(|input| input.key_pressed(egui::Key::Enter)) {
        return open_selected_in_editor(state);
    }

    None
}

fn render_row(
    ui: &mut Ui,
    ctx: &mut PanelContext<'_>,
    path: &PathBuf,
    depth: usize,
    node: &FileNode,
) {
    let selected = ctx.state.file_tree.selected.as_ref() == Some(path);
    let indent = depth as f32 * TREE_INDENT;
    let glyph = row_glyph(&ctx.state.file_tree, node);
    let chrome_color = row_chrome_color(ctx, selected);
    let name_color = row_name_color(ctx, node, selected);

    ui.horizontal(|ui| {
        ui.set_min_height(ROW_HEIGHT);
        ui.add_space(indent);
        let icon = ui
            .add(
                egui::Label::new(RichText::new(glyph).color(chrome_color).monospace()).sense(
                    if node.is_dir {
                        egui::Sense::click()
                    } else {
                        egui::Sense::hover()
                    },
                ),
            )
            .on_hover_cursor(if node.is_dir {
                egui::CursorIcon::PointingHand
            } else {
                egui::CursorIcon::default()
            });
        if node.is_dir && icon.clicked() {
            if node.expanded {
                let _ = (ctx.dispatch)(AppCommand::FileTreeCollapse(path.clone()));
            } else {
                let _ = (ctx.dispatch)(AppCommand::FileTreeExpand(path.clone()));
            }
        }

        let name_response = render_name_and_badge(ui, node, name_color, selected, ctx);
        if name_response.clicked() {
            let _ = (ctx.dispatch)(AppCommand::FileTreeSelect(path.clone()));
        }
        if name_response.double_clicked() {
            handle_row_open(ctx, path, node);
        }
    });
}

fn render_name_and_badge(
    ui: &mut Ui,
    node: &FileNode,
    name_color: Color32,
    selected: bool,
    ctx: &PanelContext<'_>,
) -> Response {
    if let Some(status) = node.git_status.filter(|_| !node.is_dir) {
        let status_color = if selected {
            name_color
        } else {
            ctx.theme.role(status.semantic_role())
        };
        let badge = format!(" {}", status.badge());
        let mut text = RichText::new(format!("{}{badge}", node.name)).color(status_color);
        if selected {
            text = text.strong();
        }
        selectable_label(ui, text)
    } else {
        let mut text = RichText::new(&node.name).color(name_color);
        if selected {
            text = text.strong();
        }
        selectable_label(ui, text)
    }
}

fn handle_row_open(ctx: &mut PanelContext<'_>, path: &Path, node: &FileNode) {
    if node.is_dir {
        let _ = (ctx.dispatch)(AppCommand::FileTreeExpand(path.to_path_buf()));
        return;
    }
    let _ = (ctx.dispatch)(AppCommand::FileTreeSelect(path.to_path_buf()));
    let _ = (ctx.dispatch)(AppCommand::PreviewFile {
        path: path.to_path_buf(),
        line: None,
    });
}

fn preview_selected_file(state: &kiwi_core::state::AppState) -> Option<AppCommand> {
    let path = state.file_tree.selected.clone()?;
    let node = state.file_tree.nodes.get(&path)?;
    if node.is_dir {
        return None;
    }
    Some(AppCommand::PreviewFile { path, line: None })
}

fn open_selected_in_editor(state: &kiwi_core::state::AppState) -> Option<AppCommand> {
    let path = state.file_tree.selected.clone()?;
    let node = state.file_tree.nodes.get(&path)?;
    if node.is_dir {
        return None;
    }
    Some(AppCommand::OpenEditor { path, line: None })
}

fn row_chrome_color(ctx: &PanelContext<'_>, selected: bool) -> Color32 {
    if selected {
        row_selection_color(ctx)
    } else {
        ctx.theme.role(SemanticRole::Fg)
    }
}

fn row_name_color(ctx: &PanelContext<'_>, node: &FileNode, selected: bool) -> Color32 {
    if selected {
        return row_selection_color(ctx);
    }
    if let Some(status) = node.git_status.filter(|_| !node.is_dir) {
        return ctx.theme.role(status.semantic_role());
    }
    ctx.theme.role(file_type_category(node).semantic_role())
}

fn row_selection_color(ctx: &PanelContext<'_>) -> Color32 {
    ctx.theme.role(SemanticRole::Accent)
}

fn row_glyph(tree: &FileTreeState, node: &FileNode) -> &'static str {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::events::AppCommand;
    use kiwi_core::file_tree::{DirectoryEntry, FileTreeState};
    use kiwi_core::git::GitFileStatus;
    use kiwi_core::state::{AppState, ViewportMetrics};
    use kiwi_core::theme::{load_theme_with_capabilities, SemanticRole, TerminalCapabilities};

    use super::*;
    use crate::dock::PtySurfaceState;
    use crate::theme::GuiTheme;

    fn test_panel() -> (AppState, GuiTheme) {
        let config = ResolvedConfig::default();
        let theme_palette =
            load_theme_with_capabilities(&config.theme, TerminalCapabilities::TrueColor)
                .expect("theme");
        let state = AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            false,
            config.clone(),
            theme_palette,
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        );
        let gui_theme = GuiTheme::from_palette(&state.theme, &config.gui);
        (state, gui_theme)
    }

    fn panel_context<'a>(
        state: &'a mut AppState,
        theme: &'a GuiTheme,
        dispatch: &'a mut dyn FnMut(AppCommand) -> bool,
        pty_surface: &'a mut PtySurfaceState,
    ) -> PanelContext<'a> {
        PanelContext {
            state,
            theme,
            dispatch,
            pty_surface,
            focused_dock_tab: None,
        }
    }

    fn tree_with_readme() -> FileTreeState {
        let mut tree = FileTreeState::at_root(PathBuf::from("/tmp/kiwi"));
        let root = tree.root.clone();
        tree.expand(&root).expect("expand");
        tree.apply_children_loaded(
            &root,
            vec![DirectoryEntry {
                path: root.join("README.md"),
                name: "README.md".to_string(),
                is_dir: false,
            }],
            None,
        );
        tree.select(root.join("README.md"));
        tree
    }

    #[test]
    fn row_glyph_uses_bracketed_folder_icons() {
        let mut tree = FileTreeState::at_root(PathBuf::from("/tmp/kiwi"));
        let root = tree.root.clone();
        assert_eq!(row_glyph(&tree, &tree.nodes[&root]), FOLDER_COLLAPSED);
        tree.expand(&root).expect("expand");
        assert_eq!(row_glyph(&tree, &tree.nodes[&root]), FOLDER_LOADING);
        tree.apply_children_loaded(&root, vec![], None);
        assert_eq!(row_glyph(&tree, &tree.nodes[&root]), FOLDER_EXPANDED);
    }

    #[test]
    fn git_status_color_overrides_file_type_color() {
        let (mut state, theme) = test_panel();
        state.file_tree = tree_with_readme();
        let path = state.file_tree.root.join("README.md");
        state
            .file_tree
            .nodes
            .get_mut(&path)
            .expect("readme")
            .git_status = Some(GitFileStatus::Modified);

        let mut noop = |_cmd: AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let node = state.file_tree.nodes.get(&path).expect("node").clone();
        let ctx = panel_context(&mut state, &theme, &mut noop, &mut pty_surface);
        let color = row_name_color(&ctx, &node, false);
        assert_eq!(color, theme.role(SemanticRole::GitModified));
    }

    #[test]
    fn selected_row_uses_accent_color() {
        let (mut state, theme) = test_panel();
        state.file_tree = tree_with_readme();
        let path = state.file_tree.root.join("README.md");
        state.file_tree.select(path.clone());

        let mut noop = |_cmd: AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let node = state.file_tree.nodes.get(&path).expect("node").clone();
        let ctx = panel_context(&mut state, &theme, &mut noop, &mut pty_surface);
        assert_eq!(
            row_name_color(&ctx, &node, true),
            theme.role(SemanticRole::Accent)
        );
    }

    #[test]
    fn preview_command_skips_directories() {
        let (mut state, theme) = test_panel();
        state.file_tree = tree_with_readme();
        state.file_tree.select(state.file_tree.root.clone());

        let mut noop = |_cmd: AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let _ctx = panel_context(&mut state, &theme, &mut noop, &mut pty_surface);
        assert!(preview_selected_file(&state).is_none());
    }

    #[test]
    fn preview_command_returns_file_path() {
        let (mut state, theme) = test_panel();
        state.file_tree = tree_with_readme();
        let path = state.file_tree.root.join("README.md");

        let mut noop = |_cmd: AppCommand| false;
        let mut pty_surface = PtySurfaceState::default();
        let _ctx = panel_context(&mut state, &theme, &mut noop, &mut pty_surface);
        assert_eq!(
            preview_selected_file(&state),
            Some(AppCommand::PreviewFile { path, line: None })
        );
    }
}
