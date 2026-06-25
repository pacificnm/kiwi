use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn render_label_picker_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &ThemePalette,
    state: &AppState,
) {
    let Some(picker) = &state.github.label_picker else {
        return;
    };

    if area.width < 20 || area.height < 8 {
        return;
    }

    let overlay_width = area.width.clamp(24, 60);
    let overlay_height = area.height.clamp(8, 20);
    let overlay = Rect {
        x: area.x + (area.width.saturating_sub(overlay_width)) / 2,
        y: area.y + (area.height.saturating_sub(overlay_height)) / 2,
        width: overlay_width,
        height: overlay_height,
    };

    frame.render_widget(Clear, overlay);

    let title = if picker.applying {
        format!("Adding labels to #{}", picker.issue_number)
    } else if picker.loading {
        format!("Loading labels for #{}", picker.issue_number)
    } else {
        format!("Add labels to #{}", picker.issue_number)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme.get(SemanticRole::Accent));
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    if picker.loading {
        frame.render_widget(
            Paragraph::new("Loading repository labels…").style(theme.get(SemanticRole::Muted)),
            inner,
        );
        return;
    }

    if let Some(error) = &picker.error {
        let mut lines = vec![Line::from(error.clone())];
        lines.push(Line::from(Span::styled(
            "Esc cancel",
            theme.get(SemanticRole::Muted),
        )));
        frame.render_widget(Paragraph::new(lines), inner);
        return;
    }

    if picker.labels.is_empty() {
        frame.render_widget(
            Paragraph::new("No repository labels found.").style(theme.get(SemanticRole::Muted)),
            inner,
        );
        return;
    }

    let list_height = inner.height.saturating_sub(2);
    let list_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: list_height,
    };

    let items: Vec<ListItem<'_>> = picker
        .labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let selected = picker.selected.get(index).copied().unwrap_or(false);
            let marker = if selected { "[x]" } else { "[ ]" };
            let mut line = format!("{marker} {}", label.name);
            if !label.description.is_empty() {
                line.push_str(" — ");
                line.push_str(&label.description);
            }
            let mut style = theme.get(SemanticRole::Fg);
            if index == picker.cursor {
                style = style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
            } else if !selected {
                style = theme.get(SemanticRole::Muted);
            }
            ListItem::new(truncate_line(&line, list_area.width as usize)).style(style)
        })
        .collect();

    frame.render_widget(List::new(items), list_area);

    let status = if picker.applying {
        "Applying labels…"
    } else {
        "Space toggle · Enter apply · Esc cancel"
    };
    let status_area = Rect {
        x: inner.x,
        y: inner.y.saturating_add(list_height),
        width: inner.width,
        height: inner.height.saturating_sub(list_height),
    };
    frame.render_widget(
        Paragraph::new(status).style(theme.get(SemanticRole::Muted)),
        status_area,
    );
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
    use super::*;
    use crate::config::ResolvedConfig;
    use crate::github::{apply_label_picker_load, LabelPickerState, RepoLabelsLoadResult};
    use crate::layout::compute_layout;
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn test_state_with_picker() -> AppState {
        let mut state = AppState::from_startup(
            std::path::PathBuf::from("."),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        );
        let mut picker = LabelPickerState::new(7, vec!["bug".to_string()]);
        apply_label_picker_load(
            &mut picker,
            RepoLabelsLoadResult {
                labels: vec![
                    crate::github::labels::RepoLabel {
                        name: "bug".to_string(),
                        description: String::new(),
                    },
                    crate::github::labels::RepoLabel {
                        name: "docs".to_string(),
                        description: "Documentation".to_string(),
                    },
                ],
                error: None,
            },
            &["bug".to_string()],
        );
        picker.cursor = 1;
        state.github.label_picker = Some(picker);
        state
    }

    #[test]
    fn render_label_picker_overlay_draws_without_panic() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("terminal");
        let state = test_state_with_picker();
        terminal
            .draw(|frame| {
                render_label_picker_overlay(frame, frame.area(), &state.theme, &state);
            })
            .expect("draw");
    }
}
