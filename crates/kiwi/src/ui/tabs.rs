use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub fn tab_bar_line(
    tabs: &[&'static str],
    selected: usize,
    theme: &ThemePalette,
) -> Line<'static> {
    let separator = Span::styled(" | ", theme.get(SemanticRole::Muted));
    let mut spans = Vec::new();

    for (index, label) in tabs.iter().enumerate() {
        if index > 0 {
            spans.push(separator.clone());
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
    fn left_tab_labels_match_design() {
        assert_eq!(LEFT_TAB_LABELS[0], LeftNavTab::Files.label());
        assert_eq!(LEFT_TAB_LABELS[4], LeftNavTab::Search.label());
    }

    #[test]
    fn main_tab_labels_match_design() {
        assert_eq!(MAIN_TAB_LABELS[0], MainTab::Agent.label());
        assert_eq!(MAIN_TAB_LABELS[5], MainTab::Logs.label());
    }

    #[test]
    fn active_tab_uses_accent_bold_underline() {
        let theme = test_theme();
        let line = tab_bar_line(&MAIN_TAB_LABELS, 0, &theme);
        let agent = &line.spans[0];

        assert_eq!(agent.content, "Agent");
        assert!(agent.style.add_modifier.contains(Modifier::BOLD));
        assert!(agent.style.add_modifier.contains(Modifier::UNDERLINED));
        assert_eq!(agent.style.fg, Some(Color::Rgb(122, 162, 247)));
    }

    #[test]
    fn inactive_tab_uses_muted_style() {
        let theme = test_theme();
        let line = tab_bar_line(&MAIN_TAB_LABELS, 0, &theme);
        let issues = &line.spans[2];

        assert_eq!(issues.content, "Issues");
        assert!(!issues.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(issues.style.fg, Some(Color::Rgb(86, 95, 137)));
    }
}
