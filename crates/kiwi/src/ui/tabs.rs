use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

pub const LEFT_TAB_LABELS: &[&str] = &["Files", "Git", "Diff", "GH", "Search"];
pub const MAIN_TAB_LABELS: &[&str] = &["Agent", "Issues", "PRs", "Diff", "Preview", "Logs"];

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
        assert_eq!(LEFT_TAB_LABELS, &["Files", "Git", "Diff", "GH", "Search"]);
    }

    #[test]
    fn main_tab_labels_match_design() {
        assert_eq!(
            MAIN_TAB_LABELS,
            &["Agent", "Issues", "PRs", "Diff", "Preview", "Logs"]
        );
    }

    #[test]
    fn active_tab_uses_accent_bold_underline() {
        let theme = test_theme();
        let line = tab_bar_line(MAIN_TAB_LABELS, 0, &theme);
        let agent = &line.spans[0];

        assert_eq!(agent.content, "Agent");
        assert!(agent.style.add_modifier.contains(Modifier::BOLD));
        assert!(agent.style.add_modifier.contains(Modifier::UNDERLINED));
        assert_eq!(agent.style.fg, Some(Color::Rgb(122, 162, 247)));
    }

    #[test]
    fn inactive_tab_uses_muted_style() {
        let theme = test_theme();
        let line = tab_bar_line(MAIN_TAB_LABELS, 0, &theme);
        let issues = &line.spans[2];

        assert_eq!(issues.content, "Issues");
        assert!(!issues.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(issues.style.fg, Some(Color::Rgb(86, 95, 137)));
    }
}
