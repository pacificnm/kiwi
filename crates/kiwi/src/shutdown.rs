use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use signal_hook::consts::signal::{SIGINT, SIGTERM};

static SHUTDOWN_REQUESTED: OnceLock<Arc<AtomicBool>> = OnceLock::new();

fn shutdown_flag() -> Arc<AtomicBool> {
    SHUTDOWN_REQUESTED
        .get_or_init(|| Arc::new(AtomicBool::new(false)))
        .clone()
}

pub fn install_signal_handlers() {
    let flag = shutdown_flag();
    let _ = signal_hook::flag::register(SIGINT, Arc::clone(&flag));
    let _ = signal_hook::flag::register(SIGTERM, Arc::clone(&flag));
}

#[must_use]
pub fn shutdown_requested() -> bool {
    shutdown_flag().load(Ordering::Relaxed)
}

#[cfg(test)]
pub fn request_shutdown_for_test() {
    shutdown_flag().store(true, Ordering::Relaxed);
}

#[cfg(test)]
pub fn clear_shutdown_request_for_test() {
    shutdown_flag().store(false, Ordering::Relaxed);
}

use crate::bootstrap::StartupContext;
use crate::terminal::TerminalGuard;

#[cfg_attr(not(test), allow(dead_code))]
pub fn cleanup(context: &mut StartupContext) {
    cleanup_terminal(&mut context.terminal);
}

pub fn cleanup_terminal(terminal: &mut TerminalGuard) {
    let _ = terminal.restore();
    // Workspace persistence and service teardown will be added in later milestones.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_flag_starts_false() {
        clear_shutdown_request_for_test();
        assert!(!shutdown_requested());
    }

    #[test]
    fn shutdown_flag_can_be_set_for_test_helpers() {
        clear_shutdown_request_for_test();
        request_shutdown_for_test();
        assert!(shutdown_requested());
        clear_shutdown_request_for_test();
    }
}
