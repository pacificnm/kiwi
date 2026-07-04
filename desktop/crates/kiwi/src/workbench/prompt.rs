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
    pub fn resolve(&self) -> String {
        let mut out = self.visible.clone();
        for (index, paste) in self.pastes.iter().enumerate() {
            let token = paste_token(index, self.pastes.len());
            if out.contains(&token) {
                out = out.replace(&token, paste);
            }
        }
        out.trim().to_string()
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
    fn removed_placeholder_drops_paste() {
        let mut draft = PromptDraft::default();
        draft.add_paste("secret".into());
        draft.visible.clear();
        draft.visible.push_str("just typing");
        assert_eq!(draft.resolve(), "just typing");
    }
}
