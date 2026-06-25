const DEFAULT_CAPACITY: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollbackBuffer {
    lines: Vec<String>,
    partial: String,
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
            lines: Vec::new(),
            partial: String::new(),
            capacity: DEFAULT_CAPACITY,
        }
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    #[must_use]
    pub fn has_pending_line(&self) -> bool {
        !self.partial.is_empty()
    }

    #[must_use]
    pub fn pending_display(&self) -> String {
        normalize_for_display(self.partial.trim_end_matches('\r'))
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.partial.clear();
    }

    pub fn append_bytes(&mut self, data: &[u8]) {
        let chunk = String::from_utf8_lossy(data);
        self.partial.push_str(&chunk);

        while let Some(newline) = self.partial.find('\n') {
            let mut line = self.partial.drain(..=newline).collect::<String>();
            line.pop();
            line = line.trim_end_matches('\r').to_string();
            self.push_line(normalize_for_display(&line));
        }
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

        let mut lines: Vec<String> = self
            .lines
            .iter()
            .skip(start)
            .take(visible_height)
            .map(|line| truncate_line(&normalize_for_display(line), max_width))
            .collect();

        if !include_pending {
            return lines;
        }

        let pending = truncate_line(&self.pending_display(), max_width);
        if pending.is_empty() {
            return lines;
        }

        if lines.len() >= visible_height {
            lines.pop();
        }
        lines.push(pending);
        lines
    }

    fn push_line(&mut self, line: String) {
        self.lines.push(line);
        if self.lines.len() > self.capacity {
            let overflow = self.lines.len() - self.capacity;
            self.lines.drain(0..overflow);
        }
    }
}

fn normalize_for_display(input: &str) -> String {
    let mut line = strip_ansi(input);
    if let Some(index) = line.rfind('\r') {
        line = line[index + 1..].to_string();
    }
    line.replace('\t', "    ")
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.next_if_eq(&'[').is_some() {
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }

    out
}

fn truncate_line(line: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let char_count = line.chars().count();
    if char_count <= width {
        return line.to_string();
    }

    if width == 1 {
        return "…".to_string();
    }

    let prefix: String = line.chars().take(width - 1).collect();
    format!("{prefix}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_bytes_splits_on_newlines() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"hello\nworld");
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.lines[0], "hello");

        buffer.append_bytes(b"\n!");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.lines[1], "world");
    }

    #[test]
    fn scrollback_caps_at_ten_thousand_lines() {
        let mut buffer = ScrollbackBuffer::new();
        for index in 0..10_001 {
            buffer.append_bytes(format!("line {index}\n").as_bytes());
        }

        assert_eq!(buffer.line_count(), 10_000);
        assert_eq!(buffer.lines.first(), Some(&"line 1".to_string()));
        assert_eq!(buffer.lines.last(), Some(&"line 10000".to_string()));
    }

    #[test]
    fn strip_ansi_removes_color_codes() {
        assert_eq!(strip_ansi("\x1b[1;32mok\x1b[0m"), "ok");
    }

    #[test]
    fn visible_lines_respects_viewport_and_width() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"one\n two\nthree\n");
        let lines = buffer.viewport_lines(1, 2, 10, false);
        assert_eq!(lines, vec![" two".to_string(), "three".to_string()]);
    }

    #[test]
    fn viewport_lines_includes_pending_prompt_without_newline() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"user@host:~/kiwi$ ");
        let lines = buffer.viewport_lines(0, 3, 40, true);
        assert_eq!(lines, vec!["user@host:~/kiwi$ ".to_string()]);
    }

    #[test]
    fn viewport_lines_replaces_last_row_with_pending_at_tail() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"line1\nline2\n");
        buffer.append_bytes(b"partial");
        let lines = buffer.viewport_lines(0, 2, 20, true);
        assert_eq!(lines, vec!["line1".to_string(), "partial".to_string()]);
    }

    #[test]
    fn normalize_for_display_keeps_text_after_carriage_return() {
        assert_eq!(normalize_for_display("prompt\rtyped"), "typed");
    }

    #[test]
    fn normalize_for_display_expands_tabs() {
        assert_eq!(normalize_for_display("a\tb"), "a    b");
    }

    #[test]
    fn truncate_line_clips_to_width() {
        assert_eq!(truncate_line("hello world", 5), "hell…");
    }
}
