const DEFAULT_CAPACITY: usize = 10_000;
const DEFAULT_COLS: usize = 80;
const DEFAULT_SCREEN_ROWS: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollbackBuffer {
    cols: usize,
    screen: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    cursor_visible: bool,
    overwrite_line: bool,
    history: Vec<String>,
    pending: Vec<u8>,
    text_pending: Vec<u8>,
    capacity: usize,
}

impl Default for ScrollbackBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollbackBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cols: DEFAULT_COLS,
            screen: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: true,
            overwrite_line: false,
            history: Vec::new(),
            pending: Vec::new(),
            text_pending: Vec::new(),
            capacity: DEFAULT_CAPACITY,
        }
    }

    pub fn set_cols(&mut self, cols: u16) {
        self.cols = usize::from(cols.max(1));
    }

    pub fn line_count(&self) -> usize {
        let total = self.history.len() + self.screen.len();
        total.saturating_sub(self.trailing_empty_count(true))
    }

    #[must_use]
    pub fn cursor_display_position(&self, include_pending: bool) -> Option<(usize, usize)> {
        // DECSCUSR (?25) toggles the host hardware cursor. Child TUIs (including
        // the agent CLI) usually send ?25l; Kiwi still draws a focused-pane overlay
        // at the emulated cursor position tracked from PTY output and typed input.
        let line_index = self.history.len() + self.cursor_row;
        let count = (self.history.len() + self.screen.len())
            .saturating_sub(self.trailing_empty_count(include_pending));
        if line_index >= count {
            return None;
        }
        Some((line_index, self.cursor_col))
    }

    #[must_use]
    pub fn has_pending_line(&self) -> bool {
        self.current_line()
            .map(|line| !line.is_empty())
            .unwrap_or(false)
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.screen = vec![String::new()];
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.cursor_visible = true;
        self.overwrite_line = false;
        self.pending.clear();
        self.text_pending.clear();
    }

    pub fn append_bytes(&mut self, data: &[u8]) {
        if self.pending.is_empty() {
            self.process_bytes(data);
            return;
        }

        let mut combined = std::mem::take(&mut self.pending);
        combined.extend_from_slice(data);
        self.process_bytes(&combined);
    }

    #[must_use]
    pub fn viewport_start(
        &self,
        visible_height: usize,
        follow_tail: bool,
        viewport_offset: usize,
    ) -> usize {
        let max_start = self.line_count().saturating_sub(visible_height);
        if follow_tail {
            max_start
        } else {
            viewport_offset.min(max_start)
        }
    }

    #[must_use]
    pub fn recent_text(&self, max_lines: usize) -> String {
        let lines = self.lines_for_display(true);
        let start = lines.len().saturating_sub(max_lines);
        lines[start..]
            .iter()
            .map(|line| crate::ansi::strip_ansi(line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[must_use]
    pub fn viewport_lines(
        &self,
        start: usize,
        visible_height: usize,
        max_width: usize,
        include_pending: bool,
    ) -> Vec<String> {
        if visible_height == 0 || max_width == 0 {
            return Vec::new();
        }

        let count = (self.history.len() + self.screen.len())
            .saturating_sub(self.trailing_empty_count(include_pending));

        self.history
            .iter()
            .chain(self.screen.iter())
            .take(count)
            .skip(start)
            .take(visible_height)
            .map(|line| truncate_ansi_line(line, max_width))
            .collect()
    }

    fn trailing_empty_count(&self, include_pending: bool) -> usize {
        let h_len = self.history.len();
        let total = h_len + self.screen.len();
        let mut count = 0;
        for combined_idx in (0..total).rev() {
            let line = if combined_idx < h_len {
                &self.history[combined_idx]
            } else {
                &self.screen[combined_idx - h_len]
            };
            if !line.is_empty() {
                break;
            }
            if include_pending && combined_idx == self.cursor_row && self.cursor_col > 0 {
                break;
            }
            count += 1;
        }
        count
    }

    fn lines_for_display(&self, include_pending: bool) -> Vec<String> {
        let total = self.history.len() + self.screen.len();
        let count = total.saturating_sub(self.trailing_empty_count(include_pending));
        self.history
            .iter()
            .chain(self.screen.iter())
            .take(count)
            .cloned()
            .collect()
    }

    fn current_line(&self) -> Option<String> {
        self.screen.get(self.cursor_row).cloned()
    }

    fn process_bytes(&mut self, data: &[u8]) {
        let mut idx = 0;
        while idx < data.len() {
            if data[idx] == 0x1b {
                self.flush_text_pending();
                if let Some((consumed, action)) = parse_escape(&data[idx..]) {
                    self.apply_action(action);
                    idx += consumed;
                    continue;
                }
                if escape_needs_more(&data[idx..]) {
                    self.pending.extend_from_slice(&data[idx..]);
                    return;
                }
                if let Some(consumed) = consume_non_csi_escape(&data[idx..]) {
                    idx += consumed;
                    continue;
                }
            }

            if is_text_byte(data[idx]) {
                self.text_pending.push(data[idx]);
                idx += 1;
                self.flush_text_pending();
                continue;
            }

            self.flush_text_pending();

            match data[idx] {
                b'\n' | b'\x0c' => {
                    self.commit_line();
                    self.cursor_row += 1;
                    self.cursor_col = 0;
                    self.overwrite_line = true;
                    self.ensure_screen_row(self.cursor_row);
                    self.scroll_if_needed();
                }
                b'\r' => {
                    self.cursor_col = 0;
                    self.overwrite_line = true;
                }
                b'\t' => {
                    let next = (self.cursor_col / 8 + 1) * 8;
                    self.write_str(&" ".repeat(next - self.cursor_col));
                }
                b'\x08' => {
                    self.cursor_col = self.cursor_col.saturating_sub(1);
                }
                _ => {}
            }
            idx += 1;
        }
    }

    fn flush_text_pending(&mut self) {
        loop {
            if self.text_pending.is_empty() {
                return;
            }

            match std::str::from_utf8(&self.text_pending) {
                Ok(text) => {
                    let text = text.to_string();
                    self.text_pending.clear();
                    self.write_str(&text);
                }
                Err(error) => {
                    let valid = error.valid_up_to();
                    if valid > 0 {
                        let text = std::str::from_utf8(&self.text_pending[..valid])
                            .expect("valid utf-8 prefix")
                            .to_string();
                        self.text_pending.drain(0..valid);
                        self.write_str(&text);
                        continue;
                    }

                    match error.error_len() {
                        Some(invalid_len) => {
                            self.write_char('\u{FFFD}');
                            self.text_pending.drain(0..invalid_len);
                        }
                        None => return,
                    }
                }
            }
        }
    }

    fn apply_action(&mut self, action: EscapeAction) {
        match action {
            EscapeAction::Ignore => {}
            EscapeAction::ClearScreen => self.clear_screen(),
            EscapeAction::ClearBelow => self.clear_below(),
            EscapeAction::ClearLine => self.clear_line(),
            EscapeAction::CursorPosition { row, col } => {
                self.cursor_row = row.saturating_sub(1);
                self.cursor_col = col.saturating_sub(1);
                self.ensure_screen_row(self.cursor_row);
            }
            EscapeAction::CursorUp(n) => {
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            EscapeAction::CursorDown(n) => {
                self.cursor_row += n;
                self.ensure_screen_row(self.cursor_row);
            }
            EscapeAction::CursorForward(n) => {
                self.cursor_col += n;
            }
            EscapeAction::CursorBack(n) => {
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            EscapeAction::ShowCursor => self.cursor_visible = true,
            EscapeAction::HideCursor => self.cursor_visible = false,
            EscapeAction::Raw(sequence) => self.write_str(&sequence),
        }
    }

    fn clear_screen(&mut self) {
        self.screen = vec![String::new()];
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.overwrite_line = true;
    }

    fn clear_below(&mut self) {
        self.clear_line();
        if self.cursor_row + 1 < self.screen.len() {
            self.screen.truncate(self.cursor_row + 1);
        }
        self.ensure_screen_row(self.cursor_row);
    }

    fn clear_line(&mut self) {
        self.ensure_screen_row(self.cursor_row);
        self.screen[self.cursor_row].clear();
        self.cursor_col = 0;
        self.overwrite_line = true;
    }

    fn commit_line(&mut self) {
        self.ensure_screen_row(self.cursor_row);
    }

    fn write_char(&mut self, ch: char) {
        self.write_str(&ch.to_string());
    }

    fn write_str(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        self.ensure_screen_row(self.cursor_row);
        let line = &mut self.screen[self.cursor_row];

        if self.overwrite_line && self.cursor_col == 0 {
            line.clear();
            self.overwrite_line = false;
        }

        let visible_len = crate::ansi::visible_width(line);
        if self.cursor_col > visible_len {
            line.push_str(&" ".repeat(self.cursor_col - visible_len));
        }

        if self.cursor_col == visible_len {
            line.push_str(text);
        } else {
            let (prefix, suffix) = split_at_visible(line, self.cursor_col);
            *line = format!("{prefix}{text}{suffix}");
        }

        self.cursor_col += crate::ansi::visible_width(text);
    }

    fn ensure_screen_row(&mut self, row: usize) {
        while self.screen.len() <= row {
            self.screen.push(String::new());
        }
        if self.screen.len() > DEFAULT_SCREEN_ROWS {
            let overflow = self.screen.len() - DEFAULT_SCREEN_ROWS;
            self.history.extend(self.screen.drain(0..overflow));
            self.cursor_row = self.cursor_row.saturating_sub(overflow);
            self.trim_history();
        }
    }

    fn scroll_if_needed(&mut self) {
        if self.screen.len() > DEFAULT_SCREEN_ROWS {
            let overflow = self.screen.len() - DEFAULT_SCREEN_ROWS;
            self.history.extend(self.screen.drain(0..overflow));
            self.cursor_row = self.cursor_row.saturating_sub(overflow);
            self.trim_history();
        }
    }

    fn trim_history(&mut self) {
        let total = self.history.len().saturating_add(self.screen.len());
        if total <= self.capacity {
            return;
        }
        let overflow = total - self.capacity;
        if overflow >= self.history.len() {
            self.history.clear();
            return;
        }
        self.history.drain(0..overflow);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EscapeAction {
    Ignore,
    ClearScreen,
    ClearBelow,
    ClearLine,
    CursorPosition { row: usize, col: usize },
    CursorUp(usize),
    CursorDown(usize),
    CursorForward(usize),
    CursorBack(usize),
    ShowCursor,
    HideCursor,
    Raw(String),
}

fn escape_needs_more(data: &[u8]) -> bool {
    if data.first() != Some(&0x1b) {
        return false;
    }

    match data.get(1) {
        None => true,
        Some(b'[') => !data[2..].iter().any(|byte| (0x40..=0x7E).contains(byte)),
        Some(b']') => !data[2..].iter().any(|&byte| byte == 0x07 || byte == b'\\'),
        Some(_) => data.len() < 2,
    }
}

fn consume_non_csi_escape(data: &[u8]) -> Option<usize> {
    if data.first() != Some(&0x1b) {
        return None;
    }

    let next = *data.get(1)?;
    if next == b'[' || next == b']' {
        return None;
    }

    Some(2)
}

fn parse_escape(data: &[u8]) -> Option<(usize, EscapeAction)> {
    if data.first() != Some(&0x1b) {
        return None;
    }

    if data.get(1) == Some(&b']') {
        return parse_osc(data);
    }

    if data.get(1) == Some(&b'[') {
        return parse_csi(data);
    }

    None
}

fn parse_csi(data: &[u8]) -> Option<(usize, EscapeAction)> {
    let mut idx = 2;
    while idx < data.len() {
        let byte = data[idx];
        if (0x30..=0x3F).contains(&byte) || (0x20..=0x2F).contains(&byte) {
            idx += 1;
            continue;
        }
        if (0x40..=0x7E).contains(&byte) {
            let params = String::from_utf8_lossy(&data[2..idx]).into_owned();
            let action = if byte == b'm' {
                EscapeAction::Raw(String::from_utf8_lossy(&data[..idx + 1]).into_owned())
            } else {
                decode_csi(byte, &params)
            };
            return Some((idx + 1, action));
        }
        return None;
    }

    None
}

fn parse_osc(data: &[u8]) -> Option<(usize, EscapeAction)> {
    let mut idx = 2;
    while idx < data.len() {
        if data[idx] == 0x07
            || (data[idx] == b'\\' && idx + 1 < data.len() && data[idx + 1] == b'\\')
        {
            return Some((idx + 1, EscapeAction::Ignore));
        }
        if data[idx] == b'\\' {
            return Some((idx + 1, EscapeAction::Ignore));
        }
        idx += 1;
    }
    None
}

fn decode_csi(final_byte: u8, params: &str) -> EscapeAction {
    let parts = params.split(';').collect::<Vec<_>>();
    let n = |index: usize| -> usize {
        parts
            .get(index)
            .and_then(|value| value.parse().ok())
            .unwrap_or(0)
    };
    let first = parts.first().copied().unwrap_or("");

    match final_byte {
        b'H' | b'f' => EscapeAction::CursorPosition {
            row: n(0).max(1),
            col: n(1).max(1),
        },
        b'A' => EscapeAction::CursorUp(n(0).max(1)),
        b'B' => EscapeAction::CursorDown(n(0).max(1)),
        b'C' => EscapeAction::CursorForward(n(0).max(1)),
        b'D' => EscapeAction::CursorBack(n(0).max(1)),
        b'J' => match n(0) {
            0 => EscapeAction::ClearBelow,
            1 => EscapeAction::Ignore,
            _ => EscapeAction::ClearScreen,
        },
        b'K' => EscapeAction::ClearLine,
        b'h' | b'l' if first == "?1049" => {
            if final_byte == b'h' {
                EscapeAction::ClearScreen
            } else {
                EscapeAction::Ignore
            }
        }
        b'h' if first == "?25" => EscapeAction::ShowCursor,
        b'l' if first == "?25" => EscapeAction::HideCursor,
        b'h' | b'l' => EscapeAction::Ignore,
        _ => EscapeAction::Ignore,
    }
}

fn is_text_byte(byte: u8) -> bool {
    byte >= 0x20 && byte != 0x7f || (0x80..=0xFF).contains(&byte)
}

fn split_at_visible(line: &str, col: usize) -> (String, String) {
    if col == 0 {
        return (String::new(), line.to_string());
    }

    let mut visible = 0usize;
    let mut split_idx = 0usize;
    let mut byte_pos = 0usize;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            byte_pos += ch.len_utf8();
            if chars.next_if_eq(&'[').is_some() {
                byte_pos += '['.len_utf8();
                for c in chars.by_ref() {
                    byte_pos += c.len_utf8();
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            split_idx = byte_pos;
            continue;
        }
        if visible == col {
            return (line[..split_idx].to_string(), line[split_idx..].to_string());
        }
        visible += 1;
        byte_pos += ch.len_utf8();
        split_idx = byte_pos;
    }

    (line.to_string(), String::new())
}

fn truncate_ansi_line(line: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let visible = crate::ansi::visible_width(line);
    if visible <= width {
        return line.to_string();
    }

    if width == 1 {
        return "…".to_string();
    }

    let mut out = String::new();
    let mut seen = 0usize;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            out.push(ch);
            if chars.next_if_eq(&'[').is_some() {
                out.push('[');
                for c in chars.by_ref() {
                    out.push(c);
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            continue;
        }

        if seen >= width - 1 {
            out.push('…');
            break;
        }

        out.push(ch);
        seen += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_bytes_splits_on_newlines() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"hello\nworld");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.lines_for_display(false)[0], "hello");
        assert_eq!(buffer.lines_for_display(false)[1], "world");
    }

    #[test]
    fn line_count_excludes_trailing_blank_screen_rows() {
        // A trailing newline leaves a blank screen row at the cursor position.
        // line_count() and viewport_lines() must not include it.
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"alpha\nbeta\n");
        assert_eq!(buffer.line_count(), 2);
        let lines = buffer.viewport_lines(0, 10, 80, false);
        assert_eq!(lines, vec!["alpha".to_string(), "beta".to_string()]);
    }

    #[test]
    fn carriage_return_overwrites_current_line() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"prompt\rtyped");
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.lines_for_display(true)[0], "typed");
    }

    #[test]
    fn clear_screen_drops_duplicate_prompts() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"> prompt 1\n");
        buffer.append_bytes(b"\x1b[2J\x1b[H> prompt 2");
        let lines = buffer.lines_for_display(true);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("prompt 2"));
        assert!(!lines[0].contains("prompt 1"));
    }

    #[test]
    fn cursor_up_allows_redrawing_previous_line() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"line one\n");
        buffer.append_bytes(b"line two\n");
        buffer.append_bytes(b"\x1b[1A\rline two updated");
        let lines = buffer.lines_for_display(true);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1], "line two updated");
    }

    #[test]
    fn viewport_lines_includes_current_screen_row() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"user@host:~/kiwi$ ");
        let lines = buffer.viewport_lines(0, 3, 40, true);
        assert_eq!(lines, vec!["user@host:~/kiwi$ ".to_string()]);
    }

    #[test]
    fn viewport_lines_preserves_ansi_color_codes() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"\x1b[32mgreen\x1b[0m\n");
        let lines = buffer.viewport_lines(0, 1, 20, false);
        assert_eq!(lines, vec!["\x1b[32mgreen\x1b[0m".to_string()]);
    }

    #[test]
    fn recent_text_strips_ansi_for_heuristics() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"\x1b[32mRunning tool\x1b[0m\n");
        assert!(buffer.recent_text(4).contains("Running tool"));
    }

    #[test]
    fn truncate_ansi_line_clips_visible_width() {
        assert_eq!(truncate_ansi_line("hello world", 5), "hell…");
    }

    #[test]
    fn split_escape_sequence_across_reads_is_reassembled() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"> prompt 1\n");
        buffer.append_bytes(b"\x1b[2");
        buffer.append_bytes(b"J\x1b[H> prompt 2");
        let lines = buffer.lines_for_display(true);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("prompt 2"));
        assert!(!lines[0].contains("prompt 1"));
    }

    #[test]
    fn utf8_multibyte_characters_decode_correctly() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes("arrow → ok\n".as_bytes());
        let lines = buffer.lines_for_display(false);
        assert_eq!(lines, vec!["arrow → ok".to_string()]);
    }

    #[test]
    fn utf8_split_across_reads_is_reassembled() {
        let mut buffer = ScrollbackBuffer::new();
        let text = "→";
        let bytes = text.as_bytes();
        buffer.append_bytes(&bytes[..2]);
        buffer.append_bytes(&bytes[2..]);
        buffer.append_bytes(b"\n");
        assert_eq!(buffer.lines_for_display(false), vec!["→".to_string()]);
    }

    #[test]
    fn private_mode_sequences_are_not_printed() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"\x1b[?25l\x1b[?2004hhello\n");
        let lines = buffer.lines_for_display(false);
        assert_eq!(lines, vec!["hello".to_string()]);
    }

    #[test]
    fn cursor_display_position_tracks_active_line() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"prompt$ ");
        assert_eq!(buffer.cursor_display_position(true), Some((0, 8)));
    }

    #[test]
    fn hide_cursor_mode_still_reports_overlay_position() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"\x1b[?25lprompt$ ");
        assert_eq!(buffer.cursor_display_position(true), Some((0, 8)));
        buffer.append_bytes(b"\x1b[?25h");
        assert_eq!(buffer.cursor_display_position(true), Some((0, 8)));
    }

    #[test]
    fn split_at_visible_plain_text() {
        assert_eq!(
            split_at_visible("hello world", 5),
            ("hello".to_string(), " world".to_string())
        );
    }

    #[test]
    fn split_at_visible_ansi_prefix_not_counted_as_columns() {
        // "\x1b[32m" is 5 bytes but 0 visible columns — split at col 3 must
        // not treat the escape characters as visible width.
        let colored = "\x1b[32mabc\x1b[0m";
        let (left, right) = split_at_visible(colored, 2);
        assert_eq!(left, "\x1b[32mab");
        assert_eq!(right, "c\x1b[0m");
    }

    #[test]
    fn split_at_visible_ansi_between_chars_goes_to_left() {
        // Escape between "b" and "c": split at col 2 should include the escape
        // in the left (prefix) portion since it precedes the split point.
        let line = "ab\x1b[32mc";
        let (left, right) = split_at_visible(line, 2);
        assert_eq!(left, "ab\x1b[32m");
        assert_eq!(right, "c");
    }

    #[test]
    fn split_at_visible_col_zero_returns_empty_prefix() {
        assert_eq!(
            split_at_visible("abc", 0),
            (String::new(), "abc".to_string())
        );
    }

    #[test]
    fn split_at_visible_col_beyond_length_returns_full_line() {
        assert_eq!(
            split_at_visible("abc", 10),
            ("abc".to_string(), String::new())
        );
    }

    #[test]
    fn ansi_overwrite_via_carriage_return_preserves_color() {
        // Simulates a colored prompt being overwritten at col 0:
        // "\r\x1b[31mERR" overwrites from column 0 on a line that already
        // has "\x1b[32mOK\x1b[0m".
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"\x1b[32mOK\x1b[0m");
        buffer.append_bytes(b"\r\x1b[31mERR\x1b[0m");
        let lines = buffer.lines_for_display(true);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("ERR"));
        assert!(!lines[0].contains("OK"));
    }
}
