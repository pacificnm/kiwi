use std::fmt;
use std::io::{self, stdout, Write};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, terminal};

use crate::config::{MouseMode, MouseSettings};

const MIN_COLUMNS: u16 = 80;
const MIN_ROWS: u16 = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalError {
    pub message: String,
}

impl TerminalError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for TerminalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TerminalError {}

pub struct TerminalGuard {
    active: bool,
    suspended: bool,
    mouse_enabled: bool,
}

impl TerminalGuard {
    pub fn init(mouse: &MouseSettings) -> Result<Self, TerminalError> {
        warn_if_terminal_too_small()?;

        enable_raw_mode()
            .map_err(|err| TerminalError::new(format!("failed to enable raw mode: {err}")))?;

        let mut out = stdout();
        let mouse_enabled = should_enable_mouse(mouse);

        if let Err(err) = (|| {
            execute!(out, EnterAlternateScreen, Hide)?;
            execute!(out, EnableBracketedPaste)?;
            if mouse_enabled {
                execute!(out, EnableMouseCapture)?;
            }
            Ok::<(), io::Error>(())
        })() {
            let _ = disable_raw_mode();
            return Err(TerminalError::new(format!(
                "failed to initialize terminal: {err}"
            )));
        }

        Ok(Self {
            active: true,
            suspended: false,
            mouse_enabled,
        })
    }

    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn inactive() -> Self {
        Self {
            active: false,
            suspended: false,
            mouse_enabled: false,
        }
    }

    /// Release the terminal to a foreground TUI editor (nano, vim, etc.).
    pub fn suspend(&mut self) -> Result<(), TerminalError> {
        if !self.active || self.suspended {
            return Ok(());
        }

        let mut out = stdout();
        if let Err(err) = (|| {
            if self.mouse_enabled {
                execute!(out, DisableMouseCapture)?;
            }
            execute!(out, DisableBracketedPaste)?;
            execute!(out, LeaveAlternateScreen, Show)?;
            disable_raw_mode()?;
            out.flush()?;
            Ok::<(), io::Error>(())
        })() {
            return Err(TerminalError::new(format!(
                "failed to suspend terminal: {err}"
            )));
        }

        self.suspended = true;
        Ok(())
    }

    /// Restore Kiwi TUI control after a foreground editor exits.
    pub fn resume(&mut self) -> Result<(), TerminalError> {
        if !self.active || !self.suspended {
            return Ok(());
        }

        let mut out = stdout();
        if let Err(err) = (|| {
            enable_raw_mode()?;
            execute!(out, EnterAlternateScreen, Hide)?;
            execute!(out, EnableBracketedPaste)?;
            if self.mouse_enabled {
                execute!(out, EnableMouseCapture)?;
            }
            out.flush()?;
            Ok::<(), io::Error>(())
        })() {
            return Err(TerminalError::new(format!(
                "failed to resume terminal: {err}"
            )));
        }

        self.suspended = false;
        Ok(())
    }

    pub fn restore(&mut self) -> Result<(), TerminalError> {
        if !self.active {
            return Ok(());
        }

        if self.suspended {
            // Final shutdown while suspended: leave the user's shell visible.
            self.suspended = false;
            return Ok(());
        }

        let mut out = stdout();
        if let Err(err) = (|| {
            if self.mouse_enabled {
                execute!(out, DisableMouseCapture)?;
            }
            execute!(out, DisableBracketedPaste)?;
            execute!(out, LeaveAlternateScreen, Show)?;
            disable_raw_mode()?;
            out.flush()?;
            Ok::<(), io::Error>(())
        })() {
            return Err(TerminalError::new(format!(
                "failed to restore terminal: {err}"
            )));
        }

        self.active = false;
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn should_enable_mouse(mouse: &MouseSettings) -> bool {
    mouse.enabled && mouse.mode == MouseMode::Hybrid
}

fn warn_if_terminal_too_small() -> Result<(), TerminalError> {
    let (columns, rows) = terminal::size()
        .map_err(|err| TerminalError::new(format!("failed to read terminal size: {err}")))?;

    if columns < MIN_COLUMNS || rows < MIN_ROWS {
        eprintln!(
            "warning: terminal size {columns}x{rows} is below recommended {MIN_COLUMNS}x{MIN_ROWS}"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::{MouseMode, MouseSettings};

    use super::should_enable_mouse;

    #[test]
    fn mouse_capture_requires_enabled_hybrid_mode() {
        let enabled = MouseSettings {
            enabled: true,
            mode: MouseMode::Hybrid,
        };
        let disabled = MouseSettings {
            enabled: false,
            mode: MouseMode::Hybrid,
        };
        let mode_disabled = MouseSettings {
            enabled: true,
            mode: MouseMode::Disabled,
        };

        assert!(should_enable_mouse(&enabled));
        assert!(!should_enable_mouse(&disabled));
        assert!(!should_enable_mouse(&mode_disabled));
    }

    #[test]
    fn restore_is_idempotent_for_inactive_guard() {
        let mut guard = super::TerminalGuard::inactive();
        guard.restore().expect("inactive guard restores cleanly");
    }
}
