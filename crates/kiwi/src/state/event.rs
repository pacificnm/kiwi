use std::path::PathBuf;

use crate::navigation::NavCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Command(AppCommand),
    TerminalResize {
        width: u16,
        height: u16,
    },
    GitRefreshRequested,
    #[cfg_attr(not(test), allow(dead_code))]
    GitStatusUpdated {
        modified_files: Vec<String>,
    },
    #[cfg_attr(not(test), allow(dead_code))]
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCommand {
    Navigation(NavCommand),
    Quit,
    #[cfg_attr(not(test), allow(dead_code))]
    RequestGitRefresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideEffect {
    Quit,
    SpawnGitRefresh,
    #[cfg_attr(not(test), allow(dead_code))]
    SaveWorkspace,
    #[cfg_attr(not(test), allow(dead_code))]
    LaunchEditor(PathBuf),
}

impl AppCommand {
    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn from_nav(command: NavCommand) -> Self {
        Self::Navigation(command)
    }
}

impl From<NavCommand> for AppCommand {
    fn from(command: NavCommand) -> Self {
        Self::Navigation(command)
    }
}

#[cfg(test)]
mod tests {
    use crate::layout::FocusTarget;
    use crate::navigation::{LeftNavTab, MainTab};

    use super::*;

    #[test]
    fn navigation_command_wraps_nav_command() {
        let cmd = AppCommand::from_nav(NavCommand::SelectLeftTab(LeftNavTab::Git));
        assert!(matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Git))
        ));
    }

    #[test]
    fn focus_target_available_for_commands() {
        let cmd = AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Shell));
        assert!(matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SetFocus(FocusTarget::Shell))
        ));
    }

    #[test]
    fn main_tab_command_converts() {
        let cmd: AppCommand = NavCommand::SelectMainTab(MainTab::Issues).into();
        assert!(matches!(
            cmd,
            AppCommand::Navigation(NavCommand::SelectMainTab(MainTab::Issues))
        ));
    }
}
