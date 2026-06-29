//! Native chat data model for API-based agents (migration from PTY pipeline).
//!
//! This module defines the structured conversation types that replace the PTY
//! `ScrollbackBuffer` approach. See `docs/design/native-chat-agent-migration.md`.

use serde::{Deserialize, Serialize};

use super::AgentStatus;

/// Full state for one native-chat agent session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSession {
    /// Completed conversation turns (user + assistant messages).
    pub messages: Vec<ChatMessage>,
    /// Current text in the input box (not yet sent).
    pub input_draft: String,
    /// True while the API is streaming a response.
    pub is_streaming: bool,
    /// Accumulates token chunks during streaming; flushed to `messages` on turn complete.
    pub streaming_text: String,
    /// Tool call in progress (awaiting execution result).
    pub active_tool_call: Option<ToolUse>,
    /// Scroll position: index of first visible message.
    pub scroll_offset: usize,
    /// True when the view should track the newest message (bottom).
    pub follow_tail: bool,
    /// Activity status reflected in the chrome and status bar.
    pub status: AgentStatus,
    /// Last API or tool error message, shown in the panel.
    pub error: Option<String>,
    /// Cached status-bar label; refresh via [`ChatSession::refresh_status_bar_label`].
    pub status_bar_label: String,
    /// Model identifier sent with each API request (e.g. `"claude-opus-4-8"`).
    pub model: String,
    /// Which API provider backs this session.
    pub provider: AgentProvider,
}

impl Default for ChatSession {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input_draft: String::new(),
            is_streaming: false,
            streaming_text: String::new(),
            active_tool_call: None,
            scroll_offset: 0,
            follow_tail: true,
            status: AgentStatus::Idle,
            error: None,
            status_bar_label: AgentStatus::Idle.status_bar_label(false).to_string(),
            model: "claude-opus-4-8".to_string(),
            provider: AgentProvider::Claude,
        }
    }
}

impl ChatSession {
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.is_streaming || !self.messages.is_empty()
    }

    #[must_use]
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn refresh_status_bar_label(&mut self) {
        self.status_bar_label = self.status.status_bar_label(self.is_streaming).to_string();
    }

    pub fn append_user_message(&mut self, text: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text(text)],
        });
        self.follow_tail = true;
    }

    /// Flush the accumulated `streaming_text` into a completed assistant message.
    pub fn flush_streaming_turn(&mut self) {
        if !self.streaming_text.is_empty() {
            let text = std::mem::take(&mut self.streaming_text);
            if let Some(last) = self.messages.last_mut() {
                if last.role == MessageRole::Assistant {
                    last.blocks.push(ContentBlock::Text(text));
                    return;
                }
            }
            self.messages.push(ChatMessage {
                role: MessageRole::Assistant,
                blocks: vec![ContentBlock::Text(text)],
            });
        }
        self.is_streaming = false;
        self.active_tool_call = None;
    }

    pub fn append_tool_use(&mut self, tool: ToolUse) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == MessageRole::Assistant {
                last.blocks.push(ContentBlock::ToolUse(tool));
                return;
            }
        }
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            blocks: vec![ContentBlock::ToolUse(tool)],
        });
    }

    pub fn append_tool_result(&mut self, result: ToolResult) {
        // Anthropic API requires tool_result blocks to be in user messages.
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::ToolResult(result)],
        });
    }

    pub fn toggle_tool_expand(&mut self, tool_use_id: &str) {
        for message in &mut self.messages {
            for block in &mut message.blocks {
                if let ContentBlock::ToolUse(tool) = block {
                    if tool.id == tool_use_id {
                        tool.collapsed = !tool.collapsed;
                        return;
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.input_draft.clear();
        self.is_streaming = false;
        self.streaming_text.clear();
        self.active_tool_call = None;
        self.scroll_offset = 0;
        self.follow_tail = true;
        self.error = None;
        self.status = AgentStatus::Idle;
        self.refresh_status_bar_label();
    }
}

/// A single turn in the conversation (one user or assistant exchange).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub blocks: Vec<ContentBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
}

/// Content within a message: text, tool invocation, or tool result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentBlock {
    Text(String),
    ToolUse(ToolUse),
    ToolResult(ToolResult),
}

/// A tool call made by the assistant (rendered as a collapsible widget).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolUse {
    /// Correlation ID returned by the API (matches a [`ToolResult::tool_use_id`]).
    pub id: String,
    /// Tool name (e.g. `"read_file"`, `"run_bash"`).
    pub name: String,
    /// JSON-encoded input parameters.
    pub input_json: String,
    /// UI state: collapsed by default, expanded on click.
    pub collapsed: bool,
}

