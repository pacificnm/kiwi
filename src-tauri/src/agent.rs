//! PTY-backed agent runtime for the Agent Panel.
//!
//! Runs an external coding agent (Claude Code, Codex, OpenCode, …) launched via
//! `ollama launch <runtime> --model <model>` inside a pseudo-terminal. Output is
//! streamed to the webview as base64 chunks over the [`AGENT_OUTPUT_EVENT`]
//! event; keystrokes flow back through [`crate::commands`].

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use base64::Engine;
use nest_error::{NestError, NestResult};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

/// Event carrying a base64-encoded chunk of agent PTY output.
pub const AGENT_OUTPUT_EVENT: &str = "kiwi://agent-output";
/// Event fired once when the agent process exits.
pub const AGENT_EXIT_EVENT: &str = "kiwi://agent-exit";

/// Payload for [`AGENT_OUTPUT_EVENT`].
#[derive(Clone, Serialize)]
pub struct AgentOutput {
    /// Base64-encoded raw PTY bytes (may split UTF-8 across chunks).
    pub base64: String,
}

/// Payload for [`AGENT_EXIT_EVENT`].
#[derive(Clone, Serialize)]
pub struct AgentExit {
    /// Human-readable reason (exit status or error).
    pub message: String,
}

/// A running agent PTY session.
struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
}

/// Managed Tauri state holding at most one live agent session.
#[derive(Default)]
pub struct AgentPty {
    session: Arc<Mutex<Option<Session>>>,
}

impl AgentPty {
    /// Spawns `ollama launch <runtime> --model <model>` in a PTY at `cwd`.
    ///
    /// Replaces any existing session. Output is streamed to `app` via
    /// [`AGENT_OUTPUT_EVENT`]; exit is signalled with [`AGENT_EXIT_EVENT`].
    #[allow(clippy::too_many_arguments)]
    pub fn launch<R: Runtime>(
        &self,
        app: AppHandle<R>,
        runtime: &str,
        model: &str,
        ollama_host: Option<&str>,
        cwd: Option<PathBuf>,
        direct: bool,
        rows: u16,
        cols: u16,
    ) -> NestResult<()> {
        self.stop();

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| NestError::io(format!("failed to open pty: {error}")))?;

        // Two connection modes:
        //  * Ollama:  `ollama launch <rt> --model <model>` routes the agent to the
        //             Ollama server (OLLAMA_HOST). Ollama injects the provider
        //             base-URL overrides so the CLI talks to Ollama, not the cloud.
        //  * Direct:  run the agent binary itself (`claude`, `codex`, …) with no
        //             base-URL override, so it uses its own signed-in account
        //             (Anthropic / OpenAI) and cloud models.
        let (mut command, describe) = if direct {
            (direct_command(runtime), format!("{} (account)", direct_binary(runtime)))
        } else {
            let mut command = CommandBuilder::new("ollama");
            command.arg("launch");
            command.arg(runtime);
            command.arg("--model");
            command.arg(model);
            (command, format!("ollama launch {runtime} --model {model}"))
        };

        if let Some(dir) = cwd.filter(|dir| dir.is_dir()) {
            configure_agent_mcp_env(&mut command, &dir, runtime);
            command.cwd(dir);
        }
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("PATH", augmented_path());
        if let Ok(home) = std::env::var("HOME") {
            command.env("HOME", home);
        }
        // Only in Ollama mode do we point the CLI at the inference server. In
        // direct mode we deliberately leave OLLAMA_HOST / ANTHROPIC_BASE_URL
        // unset so the agent uses its native account + endpoint.
        if !direct {
            if let Some(host) = ollama_host.map(str::trim).filter(|h| !h.is_empty()) {
                command.env("OLLAMA_HOST", normalize_ollama_host(host));
            }
        }

        tracing::info!(
            target: "kiwi",
            runtime,
            model,
            direct,
            ollama_host = ollama_host.unwrap_or("(local)"),
            "launching agent: {describe}"
        );

        let mut child = pair.slave.spawn_command(command).map_err(|error| {
            let message = if direct {
                format!(
                    "failed to launch `{}`: {error}. Is it installed and on PATH?",
                    direct_binary(runtime)
                )
            } else {
                format!(
                    "failed to launch `ollama launch {runtime} --model {model}`: {error}. \
                     Is Ollama v0.15+ installed and on PATH?"
                )
            };
            tracing::error!(target: "kiwi", %error, runtime, direct, "agent launch failed");
            NestError::io(message)
        })?;
        drop(pair.slave);

        let master = pair.master;
        let reader = master
            .try_clone_reader()
            .map_err(|error| NestError::io(format!("failed to clone pty reader: {error}")))?;
        let writer = master
            .take_writer()
            .map_err(|error| NestError::io(format!("failed to open pty writer: {error}")))?;

        *self.session.lock().expect("agent pty mutex") = Some(Session { master, writer });

        // Stream PTY output to the webview.
        let out_app = app.clone();
        thread::spawn(move || read_loop(reader, out_app));

