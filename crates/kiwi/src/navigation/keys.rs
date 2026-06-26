//! Default keybindings for tab switching and focus routing.
//!
//! Bindings match `docs/design/keyboard-shortcuts.md` and SPEC-004. Navigation
//! shortcuts are suppressed when the shell PTY has focus so keys can be forwarded.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::layout::FocusTarget;
use crate::navigation::{LeftNavTab, MainTab, NavCommand};

pub fn map_key(event: KeyEvent, focus: FocusTarget) -> Option<NavCommand> {
    if focus == FocusTarget::Shell {
        return None;
    }

    if event.modifiers.contains(KeyModifiers::CONTROL) {
        return None;
    }

    if event.modifiers.contains(KeyModifiers::ALT) {
        return left_tab_from_digit(event.code);
    }

    match event.code {
        KeyCode::Tab if event.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(NavCommand::PreviousFocus)
        }
        KeyCode::Tab => Some(NavCommand::NextFocus),
        KeyCode::Char(c @ '1'..='8') if matches!(focus, FocusTarget::Main | FocusTarget::Left) => {
            main_tab_from_digit(c)
        }
        _ => None,
    }
}

fn left_tab_from_digit(code: KeyCode) -> Option<NavCommand> {
    let index = match code {
        KeyCode::Char('1') => 0,
        KeyCode::Char('2') => 1,
        KeyCode::Char('3') => 2,
        KeyCode::Char('4') => 3,
        _ => return None,
    };
    LeftNavTab::from_index(index).map(NavCommand::SelectLeftTab)
}

fn main_tab_from_digit(digit: char) -> Option<NavCommand> {
    let index = digit.to_digit(10)? as usize;
    MainTab::from_index(index.saturating_sub(1)).map(NavCommand::SelectMainTab)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventKind, KeyEventState};

    use super::*;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn press_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn alt_digit_selects_left_tab() {
        let cmd = map_key(
            press_with_modifiers(KeyCode::Char('3'), KeyModifiers::ALT),
            FocusTarget::Main,
        );
        assert_eq!(cmd, Some(NavCommand::SelectLeftTab(LeftNavTab::Gh)));
    }

    #[test]
    fn left_tab_shortcuts_match_design_doc() {
        let expected = [
            (KeyCode::Char('1'), LeftNavTab::Files),
            (KeyCode::Char('2'), LeftNavTab::Git),
            (KeyCode::Char('3'), LeftNavTab::Gh),
            (KeyCode::Char('4'), LeftNavTab::Search),
        ];

        for (digit, tab) in expected {
            let cmd = map_key(
                press_with_modifiers(digit, KeyModifiers::ALT),
                FocusTarget::Left,
            );
            assert_eq!(cmd, Some(NavCommand::SelectLeftTab(tab)));
        }
    }

    #[test]
    fn digit_selects_main_tab_when_main_focused() {
        let cmd = map_key(press(KeyCode::Char('2')), FocusTarget::Main);
        assert_eq!(cmd, Some(NavCommand::SelectMainTab(MainTab::Issues)));
    }

    #[test]
    fn digit_selects_main_tab_when_left_focused() {
        let cmd = map_key(press(KeyCode::Char('6')), FocusTarget::Left);
        assert_eq!(cmd, Some(NavCommand::SelectMainTab(MainTab::Preview)));
    }

    #[test]
    fn digit_seven_selects_logs_tab() {
        let cmd = map_key(press(KeyCode::Char('7')), FocusTarget::Main);
        assert_eq!(cmd, Some(NavCommand::SelectMainTab(MainTab::Logs)));
    }

    #[test]
    fn digit_eight_selects_settings_tab() {
        let cmd = map_key(press(KeyCode::Char('8')), FocusTarget::Main);
        assert_eq!(cmd, Some(NavCommand::SelectMainTab(MainTab::Settings)));
    }

    #[test]
    fn main_tab_shortcuts_match_design_doc() {
        let expected = [
            (KeyCode::Char('1'), MainTab::Agent),
            (KeyCode::Char('2'), MainTab::Issues),
            (KeyCode::Char('3'), MainTab::Branches),
            (KeyCode::Char('4'), MainTab::Prs),
            (KeyCode::Char('5'), MainTab::Diff),
            (KeyCode::Char('6'), MainTab::Preview),
            (KeyCode::Char('7'), MainTab::Logs),
            (KeyCode::Char('8'), MainTab::Settings),
        ];

        for (digit, tab) in expected {
            let cmd = map_key(press(digit), FocusTarget::Main);
            assert_eq!(cmd, Some(NavCommand::SelectMainTab(tab)));
        }
    }

    #[test]
    fn main_tab_digits_ignored_when_palette_focused() {
        let cmd = map_key(press(KeyCode::Char('2')), FocusTarget::CommandPalette);
        assert_eq!(cmd, None);
    }

    #[test]
    fn tab_cycles_focus() {
        let cmd = map_key(press(KeyCode::Tab), FocusTarget::Main);
        assert_eq!(cmd, Some(NavCommand::NextFocus));

        let cmd = map_key(
            press_with_modifiers(KeyCode::Tab, KeyModifiers::SHIFT),
            FocusTarget::Main,
        );
        assert_eq!(cmd, Some(NavCommand::PreviousFocus));
    }

    #[test]
    fn ctrl_p_is_not_handled_by_navigation_mapper() {
        let cmd = map_key(
            press_with_modifiers(KeyCode::Char('p'), KeyModifiers::CONTROL),
            FocusTarget::Main,
        );
        assert_eq!(cmd, None);
    }

    #[test]
    fn shell_focus_suppresses_navigation_shortcuts() {
        assert_eq!(map_key(press(KeyCode::Tab), FocusTarget::Shell), None);
        assert_eq!(
            map_key(
                press_with_modifiers(KeyCode::Char('1'), KeyModifiers::ALT),
                FocusTarget::Shell,
            ),
            None
        );
        assert_eq!(map_key(press(KeyCode::Char('2')), FocusTarget::Shell), None);
        assert_eq!(
            map_key(
                press_with_modifiers(KeyCode::Char('p'), KeyModifiers::CONTROL),
                FocusTarget::Shell,
            ),
            None
        );
    }
}
