//! Keyboard input mapping for the integrated terminal.

use egui::{Key, Modifiers};

/// Maps a focused egui key press to bytes for the PTY.
///
/// Printable characters arrive via [`egui::Event::Text`]; this handles control keys
/// and modifier combinations.
pub fn key_to_bytes(key: Key, modifiers: Modifiers) -> Option<Vec<u8>> {
    if modifiers.ctrl {
        return ctrl_key(key);
    }

    match key {
        Key::Enter => Some(vec![b'\r']),
        Key::Backspace => Some(vec![0x7f]),
        Key::Tab => Some(vec![b'\t']),
        Key::ArrowUp => Some(b"\x1b[A".to_vec()),
        Key::ArrowDown => Some(b"\x1b[B".to_vec()),
        Key::ArrowRight => Some(b"\x1b[C".to_vec()),
        Key::ArrowLeft => Some(b"\x1b[D".to_vec()),
        Key::Home => Some(b"\x1b[H".to_vec()),
        Key::End => Some(b"\x1b[F".to_vec()),
        Key::PageUp => Some(b"\x1b[5~".to_vec()),
        Key::PageDown => Some(b"\x1b[6~".to_vec()),
        Key::Delete => Some(b"\x1b[3~".to_vec()),
        Key::Escape => Some(vec![0x1b]),
        _ => None,
    }
}

fn ctrl_key(key: Key) -> Option<Vec<u8>> {
    let code = match key {
        Key::A => Some(b'a'),
        Key::B => Some(b'b'),
        Key::C => Some(b'c'),
        Key::D => Some(b'd'),
        Key::E => Some(b'e'),
        Key::F => Some(b'f'),
        Key::G => Some(b'g'),
        Key::H => Some(b'h'),
        Key::I => Some(b'i'),
        Key::J => Some(b'j'),
        Key::K => Some(b'k'),
        Key::L => Some(b'l'),
        Key::M => Some(b'm'),
        Key::N => Some(b'n'),
        Key::O => Some(b'o'),
        Key::P => Some(b'p'),
        Key::Q => Some(b'q'),
        Key::R => Some(b'r'),
        Key::S => Some(b's'),
        Key::T => Some(b't'),
        Key::U => Some(b'u'),
        Key::V => Some(b'v'),
        Key::W => Some(b'w'),
        Key::X => Some(b'x'),
        Key::Y => Some(b'y'),
        Key::Z => Some(b'z'),
        Key::Enter => return Some(vec![b'\n']),
        _ => None,
    }?;
    Some(vec![code & 0x1f])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_enter_to_carriage_return() {
        assert_eq!(key_to_bytes(Key::Enter, Modifiers::NONE), Some(vec![b'\r']));
    }

    #[test]
    fn maps_ctrl_c_to_etx() {
        assert_eq!(
            key_to_bytes(Key::C, Modifiers::CTRL),
            Some(vec![0x03])
        );
    }

    #[test]
    fn maps_arrow_up_to_ansi_sequence() {
        assert_eq!(
            key_to_bytes(Key::ArrowUp, Modifiers::NONE),
            Some(b"\x1b[A".to_vec())
        );
    }
}
