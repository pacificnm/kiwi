use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn render_notifications(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    if let Some(modal) = &state.notifications.modal {
        render_modal(frame, area, &state.theme, &modal.title, &modal.message);
        return;
    }

    if let Some(message) = &state.notifications.toast.message {
        render_toast(frame, area, &state.theme, message);
    }
}

fn render_modal(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &ThemePalette,
    title: &str,
    message: &str,
) {
    let width = area.width.clamp(24, 72);
    let height = area.height.clamp(5, 12);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    let modal_area = Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme.get(SemanticRole::Accent))
        .style(chrome_style(theme));
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    frame.render_widget(
        Paragraph::new(message)
            .wrap(Wrap { trim: true })
            .style(chrome_style(theme)),
        inner,
    );
}

fn render_toast(frame: &mut Frame<'_>, area: Rect, theme: &ThemePalette, message: &str) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let toast_height = 1u16;
    let toast_area = Rect {
        x: area.x,
        y: area.y.saturating_sub(toast_height),
        width: area.width,
        height: toast_height.min(area.height),
    };

    let style = chrome_style(theme).add_modifier(Modifier::ITALIC);
    frame.render_widget(
        Paragraph::new(Line::from(message))
            .alignment(Alignment::Center)
            .style(style),
        toast_area,
    );
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
