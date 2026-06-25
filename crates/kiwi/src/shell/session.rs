use std::io::Read;
use std::path::Path;

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

use crate::config::ShellSettings;

use super::command::{shell_launch_spec, ShellLaunchSpec};
use super::error::ShellError;

pub struct ShellSession {
    master: Box<dyn MasterPty + Send>,
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
        for arg in &spec.args {
            command.arg(arg);
        }

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|err| ShellError::spawn(err.to_string()))?;

        drop(pair.slave);

        Ok(Self {
            master: pair.master,
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
        let _ = self.child.kill();
        let _ = self.child.wait();
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
    }
}
