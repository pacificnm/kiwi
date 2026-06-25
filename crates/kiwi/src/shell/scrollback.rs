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
            self.push_line(strip_ansi(&line));
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
    pub fn visible_lines(
        &self,
        start: usize,
        visible_height: usize,
        max_width: usize,
    ) -> Vec<String> {
        self.lines
            .iter()
            .skip(start)
            .take(visible_height)
            .map(|line| truncate_line(line, max_width))
            .collect()
    }

    fn push_line(&mut self, line: String) {
        self.lines.push(line);
        if self.lines.len() > self.capacity {
            let overflow = self.lines.len() - self.capacity;
            self.lines.drain(0..overflow);
        }
    }
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
        let lines = buffer.visible_lines(1, 2, 10);
        assert_eq!(lines, vec![" two".to_string(), "three".to_string()]);
    }
}
