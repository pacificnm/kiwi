use crate::bootstrap::StartupContext;

pub fn cleanup(context: &mut StartupContext) {
    let _ = context.terminal.restore();
    // Workspace persistence and service teardown will be added in later milestones.
}
