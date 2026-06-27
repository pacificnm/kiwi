//! Parse PTY ANSI SGR sequences into egui [`LayoutJob`] sections.

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Stroke, TextStyle};
use kiwi_core::ansi::visible_width;

#[derive(Debug, Clone, Copy, Default)]
struct PtyStyle {
    fg: Option<Color32>,
    bg: Option<Color32>,
    bold: bool,
    italic: bool,
    underline: bool,
}

/// Build a monospace layout job from PTY text with ANSI colors preserved.
#[must_use]
pub fn ansi_layout_job(text: &str, max_width: usize, font_id: FontId) -> LayoutJob {
    if max_width == 0 {
        return LayoutJob::default();
    }

    let mut job = LayoutJob::default();
    let mut style = PtyStyle::default();
    let mut buf = String::new();
    let mut visible = 0usize;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            flush_span(&mut job, &mut buf, style, &font_id);
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

    flush_span(&mut job, &mut buf, style, &font_id);

    if visible >= max_width && visible_width(text) > max_width {
        append_ellipsis(&mut job, style, &font_id);
    }

    job
}

pub fn monospace_font_id(ui: &egui::Ui, theme_font_size: f32) -> FontId {
    let style = ui.style();
    let resolved = TextStyle::Monospace.resolve(style);
    if resolved.size > 0.0 {
        resolved
    } else {
        FontId::monospace(theme_font_size)
    }
}

#[must_use]
pub fn max_cols_for_ui(ui: &egui::Ui, font_id: &FontId) -> usize {
    let width = ui.clip_rect().width().max(1.0);
    ui.fonts(|fonts| {
        let char_width = fonts.glyph_width(font_id, 'm').max(1.0);
        (width / char_width).floor().max(1.0) as usize
    })
}

fn flush_span(job: &mut LayoutJob, buf: &mut String, style: PtyStyle, font_id: &FontId) {
    if buf.is_empty() {
        return;
    }
    let color = style.fg.unwrap_or(default_pty_fg());
    job.append(
        &std::mem::take(buf),
        0.0,
        TextFormat {
            font_id: font_id.clone(),
            color,
            background: style.bg.unwrap_or(Color32::TRANSPARENT),
            italics: style.italic,
            underline: if style.underline {
                Stroke::new(1.0, color)
            } else {
                Stroke::NONE
            },
            ..Default::default()
        },
    );
}

fn append_ellipsis(job: &mut LayoutJob, style: PtyStyle, font_id: &FontId) {
    if let Some(section) = job.sections.last_mut() {
        let start = section.byte_range.start;
        let end = section.byte_range.end;
        if end > start {
            let prefix: String = job.text[start..end]
                .chars()
                .take(job.text[start..end].chars().count().saturating_sub(1))
                .collect();
            job.text.replace_range(start..end, &prefix);
            section.byte_range.end = start + prefix.len();
        }
    }
    let color = style.fg.unwrap_or(default_pty_fg());
    job.append(
        "…",
        0.0,
        TextFormat {
            font_id: font_id.clone(),
            color,
            ..Default::default()
        },
    );
}

fn apply_sgr(params: &str, mut style: PtyStyle) -> PtyStyle {
    let codes: Vec<u16> = params
        .split(';')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect();

    if codes.is_empty() {
        return PtyStyle::default();
    }

    let mut index = 0;
    while index < codes.len() {
        match codes[index] {
            0 => style = PtyStyle::default(),
            1 => style.bold = true,
            3 => style.italic = true,
            4 => style.underline = true,
            22 => style.bold = false,
            23 => style.italic = false,
            24 => style.underline = false,
            30..=37 => style.fg = Some(ansi_color(codes[index] - 30)),
            38 if index + 2 < codes.len() && codes[index + 1] == 5 => {
                style.fg = Some(xterm_color(codes[index + 2]));
                index += 2;
            }
            39 => style.fg = None,
            40..=47 => style.bg = Some(ansi_color(codes[index] - 40)),
            48 if index + 2 < codes.len() && codes[index + 1] == 5 => {
                style.bg = Some(xterm_color(codes[index + 2]));
                index += 2;
            }
            49 => style.bg = None,
            90..=97 => style.fg = Some(ansi_bright_color(codes[index] - 90)),
            100..=107 => style.bg = Some(ansi_bright_color(codes[index] - 100)),
            _ => {}
        }
        index += 1;
    }

    style
}

fn default_pty_fg() -> Color32 {
    Color32::from_gray(220)
}

fn ansi_color(index: u16) -> Color32 {
    match index {
        0 => Color32::from_gray(30),
        1 => Color32::from_rgb(205, 49, 49),
        2 => Color32::from_rgb(13, 188, 121),
        3 => Color32::from_rgb(229, 229, 16),
        4 => Color32::from_rgb(36, 114, 200),
        5 => Color32::from_rgb(188, 63, 188),
        6 => Color32::from_rgb(17, 168, 205),
        7 => Color32::from_gray(180),
        _ => default_pty_fg(),
    }
}

fn ansi_bright_color(index: u16) -> Color32 {
    match index {
        0 => Color32::from_gray(120),
        1 => Color32::from_rgb(241, 76, 76),
        2 => Color32::from_rgb(35, 209, 139),
        3 => Color32::from_rgb(245, 245, 67),
        4 => Color32::from_rgb(59, 142, 234),
        5 => Color32::from_rgb(214, 112, 214),
        6 => Color32::from_rgb(41, 184, 219),
        7 => Color32::from_gray(230),
        _ => default_pty_fg(),
    }
}

fn xterm_color(index: u16) -> Color32 {
    if index < 16 {
        return if index < 8 {
            ansi_color(index)
        } else {
            ansi_bright_color(index - 8)
        };
    }
    if index < 232 {
        let index = index - 16;
        let r = index / 36;
        let g = (index / 6) % 6;
        let b = index % 6;
        let scale = |step: u16| -> u8 {
            if step == 0 {
                0
            } else {
                (55 + step * 40).min(255) as u8
            }
        };
        return Color32::from_rgb(scale(r), scale(g), scale(b));
    }
    let gray = (index - 232) * 10 + 8;
    Color32::from_gray(gray.min(255) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_layout_preserves_green_text() {
        let job = ansi_layout_job("\x1b[32mgreen\x1b[0m", 20, FontId::monospace(14.0));
        assert_eq!(job.sections.len(), 1);
        assert_eq!(&job.text[job.sections[0].byte_range.clone()], "green");
        assert_eq!(job.sections[0].format.color, ansi_color(2));
    }

    #[test]
    fn ansi_layout_truncates_visible_width() {
        let job = ansi_layout_job("hello world", 5, FontId::monospace(14.0));
        assert!(job.text.starts_with("hell"));
        assert!(job.text.contains('…'));
    }
}
