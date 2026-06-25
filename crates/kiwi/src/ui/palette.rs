use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::commands::{command_available, COMMANDS, MAX_VISIBLE_MATCHES};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn render_palette_pane(
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

    let title = if let Some(prompt) = &state.palette.prompt {
        prompt.title()
    } else {
        "Commands".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut lines = Vec::new();
    let prompt = if state.palette.open {
        format!("> {}", state.palette.input)
    } else {
        "Ctrl+P for commands".to_string()
    };
    lines.push(Line::from(Span::styled(
        truncate_line(&prompt, inner.width as usize),
        theme.get(SemanticRole::Fg),
    )));

    if state.palette.open {
        if let Some(prompt) = &state.palette.prompt {
            lines.push(Line::from(Span::styled(
                truncate_line(prompt.hint(), inner.width as usize),
                theme.get(SemanticRole::Muted),
            )));
        } else {
            let visible_rows = inner
                .height
                .saturating_sub(1)
                .min(MAX_VISIBLE_MATCHES as u16) as usize;
            for match_index in 0..visible_rows {
                let Some(line) = render_match_line(state, theme, match_index, inner.width as usize)
                else {
                    break;
                };
                lines.push(line);
            }
        }
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_match_line(
    state: &AppState,
    theme: &ThemePalette,
    match_index: usize,
    max_width: usize,
) -> Option<Line<'static>> {
    let registry_index = *state.palette.matches.get(match_index)?;
    let command = COMMANDS.get(registry_index)?;
    let available = command_available(state, command);
    let selected = state.palette.selected == match_index;

    let mut style = theme.get(SemanticRole::Fg);
    if !available {
        style = theme.get(SemanticRole::Muted);
    }
    if selected {
        style = style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
    }

    let shortcut = command
        .shortcut
        .map(|value| format!(" ({value})"))
        .unwrap_or_default();
    let text = format!("{}{}", command.title, shortcut);
    Some(Line::from(Span::styled(
        truncate_line(&text, max_width),
        style,
    )))
}

pub fn palette_match_at(state: &AppState, area: Rect, column: u16, row: u16) -> Option<usize> {
    if !state.palette.open || state.palette.prompt.is_some() || area.width == 0 || area.height == 0
    {
        return None;
    }

    if column < area.x
        || column >= area.x.saturating_add(area.width)
        || row < area.y
        || row >= area.y.saturating_add(area.height)
    {
        return None;
    }

    let inner_y = row.saturating_sub(area.y + 1);
    if inner_y == 0 {
        return None;
    }

    let match_index = usize::from(inner_y.saturating_sub(1));
    if match_index >= state.palette.matches.len() {
        return None;
    }

    Some(match_index)
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
    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::state::{AppState, PalettePrompt};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            std::path::PathBuf::from("."),
            true,
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
    fn palette_match_at_maps_rows_below_prompt() {
        let mut state = test_state();
        state.palette.open = true;
        state.palette.matches = vec![0, 1];
        let area = Rect::new(0, 0, 30, 5);
        assert_eq!(palette_match_at(&state, area, 2, 2), Some(0));
        assert_eq!(palette_match_at(&state, area, 2, 3), Some(1));
    }

    #[test]
    fn palette_match_at_disabled_in_prompt_mode() {
        let mut state = test_state();
        state.palette.open = true;
        state.palette.prompt = Some(PalettePrompt::GitHubIssueComment { number: 1 });
        state.palette.matches = vec![0];
        let area = Rect::new(0, 0, 30, 5);
        assert_eq!(palette_match_at(&state, area, 2, 3), None);
    }
}
