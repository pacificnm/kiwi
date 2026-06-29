use std::io::{Read, Write};
use std::path::Path;
#[cfg(test)]
use std::thread;
#[cfg(test)]
use std::time::Duration;

use portable_pty::{native_pty_system, Child, CommandBuilder, ExitStatus, MasterPty, PtySize};

use crate::config::AgentSettings;
use crate::env::repo_env_vars;

use super::command::{agent_launch_spec, AgentLaunchSpec};
use super::error::AgentError;

pub struct AgentSession {
    master: Box<dyn MasterPty + Send>,
    writer: Option<Box<dyn Write + Send>>,
    child: Box<dyn Child + Send + Sync>,
    pub spec: AgentLaunchSpec,
    pub cols: u16,
    pub rows: u16,
}

impl AgentSession {
    pub fn spawn(
        repo_root: &Path,
        settings: &AgentSettings,
        cols: u16,
        rows: u16,
    ) -> Result<Self, AgentError> {
        let spec = agent_launch_spec(settings);
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|err| AgentError::spawn(err.to_string()))?;

        let mut command = CommandBuilder::new(&spec.command);
        command.cwd(repo_root);
        apply_pty_env(&mut command);
        apply_repo_env(&mut command, repo_root, settings);
        for arg in &spec.args {
            command.arg(arg);
        }
        for (key, value) in &spec.env {
            command.env(key, value);
        }

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|err| AgentError::spawn(err.to_string()))?;

        drop(pair.slave);

        let writer = pair
            .master
            .take_writer()
            .map_err(|err| AgentError::spawn(err.to_string()))?;

        Ok(Self {
            master: pair.master,
            writer: Some(writer),
            child,
            spec,
            cols,
            rows,
        })
    }

    #[must_use]
    pub fn pid(&self) -> Option<u32> {
        self.child.process_id()
    }

    pub fn try_clone_reader(&self) -> Result<Box<dyn Read + Send>, AgentError> {
        self.master
            .try_clone_reader()
            .map_err(|err| AgentError::spawn(err.to_string()))
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), AgentError> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| AgentError::write("agent is closed"))?;
        writer
            .write_all(data)
            .map_err(|err| AgentError::write(err.to_string()))?;
        writer
            .flush()
            .map_err(|err| AgentError::write(err.to_string()))
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AgentError> {
        if cols == self.cols && rows == self.rows {
            return Ok(());
        }

        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|err| AgentError::spawn(err.to_string()))?;

        self.cols = cols;
        self.rows = rows;
        Ok(())
    }

    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }

    #[must_use]
    pub fn poll_exit(&mut self) -> Option<i32> {
        match self.child.try_wait() {
            Ok(None) => None,
            Ok(Some(status)) => Some(exit_status_code(&status)),
            Err(_) => Some(1),
        }
    }

    pub fn shutdown(&mut self) {
        self.writer.take();
        let _ = self.child.kill();
    }

    #[cfg(test)]
    fn shutdown_and_reap(&mut self) {
        self.shutdown();
        self.reap_child();
    }

    #[cfg(test)]
    fn reap_child(&mut self) {
        for _ in 0..100 {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(_) => return,
            }
        }
        let _ = self.child.kill();
        let _ = self.child.try_wait();
    }
}

impl Drop for AgentSession {
    fn drop(&mut self) {
        if self.writer.is_some() {
            self.shutdown();
        }
    }
}

fn apply_pty_env(command: &mut CommandBuilder) {
    let term = std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
    command.env("TERM", term);
}

/// Forward repo `.env` MCP credentials to the agent without overriding explicit config.
fn apply_repo_env(command: &mut CommandBuilder, repo_root: &Path, settings: &AgentSettings) {
    command.env("KIWI_REPO_ROOT", repo_root.as_os_str());

    for (key, value) in repo_env_vars(repo_root) {
        if settings.env.contains_key(&key) {
            continue;
        }
        command.env(key, value);
    }
}

fn exit_status_code(status: &ExitStatus) -> i32 {
    i32::try_from(status.exit_code()).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::config::AgentSettings;

    use super::*;

    #[test]
    fn spawns_agent_in_repo_root() {
        if !Path::new("/bin/bash").exists() && !Path::new("/usr/bin/bash").exists() {
            return;
        }

        let repo = std::env::temp_dir().join("kiwi-agent-spawn-test");
        std::fs::create_dir_all(&repo).expect("create temp repo");

        let settings = AgentSettings {
            command: "bash".to_string(),
            args: Vec::new(),
            env: Default::default(),
            ..AgentSettings::default()
        };
        let mut session = AgentSession::spawn(&repo, &settings, 80, 24).expect("spawn agent");
        assert!(session.pid().is_some());
        assert!(session.is_running());
        session.shutdown_and_reap();
        assert!(!session.is_running());
    }
}
