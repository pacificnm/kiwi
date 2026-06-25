use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn encode_key(event: KeyEvent) -> Option<Vec<u8>> {
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        return encode_control_key(event.code);
    }

    match event.code {
        KeyCode::Char(ch) => Some(ch.to_string().into_bytes()),
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
        KeyCode::Insert => Some(b"\x1b[2~".to_vec()),
        KeyCode::F(n) => encode_function_key(n),
        _ => None,
    }
}

fn encode_control_key(code: KeyCode) -> Option<Vec<u8>> {
    match code {
        KeyCode::Char('c' | 'C') => Some(vec![0x03]),
        KeyCode::Char('d' | 'D') => Some(vec![0x04]),
        KeyCode::Char('z' | 'Z') => Some(vec![0x1a]),
        KeyCode::Char(ch) if ch.is_ascii_alphabetic() => {
            Some(vec![(ch.to_ascii_lowercase() as u8) & 0x1f])
        }
        KeyCode::Char(ch) if ch.is_ascii_digit() => Some(vec![(ch as u8) & 0x1f]),
        _ => None,
    }
}

fn encode_function_key(number: u8) -> Option<Vec<u8>> {
    match number {
        1 => Some(b"\x1bOP".to_vec()),
        2 => Some(b"\x1bOQ".to_vec()),
        3 => Some(b"\x1bOR".to_vec()),
        4 => Some(b"\x1bOS".to_vec()),
        5..=12 => Some(format!("\x1b[{}~", number + 10).into_bytes()),
        _ => None,
    }
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

    fn press_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn encodes_printable_characters() {
        assert_eq!(encode_key(press(KeyCode::Char('a'))), Some(b"a".to_vec()));
    }

    #[test]
    fn encodes_enter_and_backspace() {
        assert_eq!(encode_key(press(KeyCode::Enter)), Some(b"\r".to_vec()));
        assert_eq!(encode_key(press(KeyCode::Backspace)), Some(vec![0x7f]));
    }

    #[test]
    fn encodes_control_c_as_interrupt() {
        assert_eq!(encode_key(press_ctrl(KeyCode::Char('c'))), Some(vec![0x03]));
    }

    #[test]
    fn encodes_arrow_keys() {
        assert_eq!(encode_key(press(KeyCode::Up)), Some(b"\x1b[A".to_vec()));
    }
}
