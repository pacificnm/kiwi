use crate::ansi::strip_ansi;
use crate::navigation::MainTab;
use crate::shell::ScrollbackBuffer;
use crate::state::AppState;

use super::state::{SelectionPane, TextPosition, TextSelection};

impl TextSelection {
    pub fn extract_text(&self, state: &AppState) -> Option<String> {
        if !self.has_highlight() {
            return None;
        }

        let pane = self.pane?;
        let (start, end) = self.normalized();
        let lines = lines_for_pane(state, pane)?;
        extract_range(&lines, start, end)
    }
}

fn lines_for_pane(state: &AppState, pane: SelectionPane) -> Option<Vec<String>> {
    match pane {
        SelectionPane::Preview => {
            if state.navigation.main_tab != MainTab::Preview {
                return None;
            }
            Some(state.preview.lines.clone())
        }
        SelectionPane::IssueDetail => {
            if state.navigation.main_tab != MainTab::Issues {
                return None;
            }
            state
                .github
                .issue_detail
                .as_ref()
                .map(|detail| detail.display_lines.clone())
        }
        SelectionPane::Agent => {
            if state.navigation.main_tab != MainTab::Agent {
                return None;
            }
            Some(scrollback_plain_lines(
                &state.agent.scrollback,
                state.agent.follow_tail,
                state.agent.viewport_offset,
                agent_visible_rows(state),
                agent_visible_cols(state),
            ))
        }
        SelectionPane::Shell => Some(scrollback_plain_lines(
            &state.shell.scrollback,
            state.shell.follow_tail,
            state.shell.viewport_offset,
            shell_visible_rows(state),
            shell_visible_cols(state),
        )),
    }
}

fn scrollback_plain_lines(
    scrollback: &ScrollbackBuffer,
    follow_tail: bool,
    viewport_offset: usize,
    visible_height: usize,
    max_width: usize,
) -> Vec<String> {
    if visible_height == 0 || max_width == 0 {
        return Vec::new();
    }

    let start = scrollback.viewport_start(visible_height, follow_tail, viewport_offset);
    scrollback
        .viewport_lines(start, visible_height, max_width, follow_tail)
        .into_iter()
        .map(|line| strip_ansi(&line))
        .collect()
}

fn extract_range(lines: &[String], start: TextPosition, end: TextPosition) -> Option<String> {
    if lines.is_empty() {
        return None;
    }

    if start.line >= lines.len() {
        return None;
    }

    if start.line == end.line {
        let line = &lines[start.line];
        let slice = slice_chars(line, start.col, end.col);
        return if slice.is_empty() { None } else { Some(slice) };
    }

    let mut out = slice_chars(&lines[start.line], start.col, char_len(&lines[start.line]));
    for line in lines.iter().take(end.line).skip(start.line + 1) {
        out.push('\n');
        out.push_str(line);
    }
    if end.line < lines.len() {
        out.push('\n');
        out.push_str(&slice_chars(&lines[end.line], 0, end.col));
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn slice_chars(text: &str, start_col: usize, end_col: usize) -> String {
    let start = start_col.min(char_len(text));
    let end = end_col.max(start).min(char_len(text));
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn char_len(text: &str) -> usize {
    text.chars().count()
}

fn shell_visible_rows(state: &AppState) -> usize {
    let inner_h = state.layout.rects.shell.height.saturating_sub(2) as usize;
    inner_h.saturating_sub(1)
}

fn shell_visible_cols(state: &AppState) -> usize {
    state.layout.rects.shell.width.saturating_sub(2) as usize
}

fn agent_visible_rows(state: &AppState) -> usize {
    let inner_h = state.layout.rects.main_content.height.saturating_sub(2) as usize;
    let footer = usize::from(state.agent.restart_hint.is_some());
    inner_h.saturating_sub(footer)
}

fn agent_visible_cols(state: &AppState) -> usize {
    state.layout.rects.main_content.width.saturating_sub(2) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_single_line_range() {
        let lines = vec!["hello world".to_string()];
        let text = extract_range(
            &lines,
            TextPosition { line: 0, col: 0 },
            TextPosition { line: 0, col: 5 },
        )
        .expect("slice");
        assert_eq!(text, "hello");
    }

    #[test]
    fn extract_multiline_range() {
        let lines = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string()];
        let text = extract_range(
            &lines,
            TextPosition { line: 0, col: 1 },
            TextPosition { line: 2, col: 2 },
        )
        .expect("range");
        assert_eq!(text, "aa\nbbb\ncc");
    }
}
