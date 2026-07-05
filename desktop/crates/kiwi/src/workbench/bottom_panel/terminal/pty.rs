//! PTY session backed by [`portable_pty`].

use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

use nest_error::{NestError, NestResult};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

/// Handle to a running shell inside a pseudo-terminal.
pub struct PtySession {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    output_rx: Receiver<Vec<u8>>,
}

impl PtySession {
    /// Spawns an interactive shell in `cwd` with the given terminal size.
    pub fn spawn(cwd: &Path, rows: u16, cols: u16) -> NestResult<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| {
                NestError::io(format!("failed to open pty: {error}")).with_module("kiwi")
            })?;

        let shell = default_shell();
        let mut command = CommandBuilder::new(&shell);
        command.cwd(cwd);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");

        if let Ok(path) = std::env::var("PATH") {
            command.env("PATH", path);
        }

        let _child = pair.slave.spawn_command(command).map_err(|error| {
            NestError::io(format!("failed to spawn shell {shell}: {error}")).with_module("kiwi")
        })?;

        drop(pair.slave);

        let master = pair.master;
        let reader = master.try_clone_reader().map_err(|error| {
            NestError::io(format!("failed to clone pty reader: {error}")).with_module("kiwi")
        })?;
        let writer = master.take_writer().map_err(|error| {
            NestError::io(format!("failed to open pty writer: {error}")).with_module("kiwi")
        })?;

        let (tx, output_rx) = mpsc::channel();
        thread::spawn(move || read_loop(reader, tx));

        Ok(Self {
            master: Arc::new(Mutex::new(master)),
            writer: Arc::new(Mutex::new(writer)),
            output_rx,
        })
    }

    /// Returns pending PTY output chunks, if any.
    pub fn drain_output(&self) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        while let Ok(chunk) = self.output_rx.try_recv() {
            chunks.push(chunk);
        }
        chunks
    }

    /// Writes bytes to the shell stdin.
    pub fn write(&self, data: &[u8]) -> NestResult<()> {
        let mut writer = self.writer.lock().expect("pty writer mutex poisoned");
        writer.write_all(data).map_err(|error| {
            NestError::io(format!("failed to write to pty: {error}")).with_module("kiwi")
        })
    }

    /// Resizes the PTY to match the UI grid.
    pub fn resize(&self, rows: u16, cols: u16) -> NestResult<()> {
        let master = self.master.lock().expect("pty master mutex poisoned");
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| {
                NestError::io(format!("failed to resize pty: {error}")).with_module("kiwi")
            })
    }
}

fn read_loop(mut reader: Box<dyn Read + Send>, tx: mpsc::Sender<Vec<u8>>) {
    let mut buffer = [0_u8; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                if tx.send(buffer[..count].to_vec()).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn default_shell() -> String {
    if cfg!(windows) {
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".into())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into())
    }
}
