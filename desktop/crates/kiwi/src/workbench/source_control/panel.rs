//! Git bottom panel — command output log and commit history.

use std::time::{SystemTime, UNIX_EPOCH};

const MAX_OUTPUT_ENTRIES: usize = 200;

/// Output or history view inside the bottom Git panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitPanelView {
    /// Scrollable git command output.
    #[default]
    Output,
    /// Recent commits from `git log`.
    History,
}

/// One line in the git command output log.
#[derive(Debug, Clone)]
pub struct GitOutputEntry {
    /// Local time label (HH:MM:SS UTC).
    pub time: String,
    /// Command that was run.
    pub command: String,
    /// Whether the command succeeded.
    pub success: bool,
    /// Combined stdout/stderr or error text.
    pub text: String,
}

/// Append-only git command output for the bottom panel.
#[derive(Debug, Clone, Default)]
pub struct GitOutputLog {
    entries: Vec<GitOutputEntry>,
}

impl GitOutputLog {
    /// Records one git command result.
    pub fn push(&mut self, command: String, success: bool, text: String) {
        if self.entries.len() >= MAX_OUTPUT_ENTRIES {
            self.entries.remove(0);
        }
        self.entries.push(GitOutputEntry {
            time: utc_time_label(),
            command,
            success,
            text,
        });
    }

    /// All log entries oldest-first.
    pub fn entries(&self) -> &[GitOutputEntry] {
        &self.entries
    }

    /// Clears the output log.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

fn utc_time_label() -> String {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return "??:??:??".into();
    };
    let total = duration.as_secs();
    let hours = (total / 3600) % 24;
    let minutes = (total / 60) % 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_evicts_oldest_entry_at_capacity() {
        let mut log = GitOutputLog::default();
        for index in 0..MAX_OUTPUT_ENTRIES + 5 {
            log.push(format!("cmd {index}"), true, "ok".into());
        }
        assert_eq!(log.entries().len(), MAX_OUTPUT_ENTRIES);
        assert!(log.entries()[0].command.contains("5"));
    }
}