        // Watch for process exit and clear the session.
        let exit_app = app.clone();
        let session_slot = Arc::clone(&self.session);
        thread::spawn(move || {
            let message = match child.wait() {
                Ok(status) => format!("agent exited ({status})"),
                Err(error) => format!("agent wait failed: {error}"),
            };
            *session_slot.lock().expect("agent pty mutex") = None;
            let _ = exit_app.emit(AGENT_EXIT_EVENT, AgentExit { message });
        });

        tracing::info!(target: "kiwi", runtime, model, rows, cols, "Agent PTY launched");
        Ok(())
    }

    /// Writes UTF-8 input (keystrokes) to the agent stdin.
    pub fn input(&self, data: &str) -> NestResult<()> {
        let mut guard = self.session.lock().expect("agent pty mutex");
        let session = guard
            .as_mut()
            .ok_or_else(|| NestError::validation("no agent session is running"))?;
        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|error| NestError::io(format!("failed to write to agent pty: {error}")))
    }

    /// Resizes the PTY grid to match the terminal viewport.
    pub fn resize(&self, rows: u16, cols: u16) -> NestResult<()> {
        let guard = self.session.lock().expect("agent pty mutex");
        if let Some(session) = guard.as_ref() {
            session
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|error| NestError::io(format!("failed to resize agent pty: {error}")))?;
        }
        Ok(())
    }

    /// Terminates the running session, if any.
    pub fn stop(&self) {
        *self.session.lock().expect("agent pty mutex") = None;
    }

    /// Returns whether a session is currently running.
    pub fn is_running(&self) -> bool {
        self.session.lock().expect("agent pty mutex").is_some()
    }
}

/// Builds a `PATH` that includes common CLI install dirs.
///
/// GUI apps launched outside a login shell often inherit a minimal `PATH`
/// (e.g. `/usr/bin`), so `ollama` / agent binaries in `/usr/local/bin`,
/// Homebrew, or `~/.local/bin` are missing. Append the usual locations.
/// Maps an `ollama launch` runtime id to its native CLI binary for direct mode.
fn direct_binary(runtime: &str) -> &str {
    match runtime {
        "claude" => "claude",
        "codex" | "codex-app" => "codex",
        // Best-effort passthrough for the remaining integrations.
        other => other,
    }
}

/// Builds the command that runs an agent CLI directly (native account mode).
fn direct_command(runtime: &str) -> CommandBuilder {
    CommandBuilder::new(direct_binary(runtime))
}

pub(crate) fn augmented_path() -> String {
    let mut parts: Vec<String> = std::env::var("PATH")
        .map(|path| std::env::split_paths(&path).map(|p| p.to_string_lossy().into_owned()).collect())
        .unwrap_or_default();

    let mut extras: Vec<String> = vec![
        "/usr/local/bin".into(),
        "/usr/bin".into(),
        "/bin".into(),
        "/opt/homebrew/bin".into(),
        "/snap/bin".into(),
    ];
    if let Ok(home) = std::env::var("HOME") {
        extras.push(format!("{home}/.local/bin"));
        extras.push(format!("{home}/bin"));
    }

    for extra in extras {
        if !parts.iter().any(|p| p == &extra) {
            parts.push(extra);
        }
    }
    parts.join(":")
}

/// Ensures the Ollama host has a scheme so agents that expect a URL work.
///
/// `192.168.88.10:11434` → `http://192.168.88.10:11434`; existing
/// `http(s)://…` values pass through unchanged.
fn normalize_ollama_host(host: &str) -> String {
    if host.starts_with("http://") || host.starts_with("https://") {
        host.to_string()
    } else {
        format!("http://{host}")
    }
}

/// Configures runtime-specific MCP environment for external agent CLIs.
fn configure_agent_mcp_env(command: &mut CommandBuilder, workspace: &Path, runtime: &str) {
    match runtime {
        "opencode" => configure_opencode_env(command, workspace),
        "claude" => configure_claude_env(command, workspace),
        _ => {}
    }
}

/// Configures OpenCode MCP env vars for a Kiwi workspace directory.
///
/// Nest MCP servers live at the monorepo root (`.venv`, `tools/mcp_*.py`). When
/// the Kiwi workspace is a nested folder such as `apps/kiwi`, walk up to that
/// root and set `NEST_PROJECT_ROOT` + `OPENCODE_CONFIG`.
fn configure_opencode_env(command: &mut CommandBuilder, workspace: &Path) {
    let Some(root) = resolve_nest_mcp_root(workspace) else {
        tracing::warn!(
            target: "kiwi",
            workspace = %workspace.display(),
            "opencode: no Nest MCP root found (missing opencode.json + .venv)"
        );
        return;
    };

    command.env(
        "NEST_PROJECT_ROOT",
        root.to_string_lossy().into_owned(),
    );

    if let Some(config) = resolve_opencode_config(workspace) {
        tracing::info!(
            target: "kiwi",
            nest_root = %root.display(),
            opencode_config = %config,
            "opencode MCP env configured"
        );
        command.env("OPENCODE_CONFIG", config);
    } else {
        tracing::warn!(
            target: "kiwi",
            nest_root = %root.display(),
            "opencode: NEST_PROJECT_ROOT set but opencode.json not found"
        );
    }
}

