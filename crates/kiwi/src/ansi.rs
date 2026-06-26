use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Default PTY text style: host terminal colors, not Kiwi theme.
#[must_use]
pub fn pty_base_style() -> Style {
    Style::default().fg(Color::Reset).bg(Color::Reset)
}

/// Build a ratatui line from PTY text, optionally highlighting a cursor column.
#[must_use]
pub fn ansi_line_with_cursor(
    text: &str,
    max_width: usize,
    cursor_col: Option<usize>,
) -> Line<'static> {
    if max_width == 0 {
        return Line::from(String::new());
    }

    let Some(cursor_col) = cursor_col else {
        return ansi_line(text, max_width);
    };

    if cursor_col >= max_width {
        return ansi_line(text, max_width);
    }

    let mut spans = Vec::new();
    let mut style = pty_base_style();
    let mut buf = String::new();
    let mut visible = 0usize;
    let mut chars = text.chars().peekable();
    let cursor_style = style.add_modifier(Modifier::REVERSED);

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            flush_span(&mut spans, &mut buf, style);
            if chars.next_if_eq(&'[').is_some() {
                let mut params = String::new();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        if c == 'm' {
                            style = apply_sgr(&params, style);
                        }
                        break;
                    }
                    params.push(c);
                }
            }
            continue;
        }

        if visible == cursor_col {
            flush_span(&mut spans, &mut buf, style);
            if visible >= max_width {
                break;
            }
            spans.push(Span::styled(ch.to_string(), cursor_style));
            visible += 1;
            continue;
        }

        if visible >= max_width {
            break;
        }

        buf.push(ch);
        visible += 1;
    }

    flush_span(&mut spans, &mut buf, style);

    if visible == cursor_col && visible < max_width {
        spans.push(Span::styled(" ", cursor_style));
    }

    if visible >= max_width && visible_width(text) > max_width {
        append_ellipsis(&mut spans);
    }

    if spans.is_empty() {
        Line::from(String::new())
    } else {
        Line::from(spans)
    }
}

/// Build a ratatui line from PTY text, preserving SGR color codes.
#[must_use]
pub fn ansi_line(text: &str, max_width: usize) -> Line<'static> {
    if max_width == 0 {
        return Line::from(String::new());
    }

    let mut spans = Vec::new();
    let mut style = pty_base_style();
    let mut buf = String::new();
    let mut visible = 0usize;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            flush_span(&mut spans, &mut buf, style);
            if chars.next_if_eq(&'[').is_some() {
                let mut params = String::new();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        if c == 'm' {
                            style = apply_sgr(&params, style);
                        }
                        break;
                    }
                    params.push(c);
                }
            }
            continue;
        }

        if visible >= max_width {
            break;
        }

        buf.push(ch);
        visible += 1;
    }

    flush_span(&mut spans, &mut buf, style);

    if visible >= max_width && visible_width(text) > max_width {
        append_ellipsis(&mut spans);
    }

    if spans.is_empty() {
        Line::from(String::new())
    } else {
        Line::from(spans)
    }
}

#[must_use]
pub fn visible_width(text: &str) -> usize {
    strip_ansi(text).chars().count()
}

#[must_use]
pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

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

fn flush_span(spans: &mut Vec<Span<'static>>, buf: &mut String, style: Style) {
    if buf.is_empty() {
        return;
    }
    spans.push(Span::styled(std::mem::take(buf), style));
}

fn append_ellipsis(spans: &mut Vec<Span<'static>>) {
    if let Some(last) = spans.last_mut() {
        let content = last.content.clone().into_owned();
        let style = last.style;
        if content.chars().count() <= 1 {
            *last = Span::styled("…".to_string(), style);
            return;
        }
        let prefix: String = content.chars().take(content.chars().count() - 1).collect();
        *last = Span::styled(format!("{prefix}…"), style);
        return;
    }
    spans.push(Span::raw("…"));
}

