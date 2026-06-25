use std::io::{Read, Write};
use std::path::Path;
#[cfg(test)]
use std::thread;
#[cfg(test)]
use std::time::Duration;

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

use crate::config::ShellSettings;

use super::command::{shell_launch_spec, ShellLaunchSpec};
use super::error::ShellError;

pub struct ShellSession {
    master: Box<dyn MasterPty + Send>,
    writer: Option<Box<dyn Write + Send>>,
    child: Box<dyn Child + Send + Sync>,
    pub spec: ShellLaunchSpec,
    pub cols: u16,
    pub rows: u16,
}

impl ShellSession {
    pub fn spawn(
        repo_root: &Path,
        settings: &ShellSettings,
        cols: u16,
        rows: u16,
    ) -> Result<Self, ShellError> {
        let spec = shell_launch_spec(settings);
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|err| ShellError::spawn(err.to_string()))?;

        let mut command = CommandBuilder::new(&spec.command);
        command.cwd(repo_root);
        apply_pty_env(&mut command);
        for arg in &spec.args {
            command.arg(arg);
        }
        apply_interactive_shell_args(&mut command, &spec);

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|err| ShellError::spawn(err.to_string()))?;

        drop(pair.slave);

        let writer = pair
            .master
            .take_writer()
            .map_err(|err| ShellError::spawn(err.to_string()))?;

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

    pub fn try_clone_reader(&self) -> Result<Box<dyn Read + Send>, ShellError> {
        self.master
            .try_clone_reader()
            .map_err(|err| ShellError::spawn(err.to_string()))
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), ShellError> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| ShellError::write("shell is closed"))?;
        writer
            .write_all(data)
            .map_err(|err| ShellError::write(err.to_string()))?;
        writer
            .flush()
            .map_err(|err| ShellError::write(err.to_string()))
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), ShellError> {
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
            .map_err(|err| ShellError::resize(err.to_string()))?;

        self.cols = cols;
        self.rows = rows;
        Ok(())
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

    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }
}

impl Drop for ShellSession {
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

fn apply_interactive_shell_args(command: &mut CommandBuilder, spec: &ShellLaunchSpec) {
    let shell_name = Path::new(&spec.command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&spec.command);

    if shell_name != "bash" {
        return;
    }

    let has_interactive = spec
        .args
        .iter()
        .any(|arg| arg == "-i" || arg.starts_with("-i") || arg == "+i" || arg.starts_with("+i"));
    if !has_interactive {
        command.arg("-i");
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::config::ShellSettings;

    use super::*;

    #[test]
    fn spawns_interactive_shell_in_repo_root() {
        if !Path::new("/bin/bash").exists() && !Path::new("/usr/bin/bash").exists() {
            return;
        }

        let repo = std::env::temp_dir().join("kiwi-shell-spawn-test");
        std::fs::create_dir_all(&repo).expect("create temp repo");

        let settings = ShellSettings {
            command: "bash".to_string(),
            args: Vec::new(),
        };
        let mut session = ShellSession::spawn(&repo, &settings, 80, 24).expect("spawn shell");
        assert!(session.pid().is_some());
        assert!(session.is_running());
        session.shutdown_and_reap();
        assert!(!session.is_running());
    }

    #[test]
    fn resize_updates_pty_dimensions() {
        if !Path::new("/bin/bash").exists() && !Path::new("/usr/bin/bash").exists() {
            return;
        }

        let repo = std::env::temp_dir().join("kiwi-shell-resize-test");
        std::fs::create_dir_all(&repo).expect("create temp repo");

        let settings = ShellSettings {
            command: "bash".to_string(),
            args: Vec::new(),
        };
        let mut session = ShellSession::spawn(&repo, &settings, 80, 24).expect("spawn shell");
        session.resize(100, 30).expect("resize pty");
        assert_eq!(session.cols, 100);
        assert_eq!(session.rows, 30);
        session.shutdown_and_reap();
    }
}
