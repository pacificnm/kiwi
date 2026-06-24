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
