use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem};
use ratatui::Frame;

use crate::github::GhContextMenuState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn render_github_context_menu(
    frame: &mut Frame<'_>,
    bounds: Rect,
    theme: &ThemePalette,
    menu: &GhContextMenuState,
) {
    if bounds.width < 8 || bounds.height < 4 || menu.items.is_empty() {
        return;
    }

    let width = menu.menu_width().min(bounds.width);
    let height = menu.menu_height().min(bounds.height);
    let mut x = menu.anchor_x;
    let mut y = menu.anchor_y;

    if x.saturating_add(width) > bounds.x.saturating_add(bounds.width) {
        x = bounds.x.saturating_add(bounds.width).saturating_sub(width);
    }
    if y.saturating_add(height) > bounds.y.saturating_add(bounds.height) {
        y = bounds
            .y
            .saturating_add(bounds.height)
            .saturating_sub(height);
    }
    x = x.max(bounds.x);
    y = y.max(bounds.y);

    let area = Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title("Actions")
        .borders(Borders::ALL)
        .border_style(theme.get(SemanticRole::Accent));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let items: Vec<ListItem<'_>> = menu
        .items
        .iter()
        .enumerate()
        .map(|(index, action)| {
            let mut style = theme.get(SemanticRole::Fg);
            if index == menu.cursor {
                style = style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
            }
            ListItem::new(Line::from(Span::styled(action.label(), style)))
        })
        .collect();

    frame.render_widget(List::new(items), inner);
}

#[must_use]
pub fn github_context_menu_rect(bounds: Rect, menu: &GhContextMenuState) -> Rect {
    let width = menu.menu_width().min(bounds.width);
    let height = menu.menu_height().min(bounds.height);
    let mut x = menu.anchor_x;
    let mut y = menu.anchor_y;

    if x.saturating_add(width) > bounds.x.saturating_add(bounds.width) {
        x = bounds.x.saturating_add(bounds.width).saturating_sub(width);
    }
    if y.saturating_add(height) > bounds.y.saturating_add(bounds.height) {
        y = bounds
            .y
            .saturating_add(bounds.height)
            .saturating_sub(height);
    }
    x = x.max(bounds.x);
    y = y.max(bounds.y);

    Rect {
        x,
        y,
        width,
        height,
    }
}

#[must_use]
pub fn github_context_menu_item_at(
    bounds: Rect,
    menu: &GhContextMenuState,
    column: u16,
    row: u16,
) -> Option<usize> {
    let area = github_context_menu_rect(bounds, menu);
    if column < area.x
        || column >= area.x.saturating_add(area.width)
        || row < area.y
        || row >= area.y.saturating_add(area.height)
    {
        return None;
    }

    let inner = Block::default().borders(Borders::ALL).inner(area);
    if column < inner.x
        || column >= inner.x.saturating_add(inner.width)
        || row < inner.y
        || row >= inner.y.saturating_add(inner.height)
    {
        return None;
    }

    let index = usize::from(row.saturating_sub(inner.y));
    if index < menu.items.len() {
        Some(index)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::{GhContextMenuState, GhContextTarget};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn render_github_context_menu_draws_without_panic() {
        let menu = GhContextMenuState::new(GhContextTarget::Issue { list_index: 0 }, 10, 5);
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("terminal");
        terminal
            .draw(|frame| {
                render_github_context_menu(
                    frame,
                    frame.area(),
                    &crate::theme::loader::load_theme_with_capabilities(
                        &crate::config::ResolvedConfig::default().theme,
                        crate::theme::capabilities::TerminalCapabilities::TrueColor,
                    )
                    .expect("theme"),
                    &menu,
                );
            })
            .expect("draw");
    }

    #[test]
    fn github_context_menu_item_at_maps_row_to_action() {
        let menu = GhContextMenuState::new(GhContextTarget::Issue { list_index: 0 }, 10, 5);
        let bounds = Rect::new(0, 0, 80, 24);
        let area = github_context_menu_rect(bounds, &menu);
        let inner = Block::default().borders(Borders::ALL).inner(area);
        let index = github_context_menu_item_at(bounds, &menu, inner.x, inner.y).expect("item");
        assert_eq!(index, 0);
    }
}