fn apply_sgr(params: &str, mut style: Style) -> Style {
    let codes: Vec<u16> = params
        .split(';')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect();

    if codes.is_empty() {
        return pty_base_style();
    }

    let mut index = 0;
    while index < codes.len() {
        match codes[index] {
            0 => style = pty_base_style(),
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            22 => style = style.remove_modifier(Modifier::DIM | Modifier::BOLD),
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),
            30..=37 => style = style.fg(ansi_color(codes[index] - 30)),
            38 if index + 2 < codes.len() && codes[index + 1] == 5 => {
                style = style.fg(Color::Indexed(codes[index + 2] as u8));
                index += 2;
            }
            39 => style = style.fg(Color::Reset),
            40..=47 => style = style.bg(ansi_color(codes[index] - 40)),
            48 if index + 2 < codes.len() && codes[index + 1] == 5 => {
                style = style.bg(Color::Indexed(codes[index + 2] as u8));
                index += 2;
            }
            49 => style = style.bg(Color::Reset),
            90..=97 => style = style.fg(ansi_bright_color(codes[index] - 90)),
            100..=107 => style = style.bg(ansi_bright_color(codes[index] - 100)),
            _ => {}
        }
        index += 1;
    }

    style
}

fn ansi_color(index: u16) -> Color {
    match index {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::Gray,
        _ => Color::Reset,
    }
}

fn ansi_bright_color(index: u16) -> Color {
    match index {
        0 => Color::DarkGray,
        1 => Color::LightRed,
        2 => Color::LightGreen,
        3 => Color::LightYellow,
        4 => Color::LightBlue,
        5 => Color::LightMagenta,
        6 => Color::LightCyan,
        7 => Color::White,
        _ => Color::Reset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pty_base_style_resets_terminal_colors() {
        let style = pty_base_style();
        assert_eq!(style.fg, Some(Color::Reset));
        assert_eq!(style.bg, Some(Color::Reset));
    }

    #[test]
    fn strip_ansi_removes_color_codes() {
        assert_eq!(strip_ansi("\x1b[1;32mok\x1b[0m"), "ok");
    }

    #[test]
    fn visible_width_counts_text_without_ansi() {
        assert_eq!(visible_width("\x1b[32mgreen\x1b[0m ok"), 8);
    }

    #[test]
    fn ansi_line_preserves_green_text() {
        let line = ansi_line("\x1b[32mgreen\x1b[0m", 10);
        assert_eq!(line.spans[0].content, "green");
        assert_eq!(line.spans[0].style.fg, Some(Color::Green));
    }

    #[test]
    fn ansi_line_plain_text_uses_terminal_reset() {
        let line = ansi_line("prompt$ ", 20);
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].style.fg, Some(Color::Reset));
        assert_eq!(line.spans[0].style.bg, Some(Color::Reset));
    }

    #[test]
    fn ansi_line_supports_256_color_foreground() {
        let line = ansi_line("\x1b[38;5;214morange\x1b[0m", 20);
        assert_eq!(line.spans[0].content, "orange");
        assert_eq!(line.spans[0].style.fg, Some(Color::Indexed(214)));
    }

    #[test]
    fn ansi_line_truncates_visible_width() {
        let line = ansi_line("hello world", 5);
        assert_eq!(line.to_string(), "hell…");
    }

    #[test]
    fn ansi_line_with_cursor_reverses_character_at_column() {
        let line = ansi_line_with_cursor("prompt$ ", 20, Some(7));
        assert_eq!(line.spans[0].content, "prompt$");
        assert_eq!(line.spans[1].content, " ");
        assert!(line.spans[1]
            .style
            .add_modifier
            .contains(Modifier::REVERSED));
    }

    #[test]
    fn ansi_line_with_cursor_appends_block_at_end() {
        let line = ansi_line_with_cursor("abc", 20, Some(3));
        assert_eq!(line.spans[0].content, "abc");
        assert_eq!(line.spans[1].content, " ");
        assert!(line.spans[1]
            .style
            .add_modifier
            .contains(Modifier::REVERSED));
    }
}
