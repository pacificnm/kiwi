use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::layout::FocusTarget;

use super::state::NavCommand;
use super::tabs::{LeftNavTab, MainTab};

pub fn map_key(event: KeyEvent) -> Option<NavCommand> {
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        return match event.code {
            KeyCode::Char('p') | KeyCode::Char('P') => {
                Some(NavCommand::SetFocus(FocusTarget::CommandPalette))
            }
            _ => None,
        };
    }

    if event.modifiers.contains(KeyModifiers::ALT) {
        return left_tab_from_digit(event.code);
    }

    match event.code {
        KeyCode::Tab if event.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(NavCommand::PreviousFocus)
        }
        KeyCode::Tab => Some(NavCommand::NextFocus),
        KeyCode::Char(c @ '1'..='6') => main_tab_from_digit(c),
        _ => None,
    }
}

fn left_tab_from_digit(code: KeyCode) -> Option<NavCommand> {
    let index = match code {
        KeyCode::Char('1') => 0,
        KeyCode::Char('2') => 1,
        KeyCode::Char('3') => 2,
        KeyCode::Char('4') => 3,
        KeyCode::Char('5') => 4,
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
        let cmd = map_key(press_with_modifiers(KeyCode::Char('3'), KeyModifiers::ALT));
        assert_eq!(cmd, Some(NavCommand::SelectLeftTab(LeftNavTab::Diff)));
    }

    #[test]
    fn digit_selects_main_tab() {
        let cmd = map_key(press(KeyCode::Char('2')));
        assert_eq!(cmd, Some(NavCommand::SelectMainTab(MainTab::Issues)));
    }

    #[test]
    fn tab_cycles_focus() {
        let cmd = map_key(press(KeyCode::Tab));
        assert_eq!(cmd, Some(NavCommand::NextFocus));

        let cmd = map_key(press_with_modifiers(KeyCode::Tab, KeyModifiers::SHIFT));
        assert_eq!(cmd, Some(NavCommand::PreviousFocus));
    }

    #[test]
    fn ctrl_p_focuses_palette() {
        let cmd = map_key(press_with_modifiers(
            KeyCode::Char('p'),
            KeyModifiers::CONTROL,
        ));
        assert_eq!(cmd, Some(NavCommand::SetFocus(FocusTarget::CommandPalette)));
    }
}
