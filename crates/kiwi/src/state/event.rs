use std::path::PathBuf;

use crate::diff::{DiffSource, FileDiffLoadResult};
use crate::git::GitFileEntry;
use crate::navigation::NavCommand;
use crate::preview::PreviewLoadResult;
use crate::search::{SearchMode, SearchResult};

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
        branch: Option<String>,
        ahead: u32,
        behind: u32,
        file_entries: Vec<GitFileEntry>,
        error: Option<String>,
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
    PreviewLoaded {
        path: PathBuf,
        result: PreviewLoadResult,
    },
    SearchCompleted {
        generation: u64,
        results: Vec<SearchResult>,
        truncated: bool,
        error: Option<String>,
    },
    EditorLaunched {
        path: PathBuf,
        command: String,
    },
    EditorLaunchFailed {
        path: PathBuf,
        error: String,
        show_modal: bool,
    },
    FsChanged {
        paths: Vec<PathBuf>,
    },
    DiffLoaded {
        result: FileDiffLoadResult,
    },
    GitHubAuthChecked {
        result: crate::github::GitHubAuthCheckResult,
    },
    GitHubIssuesLoaded {
        result: crate::github::IssueListLoadResult,
    },
    GitHubIssueDetailLoaded {
        number: u32,
        result: crate::github::IssueDetailLoadResult,
    },
    GitHubIssueCommentCompleted {
        number: u32,
        result: crate::github::IssueActionResult,
    },
    GitHubRepoLabelsLoaded {
        result: crate::github::RepoLabelsLoadResult,
    },
    GitHubIssueLabelsApplied {
        number: u32,
        result: crate::github::IssueActionResult,
    },
    GitHubOpenBrowserCompleted {
        target: crate::github::GitHubBrowserTarget,
        result: crate::github::IssueActionResult,
    },
    GitHubPrCreateCompleted {
        result: crate::github::IssueActionResult,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    Navigation(NavCommand),
    Quit,
    #[cfg_attr(not(test), allow(dead_code))]
    RequestGitRefresh,
    GitHubRefresh,
    GitHubMoveIssueSelection(i32),
    GitHubSelectIssue(usize),
    GitHubOpenSelected,
    GitHubSelectLeftPane(crate::github::GitHubLeftPane),
    GitHubIssueDetailScroll(i32),
    GitHubIssueDetailPageScroll(i32),
    GitHubLabelPickerMove(i32),
    GitHubLabelPickerToggle,
    GitHubLabelPickerApply,
    GitHubLabelPickerCancel,
    GitHubOpenInBrowser,
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
    GitMoveSelection(i32),
    GitSelect(usize),
    GitOpenSelected,
    GitRefresh,
    PreviewFile {
        path: PathBuf,
        line: Option<u32>,
    },
    PreviewScroll(i32),
    PreviewPageScroll(i32),
    DiffScroll(i32),
    DiffPageScroll(i32),
    DiffHorizontalScroll(i32),
    DiffToggleSource,
    #[cfg_attr(not(test), allow(dead_code))]
    DiffSetSource(crate::diff::DiffSource),
    DiffNextFile,
    DiffPrevFile,
    #[cfg_attr(not(test), allow(dead_code))]
    DiffSelectFile(String),
    #[cfg_attr(not(test), allow(dead_code))]
    SearchSetQuery(String),
    SearchAppendChar(char),
    SearchBackspace,
    SearchClear,
    SearchSetMode(SearchMode),
    SearchExecute,
    #[cfg_attr(not(test), allow(dead_code))]
    SearchCancel,
    SearchMoveSelection(i32),
    SearchSelect(usize),
    OpenEditor {
        path: PathBuf,
        line: Option<u32>,
    },
    ModalDismiss,
    ClipboardCopy,
    ClipboardCut,
    ClipboardPaste,
    PasteText(String),
    SelectionBegin {
        pane: crate::selection::SelectionPane,
        line: usize,
        col: usize,
    },
    SelectionExtend {
        line: usize,
        col: usize,
    },
    SelectionEnd,
    SelectionClear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideEffect {
    Quit,
    SpawnGitRefresh,
    #[cfg_attr(not(test), allow(dead_code))]
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
    LoadPreviewFile(PathBuf),
    LoadFileDiff {
        path: String,
        source: DiffSource,
    },
    CancelSearch,
    RunSearch {
        mode: SearchMode,
        query: String,
        generation: u64,
    },
    #[cfg_attr(not(test), allow(dead_code))]
    LaunchEditor {
        path: PathBuf,
        line: Option<u32>,
    },
    CopyToClipboard(String),
    PasteFromClipboard,
    SpawnGitHubAuthCheck,
    SpawnGitHubIssueList,
    SpawnGitHubIssueDetail {
        number: u32,
    },
    SpawnGitHubIssueComment {
        number: u32,
        body: String,
    },
    SpawnGitHubRepoLabels,
    SpawnGitHubIssueLabelApply {
        number: u32,
        labels: Vec<String>,
    },
    SpawnGitHubOpenBrowser {
        target: crate::github::GitHubBrowserTarget,
    },
    SpawnGitHubPrCreate {
        request: crate::github::PrCreateRequest,
    },
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
