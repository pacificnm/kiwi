use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOp {
    Copy,
    Cut,
    Paste,
}

pub fn clipboard_op_from_key(key: KeyEvent) -> Option<ClipboardOp> {
    if key.kind != crossterm::event::KeyEventKind::Press {
        return None;
    }

    if key.modifiers != KeyModifiers::CONTROL {
        return None;
    }

    match key.code {
        KeyCode::Char('c' | 'C') => Some(ClipboardOp::Copy),
        KeyCode::Char('x' | 'X') => Some(ClipboardOp::Cut),
        KeyCode::Char('v' | 'V') => Some(ClipboardOp::Paste),
        _ => None,
    }
}

/// Shell keeps `Ctrl+C` / `Ctrl+X` as PTY signals unless text is highlighted.
pub fn clipboard_shortcut_allowed(
    op: ClipboardOp,
    shell_focused: bool,
    shell_has_selection: bool,
) -> bool {
    if !shell_focused {
        return true;
    }

    match op {
        ClipboardOp::Copy | ClipboardOp::Cut => shell_has_selection,
        ClipboardOp::Paste => true,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventKind, KeyEventState};

    use super::*;

    fn press_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn detects_standard_clipboard_shortcuts() {
        assert_eq!(
            clipboard_op_from_key(press_ctrl(KeyCode::Char('c'))),
            Some(ClipboardOp::Copy)
        );
        assert_eq!(
            clipboard_op_from_key(press_ctrl(KeyCode::Char('x'))),
            Some(ClipboardOp::Cut)
        );
        assert_eq!(
            clipboard_op_from_key(press_ctrl(KeyCode::Char('v'))),
            Some(ClipboardOp::Paste)
        );
    }

    #[test]
    fn ignores_shift_modified_shortcuts() {
        let key = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(clipboard_op_from_key(key), None);
    }

    #[test]
    fn shell_preserves_ctrl_c_interrupt_without_selection() {
        assert!(!clipboard_shortcut_allowed(ClipboardOp::Copy, true, false));
        assert!(!clipboard_shortcut_allowed(ClipboardOp::Cut, true, false));
        assert!(clipboard_shortcut_allowed(ClipboardOp::Paste, true, false));
    }

    #[test]
    fn shell_uses_clipboard_shortcuts_when_text_is_selected() {
        assert!(clipboard_shortcut_allowed(ClipboardOp::Copy, true, true));
        assert!(clipboard_shortcut_allowed(ClipboardOp::Cut, true, true));
    }
}
