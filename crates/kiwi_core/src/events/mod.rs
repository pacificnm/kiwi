use std::path::PathBuf;

use crate::agent::AgentId;
use crate::diff::{DiffSource, FileDiffLoadResult};
use crate::file_tree::DirectoryEntry;
use crate::git::{BranchEntry, GitFileEntry};
use crate::github::{
    GhContextMenuAction, GhContextTarget, GitHubAuthCheckResult, GitHubBrowserTarget, GitHubLeftPane,
    IssueActionResult, IssueDetailLoadResult, IssueListLoadResult, PrCreateRequest, PrDetailLoadResult,
    PrListLoadResult, RepoLabelsLoadResult,
};
use crate::navigation::NavCommand;
use crate::preview::PreviewLoadResult;
use crate::search::{SearchMode, SearchResult};
use crate::selection::SelectionPane;

mod channel;

pub use channel::{EventChannel, EventSender, EVENT_CHANNEL_CAPACITY};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Command(AppCommand),
    GitRefreshRequested,
    #[cfg_attr(not(test), allow(dead_code))]
    GitStatusUpdated {
        branch: Option<String>,
        remote_repo: Option<String>,
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
    AgentOutput {
        agent_id: AgentId,
        data: Vec<u8>,
    },
    #[cfg_attr(not(test), allow(dead_code))]
    AgentExited {
        agent_id: AgentId,
        code: i32,
    },
    FileTreeChildrenLoaded {
        parent: PathBuf,
        children: Vec<DirectoryEntry>,
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
        result: GitHubAuthCheckResult,
    },
    GitHubIssuesLoaded {
        result: IssueListLoadResult,
    },
    GitHubPrsLoaded {
        result: PrListLoadResult,
    },
    GitHubIssueDetailLoaded {
        number: u32,
        result: IssueDetailLoadResult,
    },
    GitHubPrDetailLoaded {
        number: u32,
        result: PrDetailLoadResult,
    },
    GitHubIssueCommentCompleted {
        number: u32,
        result: IssueActionResult,
    },
    GitHubIssueCreateBranchCompleted {
        number: u32,
        result: IssueActionResult,
    },
    GitHubRepoLabelsLoaded {
        result: RepoLabelsLoadResult,
    },
    GitHubIssueLabelsApplied {
        number: u32,
        result: IssueActionResult,
    },
    GitHubOpenBrowserCompleted {
        target: GitHubBrowserTarget,
        result: IssueActionResult,
    },
    GitHubPrCreateCompleted {
        result: IssueActionResult,
    },
    GitHubPrMergeCompleted {
        number: u32,
        result: IssueActionResult,
    },
    BranchListLoaded {
        entries: Vec<BranchEntry>,
        error: Option<String>,
    },
    BranchCheckoutCompleted {
        branch_name: String,
        error: Option<String>,
    },
    /// Progress update while installing or reinstalling a plugin in the GUI.
    PluginInstallProgress {
        message: String,
        step: u32,
        total: u32,
    },
    /// Plugin install/reinstall finished (success or failure).
    PluginInstallFinished {
        result: Option<crate::plugins::PluginInstallResult>,
        error: Option<String>,
    },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    Navigation(NavCommand),
    Quit,
    #[cfg_attr(not(test), allow(dead_code))]
    RequestGitRefresh,
    GitHubRefresh,
    GitHubMoveIssueSelection(i32),
    GitHubMovePrSelection(i32),
    GitHubSelectIssue(usize),
    GitHubSelectPr(usize),
    GitHubOpenSelected,
    GitHubSelectLeftPane(GitHubLeftPane),
    GitHubIssueDetailScroll(i32),
    GitHubIssueDetailPageScroll(i32),
    GitHubPrDetailScroll(i32),
    GitHubPrDetailPageScroll(i32),
    GitHubLabelPickerMove(i32),
    GitHubLabelPickerToggle,
    GitHubLabelPickerApply,
    GitHubLabelPickerCancel,
    GitHubContextMenuOpen {
        anchor_x: u16,
        anchor_y: u16,
        target: GhContextTarget,
    },
    GitHubContextMenuMove(i32),
    GitHubContextMenuExecute,
    GitHubContextMenuSelect(usize),
    GitHubContextMenuCancel,
    GitHubListAction {
        target: GhContextTarget,
        action: GhContextMenuAction,
    },
    GitHubOpenInBrowser,
    ShellWrite(Vec<u8>),
    ShellScroll(i32),
    ShellScrollLines(i32),
    AgentWrite(Vec<u8>),
    AgentScroll(i32),
    AgentScrollLines(i32),
    AgentRestart,
    AgentNew,
    AgentCycleNext,
    AgentCyclePrev,
    AgentSetActive(AgentId),
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
    BranchMoveSelection(i32),
    BranchSelect(usize),
    BranchCheckoutSelected,
    BranchRefresh,
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
    DiffSetSource(DiffSource),
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
        pane: SelectionPane,
        line: usize,
        col: usize,
    },
    SelectionExtend {
        line: usize,
        col: usize,
    },
    SelectionEnd,
    SelectionClear,
    SettingsMoveSelection(i32),
    SettingsSelect(usize),
    SettingsApplyTheme,
    #[cfg_attr(not(test), allow(dead_code))]
    SetTheme(String),
    /// Enable or disable a plugin. Updates state immediately; persisted via SideEffect.
    PluginSetEnabled { name: String, enabled: bool },
    /// Install a plugin from a local directory into the plugins folder.
    PluginInstall { src_path: std::path::PathBuf },
    /// Remove an installed plugin (files and registry entry).
    PluginRemove { name: String },
    /// Remove and reinstall a plugin from a local directory.
    PluginReinstall { src_path: std::path::PathBuf },
    /// Switch the active AI agent command. Updates config immediately; persisted via SideEffect.
    SetAgent { command: String, args: Vec<String> },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitEffect {
    SpawnRefresh,
    SpawnBranchList,
    SpawnBranchCheckout { name: String },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitHubEffect {
    #[cfg_attr(not(test), allow(dead_code))]
    SpawnRefresh,
    SpawnAuthCheck,
    SpawnIssueList,
    SpawnPrList,
    SpawnIssueDetail { number: u32 },
    SpawnPrDetail { number: u32 },
    SpawnIssueComment { number: u32, body: String },
    SpawnIssueCreateBranch { number: u32 },
    SpawnRepoLabels,
    SpawnIssueLabelApply { number: u32, labels: Vec<String> },
    SpawnOpenBrowser { target: GitHubBrowserTarget },
    SpawnPrCreate { request: PrCreateRequest },
    SpawnPrMerge { number: u32 },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellEffect {
    Write(Vec<u8>),
    Resize { cols: u16, rows: u16 },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEffect {
    Spawn(AgentId),
    Restart(AgentId),
    Write(Vec<u8>),
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsEffect {
    LoadDirectoryChildren(PathBuf),
    LoadPreviewFile(PathBuf),
    LoadFileDiff { path: String, source: DiffSource },
    #[cfg_attr(not(test), allow(dead_code))]
    LaunchEditor { path: PathBuf, line: Option<u32> },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchEffect {
    Cancel,
    Run { mode: SearchMode, query: String, generation: u64 },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideEffect {
    Quit,
    SaveWorkspace,
    PersistUserTheme { name: String },
    CopyToClipboard(String),
    PasteFromClipboard,
    Git(GitEffect),
    GitHub(GitHubEffect),
    Shell(ShellEffect),
    Agent(AgentEffect),
    Fs(FsEffect),
    Search(SearchEffect),
    /// Persist enable/disable change to plugin-registry.toml.
    PluginSetEnabled { name: String, enabled: bool },
    /// Copy a plugin directory into the plugins folder and register it.
    PluginInstall { src_path: std::path::PathBuf },
    /// Remove an installed plugin from disk and the registry.
    PluginRemove { name: String },
    /// Remove and reinstall a plugin from a local directory.
    PluginReinstall { src_path: std::path::PathBuf },
    /// Register a plugin after a successful background install.
    PluginInstallRegister { result: crate::plugins::PluginInstallResult },
    /// Refresh plugin list after a failed background install.
    PluginInstallFailed,
    /// Persist the chosen agent command to ~/.config/kiwi/config.toml.
    PersistAgentConfig { command: String, args: Vec<String> },
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
    use crate::navigation::{FocusTarget, LeftNavTab, MainTab, NavCommand};

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
