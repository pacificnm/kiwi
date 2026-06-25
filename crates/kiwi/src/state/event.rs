use std::path::PathBuf;

use crate::git::GitFileEntry;
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
        file_entries: Vec<GitFileEntry>,
    },
    #[cfg_attr(not(test), allow(dead_code))]
    Quit,
    ShellOutput(Vec<u8>),
    #[cfg_attr(not(test), allow(dead_code))]
    ShellExited(i32),
    AgentOutput(Vec<u8>),
    #[cfg_attr(not(test), allow(dead_code))]
    AgentExited(i32),
    FileTreeChildrenLoaded {
        parent: PathBuf,
        children: Vec<crate::file_tree::DirectoryEntry>,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    Navigation(NavCommand),
    Quit,
    #[cfg_attr(not(test), allow(dead_code))]
    RequestGitRefresh,
    ShellWrite(Vec<u8>),
    ShellScroll(i32),
    AgentWrite(Vec<u8>),
    AgentScroll(i32),
    AgentRestart,
    PaletteOpen,
    PaletteClose,
    PaletteAppendChar(char),
    PaletteBackspace,
    PaletteMoveSelection(i32),
    PaletteHistoryUp,
    PaletteHistoryDown,
    PaletteExecuteSelected,
    PaletteExecuteMatch(usize),
    #[cfg_attr(not(test), allow(dead_code))]
    FileTreeExpand(PathBuf),
    #[cfg_attr(not(test), allow(dead_code))]
    FileTreeCollapse(PathBuf),
    #[cfg_attr(not(test), allow(dead_code))]
    FileTreeSelect(PathBuf),
    #[cfg_attr(not(test), allow(dead_code))]
    FileTreeRefresh,
    FileTreeMoveSelection(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideEffect {
    Quit,
    SpawnGitRefresh,
    SpawnGitHubRefresh,
    SpawnAgent,
    RestartAgent,
    WriteShell(Vec<u8>),
    WriteAgent(Vec<u8>),
    ResizeShell {
        cols: u16,
        rows: u16,
    },
    #[cfg_attr(not(test), allow(dead_code))]
    SaveWorkspace,
    SavePaletteHistory,
    LoadDirectoryChildren(PathBuf),
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
