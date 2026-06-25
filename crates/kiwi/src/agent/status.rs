use crate::shell::ScrollbackBuffer;
use crate::theme::SemanticRole;

/// Agent activity inferred from PTY output (SPEC-010).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentStatus {
    #[default]
    Idle,
    Thinking,
    Executing,
    Success,
    Error,
    Warning,
}

impl AgentStatus {
    #[must_use]
    pub const fn status_bar_label(self, running: bool) -> &'static str {
        match self {
            Self::Idle if running => "Agent Running",
            Self::Idle => "Agent Idle",
            Self::Thinking => "Agent Thinking",
            Self::Executing => "Agent Executing",
            Self::Success => "Agent Success",
            Self::Error => "Agent Error",
            Self::Warning => "Agent Warning",
        }
    }

    #[must_use]
    pub const fn semantic_role(self) -> SemanticRole {
        match self {
            Self::Idle => SemanticRole::Muted,
            Self::Thinking => SemanticRole::AgentThinking,
            Self::Executing => SemanticRole::AgentExecuting,
            Self::Success => SemanticRole::AgentSuccess,
            Self::Error => SemanticRole::AgentError,
            Self::Warning => SemanticRole::AgentWarning,
        }
    }

    #[must_use]
    pub const fn from_exit_code(code: i32) -> Self {
        if code == 0 {
            Self::Success
        } else {
            Self::Error
        }
    }
}

/// Scan recent scrollback text for status keywords (patterns configurable later).
#[must_use]
pub fn infer_status_from_scrollback(buffer: &ScrollbackBuffer) -> Option<AgentStatus> {
    infer_status_from_text(&buffer.recent_text(32))
}

#[must_use]
pub fn infer_status_from_text(text: &str) -> Option<AgentStatus> {
    if text.is_empty() {
        return None;
    }

    let lower = text.to_ascii_lowercase();

    if matches_pattern(
        &lower,
        &[
            "error:",
            "error ",
            " failed",
            "failed:",
            "failure",
            "fatal",
            "panic:",
            "exception",
        ],
    ) {
        return Some(AgentStatus::Error);
    }

    if matches_pattern(
        &lower,
        &["warning:", " warn:", "warn ", "caution:", "deprecated"],
    ) {
        return Some(AgentStatus::Warning);
    }

    if matches_pattern(
        &lower,
        &[
            "success",
            "completed",
            "finished",
            "all done",
            "task complete",
            "✓ done",
            "done ✓",
        ],
    ) {
        return Some(AgentStatus::Success);
    }

    if matches_pattern(
        &lower,
        &[
            "thinking",
            "planning",
            "reasoning",
            "considering",
            "reflecting",
        ],
    ) {
        return Some(AgentStatus::Thinking);
    }

    if matches_pattern(
        &lower,
        &[
            "executing",
            "running tool",
            "tool call",
            "calling tool",
            "invoke tool",
            "apply_patch",
            "run_terminal",
            "grep ",
            "read tool",
            "write tool",
            "shell tool",
            "terminal command",
        ],
    ) {
        return Some(AgentStatus::Executing);
    }

    None
}

fn matches_pattern(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_status_detects_thinking() {
        assert_eq!(
            infer_status_from_text("⠋ Thinking about next steps...  "),
            Some(AgentStatus::Thinking)
        );
    }

    #[test]
    fn infer_status_detects_executing_tool() {
        assert_eq!(
            infer_status_from_text("Running tool: grep pattern src/"),
            Some(AgentStatus::Executing)
        );
    }

    #[test]
    fn infer_status_detects_error_before_executing() {
        assert_eq!(
            infer_status_from_text("Error: command failed while running tool"),
            Some(AgentStatus::Error)
        );
    }

    #[test]
    fn infer_status_detects_warning() {
        assert_eq!(
            infer_status_from_text("Warning: large diff ahead"),
            Some(AgentStatus::Warning)
        );
    }

    #[test]
    fn infer_status_detects_success() {
        assert_eq!(
            infer_status_from_text("Task completed successfully."),
            Some(AgentStatus::Success)
        );
    }

    #[test]
    fn infer_status_returns_none_for_ambiguous_output() {
        assert_eq!(infer_status_from_text("user@host:~/repo$ "), None);
    }

    #[test]
    fn status_bar_label_uses_running_fallback_for_idle() {
        assert_eq!(AgentStatus::Idle.status_bar_label(true), "Agent Running");
        assert_eq!(AgentStatus::Idle.status_bar_label(false), "Agent Idle");
    }

    #[test]
    fn from_exit_code_maps_zero_and_nonzero() {
        assert_eq!(AgentStatus::from_exit_code(0), AgentStatus::Success);
        assert_eq!(AgentStatus::from_exit_code(1), AgentStatus::Error);
    }

    #[test]
    fn infer_status_from_scrollback_uses_recent_lines() {
        let mut buffer = ScrollbackBuffer::new();
        buffer.append_bytes(b"old prompt\n");
        buffer.append_bytes(b"Running tool: ls\n");

        assert_eq!(
            infer_status_from_scrollback(&buffer),
            Some(AgentStatus::Executing)
        );
    }
}
