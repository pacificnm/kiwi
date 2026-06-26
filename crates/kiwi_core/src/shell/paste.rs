pub fn bracketed_paste_bytes(text: &str) -> Vec<u8> {
    let mut bytes = Vec::from(b"\x1b[200~");
    bytes.extend_from_slice(text.as_bytes());
    bytes.extend_from_slice(b"\x1b[201~");
    bytes
}

/// Paste into a PTY: single-line text is sent raw; multi-line uses bracketed paste.
pub fn pty_paste_bytes(text: &str) -> Vec<u8> {
    if text.contains(['\n', '\r']) {
        bracketed_paste_bytes(text)
    } else {
        text.as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pty_paste_uses_bracketed_mode_only_for_multiline_text() {
        assert_eq!(pty_paste_bytes("hello"), b"hello".to_vec());
        assert_eq!(
            pty_paste_bytes("line\n"),
            b"\x1b[200~line\n\x1b[201~".to_vec()
        );
    }
}