impl ToolUse {
    #[must_use]
    pub fn new(id: String, name: String, input_json: String) -> Self {
        Self {
            id,
            name,
            input_json,
            collapsed: true,
        }
    }
}

/// The result of executing a tool, fed back to the API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    /// Matches the [`ToolUse::id`] this result responds to.
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}

impl ToolResult {
    #[must_use]
    pub fn ok(tool_use_id: String, content: String) -> Self {
        Self {
            tool_use_id,
            content,
            is_error: false,
        }
    }

    #[must_use]
    pub fn error(tool_use_id: String, message: String) -> Self {
        Self {
            tool_use_id,
            content: message,
            is_error: true,
        }
    }
}

/// Which API provider backs an agent session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentProvider {
    /// Anthropic Claude API (`claude-*` models).
    Claude,
    /// Local Ollama instance (`http://localhost:11434` by default).
    Ollama,
    /// OpenAI API (`gpt-*` models).
    OpenAI,
    /// Cursor AI API (`cursor-small` and proxied models).
    Cursor,
    /// Legacy PTY subprocess (used when `provider = "pty"` in config).
    Pty,
}

impl Default for AgentProvider {
    fn default() -> Self {
        Self::Claude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_chat_session_is_empty_and_idle() {
        let session = ChatSession::default();
        assert!(session.messages.is_empty());
        assert!(!session.is_streaming);
        assert!(session.input_draft.is_empty());
        assert_eq!(session.status, AgentStatus::Idle);
    }

    #[test]
    fn append_user_message_adds_turn() {
        let mut session = ChatSession::default();
        session.append_user_message("hello".to_string());
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, MessageRole::User);
        assert_eq!(
            session.messages[0].blocks[0],
            ContentBlock::Text("hello".to_string())
        );
    }

    #[test]
    fn flush_streaming_turn_promotes_text_to_messages() {
        let mut session = ChatSession::default();
        session.is_streaming = true;
        session.streaming_text = "world".to_string();
        session.flush_streaming_turn();
        assert!(!session.is_streaming);
        assert!(session.streaming_text.is_empty());
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, MessageRole::Assistant);
    }

    #[test]
    fn flush_streaming_turn_appends_to_existing_assistant_message() {
        let mut session = ChatSession::default();
        session.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            blocks: vec![ContentBlock::Text("part one ".to_string())],
        });
        session.streaming_text = "part two".to_string();
        session.flush_streaming_turn();
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].blocks.len(), 2);
    }

    #[test]
    fn toggle_tool_expand_flips_collapsed() {
        let mut session = ChatSession::default();
        let tool = ToolUse::new("id1".to_string(), "read_file".to_string(), "{}".to_string());
        assert!(tool.collapsed);
        session.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            blocks: vec![ContentBlock::ToolUse(tool)],
        });
        session.toggle_tool_expand("id1");
        if let ContentBlock::ToolUse(t) = &session.messages[0].blocks[0] {
            assert!(!t.collapsed);
        } else {
            panic!("expected ToolUse block");
        }
    }

    #[test]
    fn clear_resets_session_to_empty_state() {
        let mut session = ChatSession::default();
        session.append_user_message("hi".to_string());
        session.is_streaming = true;
        session.error = Some("oops".to_string());
        session.clear();
        assert!(session.messages.is_empty());
        assert!(!session.is_streaming);
        assert!(session.error.is_none());
    }

    #[test]
    fn tool_result_constructors_set_is_error_correctly() {
        let ok = ToolResult::ok("id".to_string(), "content".to_string());
        assert!(!ok.is_error);
        let err = ToolResult::error("id".to_string(), "fail".to_string());
        assert!(err.is_error);
    }

    #[test]
    fn append_tool_result_creates_user_message() {
        let mut session = ChatSession::default();
        // Precede with an assistant message (as happens after a tool_use turn).
        session.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            blocks: vec![ContentBlock::Text("I'll use a tool.".to_string())],
        });
        let result = ToolResult::ok("id1".to_string(), "output".to_string());
        session.append_tool_result(result);
        // Must create a NEW user message — never append to the assistant message.
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[1].role, MessageRole::User);
        assert!(
            matches!(&session.messages[1].blocks[0], ContentBlock::ToolResult(r) if r.tool_use_id == "id1")
        );
    }
}
