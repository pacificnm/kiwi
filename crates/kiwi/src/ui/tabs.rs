use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub const TAB_SEPARATOR: &str = " | ";
pub const TAB_LEADING_PAD: &str = "  ";

pub fn tab_index_at_x(local_x: u16, labels: &[&str]) -> Option<usize> {
    let pad = TAB_LEADING_PAD.len();
    let x = local_x as usize;
    if x < pad {
        return None;
    }
    let x = x - pad;
    let mut cursor = 0;

    for (index, label) in labels.iter().enumerate() {
        let start = cursor;
        let end = cursor + label.len();
        if x >= start && x < end {
            return Some(index);
        }

        cursor = end;
        if index + 1 < labels.len() {
            cursor += TAB_SEPARATOR.len();
        }
    }

    None
}

pub fn tab_bar_line(tabs: &[&'static str], selected: usize, theme: &ThemePalette) -> Line<'static> {
    tab_bar_line_str(tabs, selected, theme)
}

pub fn tab_bar_line_str<'a>(
    tabs: &[&'a str],
    selected: usize,
    theme: &ThemePalette,
) -> Line<'a> {
    let mut spans = Vec::new();
    spans.push(Span::styled(
        TAB_LEADING_PAD,
        theme.get(SemanticRole::Muted),
    ));

    for (index, label) in tabs.iter().enumerate() {
        if index > 0 {
            spans.push(separator_span(theme));
        }

        let mut style = if index == selected {
            theme.get(SemanticRole::Accent).add_modifier(Modifier::BOLD)
        } else {
            theme.get(SemanticRole::Muted)
        };

        if index == selected {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        spans.push(Span::styled(*label, style));
    }

    Line::from(spans)
}

pub(crate) fn separator_span(theme: &ThemePalette) -> Span<'static> {
    Span::styled(TAB_SEPARATOR, theme.get(SemanticRole::Muted))
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use crate::config::ResolvedConfig;
    use crate::navigation::{LeftNavTab, MainTab, LEFT_TAB_LABELS, MAIN_TAB_LABELS};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_theme() -> ThemePalette {
        load_theme_with_capabilities(
            &ResolvedConfig::default().theme,
            TerminalCapabilities::TrueColor,
        )
        .expect("theme")
    }

    #[test]
    fn tab_index_at_x_selects_label_not_separator() {
        assert_eq!(tab_index_at_x(0, &LEFT_TAB_LABELS), None);
        assert_eq!(tab_index_at_x(1, &LEFT_TAB_LABELS), None);
        assert_eq!(tab_index_at_x(2, &LEFT_TAB_LABELS), Some(0));
        assert_eq!(tab_index_at_x(6, &LEFT_TAB_LABELS), Some(0));
        assert_eq!(tab_index_at_x(7, &LEFT_TAB_LABELS), None);
        assert_eq!(tab_index_at_x(10, &LEFT_TAB_LABELS), Some(1));
        assert_eq!(tab_index_at_x(12, &MAIN_TAB_LABELS), Some(1));
    }

    #[test]
    fn left_tab_labels_match_design() {
        assert_eq!(LEFT_TAB_LABELS[0], LeftNavTab::Files.label());
        assert_eq!(LEFT_TAB_LABELS[3], LeftNavTab::Search.label());
    }

    #[test]
    fn main_tab_labels_match_design() {
        assert_eq!(MAIN_TAB_LABELS[0], MainTab::Agent.label());
        assert_eq!(MAIN_TAB_LABELS[6], MainTab::Logs.label());
    }

    #[test]
    fn active_tab_uses_accent_bold_underline() {
        let theme = test_theme();
        let line = tab_bar_line(&MAIN_TAB_LABELS, 0, &theme);
        let agent = &line.spans[1];

        assert_eq!(agent.content, "Agent");
        assert!(agent.style.add_modifier.contains(Modifier::BOLD));
        assert!(agent.style.add_modifier.contains(Modifier::UNDERLINED));
        assert_eq!(agent.style.fg, Some(Color::Rgb(122, 162, 247)));
    }

    #[test]
    fn inactive_tab_uses_muted_style() {
        let theme = test_theme();
        let line = tab_bar_line(&MAIN_TAB_LABELS, 0, &theme);
        let issues = &line.spans[3];

        assert_eq!(issues.content, "Issues");
        assert!(!issues.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(issues.style.fg, Some(Color::Rgb(86, 95, 137)));
    }
}
