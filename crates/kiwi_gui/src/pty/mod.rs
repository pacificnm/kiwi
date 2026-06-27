//! PTY process runtime for shell and agent sessions (SPEC-010 / SPEC-011, Phase 1).

mod agent_runtime;
mod resize;

use std::path::Path;

use kiwi_core::agent::{AgentId, AgentSession};
use kiwi_core::config::{AgentSettings, ResolvedConfig, ShellSettings};
use kiwi_core::events::EventSender;
use kiwi_core::navigation::MainTab;
use kiwi_core::shell::{ShellOutputReader, ShellSession};
use kiwi_core::state::AppState;

pub use agent_runtime::AgentRuntime;
pub use resize::effective_pty_size;

/// Default PTY dimensions until a terminal panel measures its viewport (Phase 4).
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

    pub fn spawn_agent(
        &mut self,
        id: AgentId,
        repo_root: &Path,
        agent_settings: &AgentSettings,
        state: &mut AppState,
        sender: EventSender,
    ) {
        if state
            .agent_manager
            .pty(id)
            .is_some_and(|pty| pty.spawned)
            || self.agent.has_session(id)
        {
            return;
        }

        let (cols, rows) = effective_pty_size(state.viewport.agent_cols, state.viewport.agent_rows);

        match AgentSession::spawn(repo_root, agent_settings, cols, rows) {
            Ok(session) => {
                if let Some(pty) = state.agent_manager.pty_mut(id) {
                    pty.apply_spawn(
                        &session.spec.command,
                        &session.spec.agent_name,
                        session.pid(),
                        session.cols,
                        session.rows,
                    );
                }
                if let Ok(reader) = session.try_clone_reader() {
                    self.agent.attach_reader(id, reader, sender);
                }
                self.agent.attach_session(id, session);
            }
            Err(err) => {
                if let Some(pty) = state.agent_manager.pty_mut(id) {
                    pty.apply_spawn_error(err.to_string());
                }
            }
        }
        state.dirty = true;
    }

    pub fn restart_agent(
        &mut self,
        id: AgentId,
        repo_root: &Path,
        config: &ResolvedConfig,
        state: &mut AppState,
        sender: EventSender,
    ) {
        if state.navigation.main_tab != MainTab::Agent {
            return;
        }

        self.agent.shutdown(id);
        if let Some(pty) = state.agent_manager.pty_mut(id) {
            pty.prepare_restart();
        }
        self.spawn_agent(id, repo_root, &config.agent, state, sender);
    }

    pub fn write_shell(&mut self, data: &[u8]) -> bool {
        let Some(shell) = self.shell.as_mut() else {
            return false;
        };
        shell.write(data).is_ok()
    }

    pub fn write_agent(&mut self, id: AgentId, data: &[u8]) -> bool {
        self.agent.write(id, data)
    }

    pub fn resize_shell(&mut self, cols: u16, rows: u16) -> bool {
        let Some(shell) = self.shell.as_mut() else {
            return false;
        };
        shell.resize(cols, rows).is_ok()
    }

    pub fn resize_agent(&mut self, id: AgentId, cols: u16, rows: u16) -> bool {
        self.agent.resize(id, cols, rows)
    }

    #[must_use]
    pub fn poll_agent_exits(&mut self) -> Vec<(AgentId, i32)> {
        self.agent.poll_exits()
    }

    pub fn shutdown(&mut self) {
        if self.shut_down {
            return;
        }
        self.shut_down = true;

        if let Some(reader) = self.shell_io.take() {
            reader.abandon();
        }
        self.agent.shutdown_all();
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