/// Configures Claude Code MCP for a Kiwi workspace directory.
///
/// Claude reads project MCP from `.mcp.json` at the project root and expands
/// `${NEST_PROJECT_ROOT}` in server commands. Kiwi sets `NEST_PROJECT_ROOT` to the
/// Nest monorepo root (`.venv` + `tools/mcp_*.py`).
fn configure_claude_env(command: &mut CommandBuilder, workspace: &Path) {
    let Some(root) = resolve_nest_mcp_root(workspace) else {
        tracing::warn!(
            target: "kiwi",
            workspace = %workspace.display(),
            "claude: no Nest MCP root found (missing opencode.json + .venv)"
        );
        return;
    };

    let root_str = root.to_string_lossy().into_owned();
    command.env("NEST_PROJECT_ROOT", &root_str);

    if let Err(error) = ensure_claude_project_mcp(workspace, &root) {
        tracing::warn!(
            target: "kiwi",
            workspace = %workspace.display(),
            %error,
            "claude: failed to sync .mcp.json into workspace"
        );
    }

    tracing::info!(
        target: "kiwi",
        nest_root = %root.display(),
        workspace = %workspace.display(),
        "claude MCP env configured (NEST_PROJECT_ROOT + .mcp.json)"
    );
}

/// Copies the repo `.mcp.json` into the workspace when missing.
fn ensure_claude_project_mcp(workspace: &Path, nest_root: &Path) -> Result<(), String> {
    let source = nest_root.join(".mcp.json");
    if !source.is_file() {
        return Err(format!(
            "missing Nest MCP config at {}",
            source.display()
        ));
    }

    let target = workspace.join(".mcp.json");
    if target.is_file() {
        return Ok(());
    }

    if workspace == nest_root {
        return Ok(());
    }

    std::fs::copy(&source, &target)
        .map(|_| ())
        .map_err(|error| {
        format!(
            "failed to copy {} to {}: {error}",
            source.display(),
            target.display()
        )
    })
}

/// Walks up from `workspace` to the Nest monorepo root that hosts MCP servers.
fn resolve_nest_mcp_root(workspace: &Path) -> Option<PathBuf> {
    let mut current = workspace.to_path_buf();
    loop {
        if current.join(".venv").is_dir()
            && (current.join("opencode.json").is_file()
                || current.join(".opencode/opencode.json").is_file()
                || current.join(".mcp.json").is_file()
                || current.join("tools/mcp_memory_server.py").is_file())
        {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Resolves the OpenCode config file for a workspace directory.
///
/// OpenCode loads `opencode.json` from the project root; `OPENCODE_CONFIG` makes
/// discovery explicit when Kiwi launches `ollama launch opencode` in a PTY.
fn resolve_opencode_config(workspace: &Path) -> Option<String> {
    let mut current = workspace.to_path_buf();
    loop {
        for candidate in [
            current.join("opencode.json"),
            current.join(".opencode/opencode.json"),
        ] {
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn read_loop<R: Runtime>(mut reader: Box<dyn Read + Send>, app: AppHandle<R>) {
    let engine = base64::engine::general_purpose::STANDARD;
    let mut buffer = [0_u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                let base64 = engine.encode(&buffer[..count]);
                if app.emit(AGENT_OUTPUT_EVENT, AgentOutput { base64 }).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolve_opencode_config_prefers_project_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("opencode.json");
        let nested = dir.path().join(".opencode/opencode.json");
        std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
        std::fs::write(&root, "{}").unwrap();
        std::fs::write(&nested, "{}").unwrap();

        assert_eq!(
            resolve_opencode_config(dir.path()),
            Some(root.to_string_lossy().into_owned())
        );
    }

    #[test]
    fn resolve_opencode_config_walks_up_to_monorepo_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("repo");
        let nested = root.join("apps/kiwi");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.join("opencode.json"), "{}").unwrap();

        assert_eq!(
            resolve_opencode_config(&nested),
            Some(root.join("opencode.json").to_string_lossy().into_owned())
        );
    }

    #[test]
    fn resolve_nest_mcp_root_walks_up_from_nested_workspace() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("repo");
        let nested = root.join("apps/kiwi");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir_all(root.join(".venv")).unwrap();
        std::fs::write(root.join("opencode.json"), "{}").unwrap();

        assert_eq!(resolve_nest_mcp_root(&nested), Some(root));
    }

    #[test]
    fn ensure_claude_project_mcp_copies_into_nested_workspace() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("repo");
        let nested = root.join("apps/kiwi");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.join(".mcp.json"), r#"{"mcpServers":{}}"#).unwrap();

        ensure_claude_project_mcp(&nested, &root).unwrap();
        assert!(nested.join(".mcp.json").is_file());
    }
}
