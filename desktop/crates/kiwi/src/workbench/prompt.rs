//! AI prompt input with collapsed large pastes (Cursor-style).

use egui::{Event, Ui};

/// Paste larger than this is collapsed to a placeholder in the input.
pub const LARGE_PASTE_CHARS: usize = 500;
/// Paste with more lines than this is collapsed.
pub const LARGE_PASTE_LINES: usize = 8;

/// Fixed height for the multiline prompt field (~3 lines).
pub const PROMPT_INPUT_HEIGHT: f32 = 72.0;

/// Chat prompt draft: visible text may contain `[Pasted item]` placeholders.
#[derive(Debug, Clone, Default)]
pub struct PromptDraft {
    /// Text shown in the input (may include paste placeholders).
    pub visible: String,
    /// Full pasted bodies referenced by placeholders, in order.
    pastes: Vec<String>,
}

impl PromptDraft {
    /// Whether the draft has no sendable content.
    pub fn is_empty(&self) -> bool {
        self.resolve().is_empty()
    }

    /// Clears visible text and stored pastes.
    pub fn clear(&mut self) {
        self.visible.clear();
        self.pastes.clear();
    }

    /// Intercepts large clipboard pastes before they reach the text edit.
    pub fn consume_large_pastes(&mut self, ui: &mut Ui) {
        let mut consumed = Vec::new();
        ui.input_mut(|input| {
            input.events.retain(|event| {
                if let Event::Paste(text) = event {
                    if is_large_paste(text) {
                        consumed.push(text.clone());
                        return false;
                    }
                }
                true
            });
        });

        for text in consumed {
            self.add_paste(text);
        }
    }

    /// Collapses pasted content that landed in the field without interception.
    pub fn collapse_oversized_visible(&mut self) {
        if self.pastes.is_empty() && is_large_paste(&self.visible) {
            let content = std::mem::take(&mut self.visible);
            self.add_paste(content);
        }
    }

    /// Resolves placeholders into the message sent to the model.
    ///
    /// Attachments whose `[Pasted item]` placeholder was removed from the visible
    /// draft are appended at the end so drag-and-drop files are never dropped silently.
    pub fn resolve(&self) -> String {
        let mut out = self.visible.clone();
        let mut used = vec![false; self.pastes.len()];

        for (index, paste) in self.pastes.iter().enumerate() {
            let token = paste_token(index, self.pastes.len());
            if out.contains(&token) {
                out = out.replace(&token, paste);
                used[index] = true;
            }
        }

        for (index, paste) in self.pastes.iter().enumerate() {
            if used[index] {
                continue;
            }
            if !out.trim().is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(paste);
        }

        out.trim().to_string()
    }

    /// Attaches an editor file for the agent/chat (large bodies collapse to a placeholder).
    pub fn attach_file(&mut self, rel_path: &str, content: &str) {
        let body = if content.is_empty() {
            format!("<file path=\"{rel_path}\">\n(empty file)\n</file>")
        } else {
            format!("<file path=\"{rel_path}\">\n{content}\n</file>")
        };
        self.attach_context(body, &format!("Questions about `{rel_path}`?"));
    }

    /// Attaches a GitHub issue for the agent/chat.
    pub fn attach_issue(&mut self, number: u64, title: &str, content: &str) {
        let body = format!(
            "<github-issue number=\"{number}\" title=\"{title}\">\n{content}\n</github-issue>"
        );
        self.attach_context(
            body,
            &format!("Help me with GitHub issue #{number}: {title}"),
        );
    }

    fn attach_context(&mut self, body: String, seed_prompt: &str) {
        self.pastes.push(body);
        let token = paste_token(self.pastes.len() - 1, self.pastes.len());
        if self.visible.trim().is_empty() {
            self.visible = format!("{seed_prompt}\n{token}");
        } else if !self.visible.ends_with('\n') {
            self.visible.push('\n');
            self.visible.push_str(&token);
        } else {
            self.visible.push_str(&token);
        }
    }

    fn add_paste(&mut self, content: String) {
        self.pastes.push(content);
        let token = paste_token(self.pastes.len() - 1, self.pastes.len());
        if self.visible.is_empty() {
            self.visible = token;
        } else if self.visible.ends_with('\n') {
            self.visible.push_str(&token);
        } else {
            self.visible.push('\n');
            self.visible.push_str(&token);
        }
    }
}

fn is_large_paste(text: &str) -> bool {
    text.len() > LARGE_PASTE_CHARS || text.lines().count() > LARGE_PASTE_LINES
}

fn paste_token(index: usize, total: usize) -> String {
    if total == 1 {
        "[Pasted item]".into()
    } else {
        format!("[Pasted item #{}]", index + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_single_paste_placeholder() {
        let mut draft = PromptDraft::default();
        draft.add_paste("hello world".into());
        assert_eq!(draft.visible, "[Pasted item]");
        assert_eq!(draft.resolve(), "hello world");
    }

    #[test]
    fn resolve_paste_with_surrounding_text() {
        let mut draft = PromptDraft::default();
        draft.visible = "Explain this:\n".into();
        draft.add_paste("fn main() {}".into());
        assert_eq!(draft.resolve(), "Explain this:\nfn main() {}");
    }

    #[test]
    fn removed_placeholder_still_sends_paste() {
        let mut draft = PromptDraft::default();
        draft.add_paste("secret".into());
        draft.visible.clear();
        draft.visible.push_str("just typing");
        assert_eq!(draft.resolve(), "just typing\n\nsecret");
    }

    #[test]
    fn attach_file_survives_prompt_rewrite() {
        let mut draft = PromptDraft::default();
        draft.attach_file("README.md", "# Hello");
        draft.visible = "read and summarize the readme".into();
        let resolved = draft.resolve();
        assert!(resolved.starts_with("read and summarize the readme"));
        assert!(resolved.contains("<file path=\"README.md\">"));
        assert!(resolved.contains("# Hello"));
    }

    #[test]
    fn attach_file_seeds_prompt_when_empty() {
        let mut draft = PromptDraft::default();
        draft.attach_file("src/main.rs", "fn main() {}");
        assert!(draft.visible.contains("src/main.rs"));
        assert!(draft.resolve().contains("fn main() {}"));
    }

    #[test]
    fn attach_issue_seeds_prompt_with_issue_context() {
        let mut draft = PromptDraft::default();
        draft.attach_issue(42, "Fix bug", "# Fix bug\n\nDetails here.");
        assert!(draft.visible.contains("issue #42"));
        let resolved = draft.resolve();
        assert!(resolved.contains("<github-issue"));
        assert!(resolved.contains("Details here."));
    }
}
