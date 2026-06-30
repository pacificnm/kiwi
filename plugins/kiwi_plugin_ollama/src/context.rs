use crate::ollama::ChatMessage;

const MAX_HISTORY_TURNS: usize = 20;

const SYSTEM_PROMPT: &str = "\
You are an expert programming assistant embedded in Kiwi, a terminal AI workspace.

When you begin reasoning about a problem, output a line starting with \"thinking: \" followed by what you are considering.
When you search code, read files, or take any action, output a line starting with \"running tool: \" followed by a brief description.
When you finish answering, output a line starting with \"completed: \" followed by a short summary.

Provide precise, idiomatic code. Explain changes concisely. Prefer editing existing code over rewriting from scratch.";

pub struct ConversationContext {
    messages: Vec<ChatMessage>,
    max_history_turns: usize,
}

impl ConversationContext {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_history_turns: MAX_HISTORY_TURNS,
        }
    }

    pub fn push_user(&mut self, content: String) {
        self.messages.push(ChatMessage::user(content));
        self.trim_history();
    }

    pub fn push_assistant(&mut self, content: String) {
        self.messages.push(ChatMessage::assistant(content));
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn pop_last(&mut self) {
        self.messages.pop();
    }

    /// Returns the content of the most recent user message, if any.
    pub fn last_user_message(&self) -> Option<&str> {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
    }

    /// Assembles the full message list for /api/chat.
    /// RAG context, if provided, is injected as a temporary exchange before
    /// the real conversation history so it is not persisted across turns.
    pub fn build_messages(&self, rag_context: Option<&str>) -> Vec<ChatMessage> {
        let mut out = Vec::new();

        out.push(ChatMessage::system(SYSTEM_PROMPT));

        if let Some(ctx) = rag_context {
            out.push(ChatMessage::user(format!(
                "[Relevant context — use this to inform your answer]\n\n{ctx}"
            )));
            out.push(ChatMessage::assistant(
                "I have reviewed the provided context and will use it to answer.",
            ));
        }

        out.extend(self.messages.iter().cloned());
        out
    }

    fn trim_history(&mut self) {
        let max_messages = self.max_history_turns * 2;
        if self.messages.len() > max_messages {
            let excess = self.messages.len() - max_messages;
            self.messages.drain(0..excess);
        }
    }
}
