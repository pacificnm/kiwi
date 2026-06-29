//! PTY process runtime for the shell session (SPEC-011).
//!
//! Agent PTY spawn/restart/write paths were removed in Phase 6 (#334).
//! Agents now use the native API streaming path exclusively in kiwi_gui.

mod agent_runtime;
mod resize;

use std::path::Path;

use kiwi_core::agent::{AgentId, StreamCancelHandle};
use kiwi_core::config::ShellSettings;
use kiwi_core::events::EventSender;
use kiwi_core::shell::{ShellOutputReader, ShellSession};
use kiwi_core::state::AppState;

pub use agent_runtime::AgentRuntime;
pub use resize::effective_pty_size;

/// Default PTY dimensions until a terminal panel measures its viewport.
pub const DEFAULT_PTY_COLS: u16 = 80;
pub const DEFAULT_PTY_ROWS: u16 = 24;

pub struct PtyRuntime {
    shell: Option<ShellSession>,
    shell_io: Option<ShellOutputReader>,
    agent: AgentRuntime,
    shut_down: bool,
}

impl PtyRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            shell: None,
            shell_io: None,
            agent: AgentRuntime::new(),
            shut_down: false,
        }
    }

    /// Spawn the interactive shell at startup and attach an output reader thread.
    pub fn spawn_shell_at_startup(
        &mut self,
        repo_root: &Path,
        shell_settings: &ShellSettings,
        state: &mut AppState,
        sender: EventSender,
    ) {
        if self.shell.is_some() {
            return;
        }

        let (cols, rows) = effective_pty_size(state.viewport.shell_cols, state.viewport.shell_rows);

        match ShellSession::spawn(repo_root, shell_settings, cols, rows) {
            Ok(session) => {
                state.shell.apply_spawn(
                    &session.spec.command,
                    &session.spec.shell_name,
                    session.pid(),
                    session.cols,
                    session.rows,
                );
                if let Ok(reader) = session.try_clone_reader() {
                    self.shell_io = Some(ShellOutputReader::spawn(reader, sender));
                }
                self.shell = Some(session);
                state.logs.push_info(format!(
                    "Shell started ({} {}×{})",
                    state.shell.shell_name, state.shell.cols, state.shell.rows
                ));
            }
            Err(err) => {
                let message = err.to_string();
                state.shell.apply_spawn_error(message.clone());
                state.logs.push_info(format!("Shell failed to start: {message}"));
            }
        }
        state.dirty = true;
    }

    pub fn write_shell(&mut self, data: &[u8]) -> bool {
        let Some(shell) = self.shell.as_mut() else {
            return false;
        };
        shell.write(data).is_ok()
    }

    pub fn resize_shell(&mut self, cols: u16, rows: u16) -> bool {
        let Some(shell) = self.shell.as_mut() else {
            return false;
        };
        shell.resize(cols, rows).is_ok()
    }

    /// Register a cancel handle for a native-chat API stream, cancelling any prior stream.
    pub fn register_stream(&mut self, id: AgentId, cancel: StreamCancelHandle) {
        self.agent.register_stream(id, cancel);
    }

    /// Cancel the active API stream for the given agent, if any.
    pub fn cancel_stream(&mut self, id: AgentId) {
        self.agent.cancel_stream(id);
    }

    pub fn shutdown(&mut self) {
        if self.shut_down {
            return;
        }
        self.shut_down = true;

        if let Some(reader) = self.shell_io.take() {
            reader.abandon();
        }
        self.agent.cancel_all_streams();
        if let Some(mut shell) = self.shell.take() {
            shell.shutdown();
        }
    }
}

impl Default for PtyRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kiwi_core::config::ResolvedConfig;
    use kiwi_core::events::EventChannel;
    use kiwi_core::state::{AppState, ViewportMetrics};
    use kiwi_core::theme::{load_theme_with_capabilities, TerminalCapabilities};

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            PathBuf::from("/tmp/kiwi"),
            false,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        )
    }

    #[test]
    fn spawn_shell_at_startup_marks_running_or_records_error() {
        if !Path::new("/bin/bash").exists() && !Path::new("/usr/bin/bash").exists() {
            return;
        }

        let mut pty = PtyRuntime::new();
        let mut state = test_state();
        let channel = EventChannel::new();
        let repo = std::env::temp_dir().join("kiwi-gui-pty-shell-test");
        std::fs::create_dir_all(&repo).expect("temp dir");

        let shell_settings = state.config.shell.clone();
        pty.spawn_shell_at_startup(
            &repo,
            &shell_settings,
            &mut state,
            channel.sender(),
        );

        assert!(pty.shell.is_some() || state.shell.spawn_error.is_some());
        if pty.shell.is_some() {
            assert!(state.shell.running);
            assert_eq!(state.shell.cols, DEFAULT_PTY_COLS);
            assert_eq!(state.shell.rows, DEFAULT_PTY_ROWS);
        }
        pty.shutdown();
    }
}
