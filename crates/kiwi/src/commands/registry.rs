use crate::layout::FocusTarget;
use crate::navigation::{LeftNavTab, MainTab, NavCommand};

use super::{CommandContext, CommandDef, PaletteAction};

pub const COMMANDS: &[CommandDef] = &[
    CommandDef {
        id: "clipboard.copy",
        title: "Clipboard: Copy",
        shortcut: Some("Ctrl+C"),
        context: CommandContext::Always,
        action: PaletteAction::ClipboardCopy,
    },
    CommandDef {
        id: "clipboard.cut",
        title: "Clipboard: Cut",
        shortcut: Some("Ctrl+X"),
        context: CommandContext::Always,
        action: PaletteAction::ClipboardCut,
    },
    CommandDef {
        id: "clipboard.paste",
        title: "Clipboard: Paste",
        shortcut: Some("Ctrl+V"),
        context: CommandContext::Always,
        action: PaletteAction::ClipboardPaste,
    },
    CommandDef {
        id: "quit",
        title: "Quit Kiwi",
        shortcut: Some("q"),
        context: CommandContext::Always,
        action: PaletteAction::Quit,
    },
    CommandDef {
        id: "git.refresh",
        title: "Git: Refresh Status",
        shortcut: Some("R"),
        context: CommandContext::RequiresGitRepo,
        action: PaletteAction::RequestGitRefresh,
    },
    CommandDef {
        id: "diff.toggle_source",
        title: "Diff: Toggle Staged/Unstaged View",
        shortcut: Some("s"),
        context: CommandContext::DiffTab,
        action: PaletteAction::DiffToggleSource,
    },
    CommandDef {
        id: "diff.next_file",
        title: "Diff: Next Changed File",
        shortcut: Some("n"),
        context: CommandContext::DiffTab,
        action: PaletteAction::DiffNextFile,
    },
    CommandDef {
        id: "diff.prev_file",
        title: "Diff: Previous Changed File",
        shortcut: Some("p"),
        context: CommandContext::DiffTab,
        action: PaletteAction::DiffPrevFile,
    },
    CommandDef {
        id: "github.refresh",
        title: "GitHub: Refresh Issues and PRs",
        shortcut: Some("R"),
        context: CommandContext::RequiresGitRepo,
        action: PaletteAction::RequestGitHubRefresh,
    },
    CommandDef {
        id: "github.issue.comment",
        title: "GitHub: Comment on Issue",
        shortcut: None,
        context: CommandContext::RequiresGitRepo,
        action: PaletteAction::GitHubIssueCommentPrompt,
    },
    CommandDef {
        id: "github.issue.label",
        title: "GitHub: Add Labels to Issue",
        shortcut: None,
        context: CommandContext::RequiresGitRepo,
        action: PaletteAction::GitHubIssueLabelPicker,
    },
    CommandDef {
        id: "github.open.browser",
        title: "GitHub: Open in Browser",
        shortcut: Some("o"),
        context: CommandContext::RequiresGitRepo,
        action: PaletteAction::GitHubOpenInBrowser,
    },
    CommandDef {
        id: "agent.restart",
        title: "Agent: Restart",
        shortcut: Some("Ctrl+Shift+R"),
        context: CommandContext::AgentTab,
        action: PaletteAction::AgentRestart,
    },
    CommandDef {
        id: "editor.open",
        title: "Open in External Editor",
        shortcut: Some("e"),
        context: CommandContext::HasEditorTarget,
        action: PaletteAction::LaunchEditor,
    },
    CommandDef {
        id: "focus.left",
        title: "Focus: Left Panel",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SetFocus(FocusTarget::Left)),
    },
    CommandDef {
        id: "focus.main",
        title: "Focus: Main Panel",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SetFocus(FocusTarget::Main)),
    },
    CommandDef {
        id: "focus.shell",
        title: "Focus: Shell",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SetFocus(FocusTarget::Shell)),
    },
    CommandDef {
        id: "focus.palette",
        title: "Focus: Command Palette",
        shortcut: Some("Ctrl+P"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SetFocus(FocusTarget::CommandPalette)),
    },
    CommandDef {
        id: "focus.next",
        title: "Focus: Next Pane",
        shortcut: Some("Tab"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::NextFocus),
    },
    CommandDef {
        id: "focus.previous",
        title: "Focus: Previous Pane",
        shortcut: Some("Shift+Tab"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::PreviousFocus),
    },
    CommandDef {
        id: "goto.agent",
        title: "Go to Agent",
        shortcut: Some("1"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectMainTab(MainTab::Agent),
            NavCommand::SetFocus(FocusTarget::Main),
        ]),
    },
    CommandDef {
        id: "goto.issues",
        title: "Go to GitHub Issues",
        shortcut: Some("2"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectLeftTab(LeftNavTab::Gh),
            NavCommand::SelectMainTab(MainTab::Issues),
            NavCommand::SetFocus(FocusTarget::Left),
        ]),
    },
    CommandDef {
        id: "goto.prs",
        title: "Go to GitHub PRs",
        shortcut: Some("3"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectLeftTab(LeftNavTab::Gh),
            NavCommand::SelectMainTab(MainTab::Prs),
            NavCommand::SetFocus(FocusTarget::Left),
        ]),
    },
    CommandDef {
        id: "goto.preview",
        title: "Go to Preview",
        shortcut: Some("5"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectMainTab(MainTab::Preview),
            NavCommand::SetFocus(FocusTarget::Main),
        ]),
    },
    CommandDef {
        id: "goto.logs",
        title: "Go to Logs",
        shortcut: Some("6"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectMainTab(MainTab::Logs),
            NavCommand::SetFocus(FocusTarget::Main),
        ]),
    },
    CommandDef {
        id: "goto.files",
        title: "Go to Files",
        shortcut: Some("Alt+1"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectLeftTab(LeftNavTab::Files),
            NavCommand::SetFocus(FocusTarget::Left),
        ]),
    },
    CommandDef {
        id: "goto.git",
        title: "Go to Git Panel",
        shortcut: Some("Alt+2"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectLeftTab(LeftNavTab::Git),
            NavCommand::SetFocus(FocusTarget::Left),
        ]),
    },
    CommandDef {
        id: "goto.diff",
        title: "Go to Diff Panel",
        shortcut: Some("Alt+3"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectLeftTab(LeftNavTab::Diff),
            NavCommand::SetFocus(FocusTarget::Left),
        ]),
    },
    CommandDef {
        id: "goto.gh",
        title: "Go to GitHub Panel",
        shortcut: Some("Alt+4"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectLeftTab(LeftNavTab::Gh),
            NavCommand::SetFocus(FocusTarget::Left),
        ]),
    },
    CommandDef {
        id: "goto.search",
        title: "Go to Search",
        shortcut: Some("Alt+5"),
        context: CommandContext::Always,
        action: PaletteAction::NavigationChain(&[
            NavCommand::SelectLeftTab(LeftNavTab::Search),
            NavCommand::SetFocus(FocusTarget::Left),
        ]),
    },
    CommandDef {
        id: "main.agent",
        title: "Main Tab: Agent",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Agent)),
    },
    CommandDef {
        id: "main.issues",
        title: "Main Tab: Issues",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Issues)),
    },
    CommandDef {
        id: "main.prs",
        title: "Main Tab: PRs",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Prs)),
    },
    CommandDef {
        id: "main.diff",
        title: "Main Tab: Diff",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Diff)),
    },
    CommandDef {
        id: "main.preview",
        title: "Main Tab: Preview",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Preview)),
    },
    CommandDef {
        id: "main.logs",
        title: "Main Tab: Logs",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Logs)),
    },
    CommandDef {
        id: "left.files",
        title: "Left Tab: Files",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Files)),
    },
    CommandDef {
        id: "left.git",
        title: "Left Tab: Git",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Git)),
    },
    CommandDef {
        id: "left.diff",
        title: "Left Tab: Diff",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Diff)),
    },
    CommandDef {
        id: "left.gh",
        title: "Left Tab: GH",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Gh)),
    },
    CommandDef {
        id: "left.search",
        title: "Left Tab: Search",
        shortcut: None,
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Search)),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_command_set_meets_adr_minimum() {
        assert!(
            COMMANDS.len() >= 28,
            "expected at least 28 palette commands, got {}",
            COMMANDS.len()
        );
    }

    #[test]
    fn spec_required_commands_are_registered() {
        let required = [
            "quit",
            "git.refresh",
            "github.refresh",
            "github.issue.comment",
            "github.issue.label",
            "github.open.browser",
            "editor.open",
            "agent.restart",
            "focus.left",
            "focus.main",
            "focus.shell",
            "goto.agent",
            "goto.issues",
        ];
        for id in required {
            assert!(
                COMMANDS.iter().any(|command| command.id == id),
                "missing required command {id}"
            );
        }
    }
}
