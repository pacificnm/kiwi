//! PTY-backed integrated terminals for the bottom panel.
//!
//! Unlike [`crate::agent`] (a single agent session), this manager hosts any
//! number of concurrent shell sessions keyed by a UI-supplied id, mirroring
//! VS Code's multi-terminal panel. Output for every session is streamed to the
//! webview over [`TERMINAL_OUTPUT_EVENT`] tagged with its `id`; keystrokes and
//! resizes flow back through [`crate::commands`].

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use base64::Engine;
use nest_error::{NestError, NestResult};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

use crate::agent::augmented_path;

/// Event carrying a base64-encoded chunk of terminal output for one session.
pub const TERMINAL_OUTPUT_EVENT: &str = "kiwi://terminal-output";
/// Event fired once when a terminal's shell process exits.
pub const TERMINAL_EXIT_EVENT: &str = "kiwi://terminal-exit";

/// Payload for [`TERMINAL_OUTPUT_EVENT`].
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutput {
    /// Session id this chunk belongs to.
    pub id: String,
    /// Base64-encoded raw PTY bytes (may split UTF-8 across chunks).
    pub base64: String,
}

/// Payload for [`TERMINAL_EXIT_EVENT`].
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalExit {
    /// Session id that exited.
    pub id: String,
    /// Human-readable reason (exit status or error).
    pub message: String,
}

/// A running terminal PTY session.
struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
}

/// Managed Tauri state holding all live terminal sessions.
#[derive(Default)]
pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl TerminalManager {
    /// Spawns an interactive shell in a PTY, keyed by `id`.
    ///
    /// An existing session with the same `id` is replaced. `shell` overrides the
    /// auto-detected login shell when non-empty.
    pub fn open<R: Runtime>(
        &self,
        app: AppHandle<R>,
        id: String,
        cwd: Option<PathBuf>,
        shell: Option<&str>,
        rows: u16,
        cols: u16,
    ) -> NestResult<()> {
        self.close(&id);

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| NestError::io(format!("failed to open pty: {error}")))?;

        let shell = resolve_shell(shell);
        let mut command = CommandBuilder::new(&shell);
        configure_shell_args(&mut command, &shell);
        if let Some(dir) = cwd.filter(|dir| dir.is_dir()) {
            command.cwd(dir);
        }
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("PATH", augmented_path());
        if let Ok(home) = std::env::var("HOME") {
            command.env("HOME", home);
        }

        let mut child = pair.slave.spawn_command(command).map_err(|error| {
            tracing::error!(target: "kiwi", %error, %shell, "terminal spawn failed");
            NestError::io(format!("failed to start shell `{shell}`: {error}"))
        })?;
        drop(pair.slave);

        let master = pair.master;
        let reader = master
            .try_clone_reader()
            .map_err(|error| NestError::io(format!("failed to clone pty reader: {error}")))?;
        let writer = master
            .take_writer()
            .map_err(|error| NestError::io(format!("failed to open pty writer: {error}")))?;

        self.sessions
            .lock()
            .expect("terminal mutex")
            .insert(id.clone(), Session { master, writer });

        // Stream this session's output to the webview.
        let out_app = app.clone();
        let out_id = id.clone();
        thread::spawn(move || read_loop(reader, out_app, out_id));

        // Watch for shell exit and drop the session.
        let exit_app = app.clone();
        let exit_id = id.clone();
        let sessions = Arc::clone(&self.sessions);
        thread::spawn(move || {
            let message = match child.wait() {
                Ok(status) => format!("terminal exited ({status})"),
                Err(error) => format!("terminal wait failed: {error}"),
            };
            sessions.lock().expect("terminal mutex").remove(&exit_id);
            let _ = exit_app.emit(
                TERMINAL_EXIT_EVENT,
                TerminalExit {
                    id: exit_id,
                    message,
                },
            );
        });

        tracing::info!(target: "kiwi", %id, %shell, rows, cols, "terminal opened");
        Ok(())
    }

    /// Writes UTF-8 input (keystrokes) to a session's shell.
    pub fn input(&self, id: &str, data: &str) -> NestResult<()> {
        let mut guard = self.sessions.lock().expect("terminal mutex");
        let session = guard
            .get_mut(id)
            .ok_or_else(|| NestError::validation(format!("no terminal session: {id}")))?;
        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|error| NestError::io(format!("failed to write to terminal: {error}")))
    }

    /// Resizes a session's PTY grid.
    pub fn resize(&self, id: &str, rows: u16, cols: u16) -> NestResult<()> {
        let guard = self.sessions.lock().expect("terminal mutex");
        if let Some(session) = guard.get(id) {
            session
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|error| NestError::io(format!("failed to resize terminal: {error}")))?;
        }
        Ok(())
    }

    /// Terminates a session; dropping the master PTY hangs up the shell.
    pub fn close(&self, id: &str) {
        self.sessions.lock().expect("terminal mutex").remove(id);
    }

    /// Ids of all currently live terminal sessions.
    pub fn list(&self) -> Vec<String> {
        self.sessions
            .lock()
            .expect("terminal mutex")
            .keys()
            .cloned()
            .collect()
    }
}

/// Picks the shell to launch: explicit override → `$SHELL` → platform default.
fn resolve_shell(shell: Option<&str>) -> String {
    if let Some(explicit) = shell.map(str::trim).filter(|value| !value.is_empty()) {
        return explicit.to_string();
    }
    if let Ok(env_shell) = std::env::var("SHELL") {
        if !env_shell.trim().is_empty() {
            return env_shell;
        }
    }
    #[cfg(windows)]
    {
        "powershell.exe".to_string()
    }
    #[cfg(not(windows))]
    {
        if std::path::Path::new("/bin/bash").exists() {
            "/bin/bash".to_string()
        } else {
            "/bin/sh".to_string()
        }
    }
}

/// Adds interactive-login flags so the shell sources rc files where sensible.
fn configure_shell_args(command: &mut CommandBuilder, shell: &str) {
    let name = std::path::Path::new(shell)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(shell);
    match name {
        "bash" | "zsh" | "sh" => command.arg("-l"),
        _ => {}
    }
}

fn read_loop<R: Runtime>(mut reader: Box<dyn Read + Send>, app: AppHandle<R>, id: String) {
    let engine = base64::engine::general_purpose::STANDARD;
    let mut buffer = [0_u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                let base64 = engine.encode(&buffer[..count]);
                let payload = TerminalOutput {
                    id: id.clone(),
                    base64,
                };
                if app.emit(TERMINAL_OUTPUT_EVENT, payload).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
