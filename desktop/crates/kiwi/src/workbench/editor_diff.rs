//! Colorized unified git diff rendering for read-only editor tabs.

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, Style, TextEdit, TextStyle, Ui};

use crate::theme::PALETTE;

/// Read-only diff view with per-line add/remove/hunk coloring.
pub fn colorized_diff_editor(ui: &mut Ui, content: &mut String) -> egui::Response {
    let style = ui.style().clone();
    let mut layouter = move |ui: &Ui, text: &str, wrap_width: f32| {
        let mut job = build_diff_layout(text, &style);
        job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(job))
    };

    ui.add(
        TextEdit::multiline(content)
            .font(TextStyle::Monospace)
            .desired_width(f32::INFINITY)
            .interactive(false)
            .frame(false)
            .layouter(&mut layouter),
    )
}

fn build_diff_layout(text: &str, style: &Style) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font = TextStyle::Monospace.resolve(style);

    for line in text.split_inclusive('\n') {
        let kind = classify_diff_line(line);
        let (fg, bg) = diff_line_style(kind);
        job.append(
            line,
            0.0,
            TextFormat {
                font_id: font.clone(),
                color: fg,
                background: bg,
                ..Default::default()
            },
        );
    }

    job
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiffLineKind {
    Addition,
    Deletion,
    Context,
    HunkHeader,
    FileHeader,
    Meta,
    Plain,
}

fn classify_diff_line(line: &str) -> DiffLineKind {
    if line.starts_with("@@") {
        return DiffLineKind::HunkHeader;
    }
    if line.starts_with("+++") || line.starts_with("---") {
        return DiffLineKind::FileHeader;
    }
    if line.starts_with('+') {
        return DiffLineKind::Addition;
    }
    if line.starts_with('-') {
        return DiffLineKind::Deletion;
    }
    if line.starts_with(' ') {
        return DiffLineKind::Context;
    }
    if line.starts_with("diff ")
        || line.starts_with("index ")
        || line.starts_with("new file mode")
        || line.starts_with("deleted file mode")
        || line.starts_with("similarity index")
        || line.starts_with("rename from")
        || line.starts_with("rename to")
        || line.starts_with("old mode")
        || line.starts_with("new mode")
        || line.starts_with("\\ No newline")
    {
        return DiffLineKind::Meta;
    }
    DiffLineKind::Plain
}

fn diff_line_style(kind: DiffLineKind) -> (Color32, Color32) {
    match kind {
        DiffLineKind::Addition => (PALETTE.success, tint(PALETTE.success, 0.14)),
        DiffLineKind::Deletion => (PALETTE.error, tint(PALETTE.error, 0.14)),
        DiffLineKind::HunkHeader => (PALETTE.info, Color32::TRANSPARENT),
        DiffLineKind::FileHeader => (PALETTE.text_secondary, Color32::TRANSPARENT),
        DiffLineKind::Context => (PALETTE.text_primary, Color32::TRANSPARENT),
        DiffLineKind::Meta => (PALETTE.text_muted, Color32::TRANSPARENT),
        DiffLineKind::Plain => (PALETTE.text_primary, Color32::TRANSPARENT),
    }
}

fn tint(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (alpha.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_hunk_and_changes() {
        assert_eq!(classify_diff_line("@@ -1,3 +1,4 @@"), DiffLineKind::HunkHeader);
        assert_eq!(classify_diff_line("+added line\n"), DiffLineKind::Addition);
        assert_eq!(classify_diff_line("-removed line\n"), DiffLineKind::Deletion);
        assert_eq!(classify_diff_line(" context line\n"), DiffLineKind::Context);
    }

    #[test]
    fn classifies_file_headers_before_change_lines() {
        assert_eq!(classify_diff_line("--- a/file.rs\n"), DiffLineKind::FileHeader);
        assert_eq!(classify_diff_line("+++ b/file.rs\n"), DiffLineKind::FileHeader);
    }

    #[test]
    fn classifies_git_meta_lines() {
        assert_eq!(
            classify_diff_line("diff --git a/x b/x\n"),
            DiffLineKind::Meta
        );
        assert_eq!(
            classify_diff_line("index 1234567..89abcde 100644\n"),
            DiffLineKind::Meta
        );
    }
}
