mod fuzzy;

use std::path::PathBuf;

use crate::layout::FocusTarget;
use crate::navigation::{LeftNavTab, MainTab, NavCommand};
use crate::state::{AppState, SideEffect};

pub use fuzzy::best_fuzzy_score;

pub const MAX_VISIBLE_MATCHES: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandContext {
    Always,
    RequiresGitRepo,
    AgentTab,
    HasEditorTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteAction {
    Quit,
    RequestGitRefresh,
    AgentRestart,
    Navigation(NavCommand),
    LaunchEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDef {
    pub id: &'static str,
    pub title: &'static str,
    pub shortcut: Option<&'static str>,
    pub context: CommandContext,
    pub action: PaletteAction,
}

pub const COMMANDS: &[CommandDef] = &[
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
        id: "main.agent",
        title: "Main Tab: Agent",
        shortcut: Some("1"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Agent)),
    },
    CommandDef {
        id: "main.issues",
        title: "Main Tab: Issues",
        shortcut: Some("2"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Issues)),
    },
    CommandDef {
        id: "main.prs",
        title: "Main Tab: PRs",
        shortcut: Some("3"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Prs)),
    },
    CommandDef {
        id: "main.diff",
        title: "Main Tab: Diff",
        shortcut: Some("4"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Diff)),
    },
    CommandDef {
        id: "main.preview",
        title: "Main Tab: Preview",
        shortcut: Some("5"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Preview)),
    },
    CommandDef {
        id: "main.logs",
        title: "Main Tab: Logs",
        shortcut: Some("6"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectMainTab(MainTab::Logs)),
    },
    CommandDef {
        id: "left.files",
        title: "Left Tab: Files",
        shortcut: Some("Alt+1"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Files)),
    },
    CommandDef {
        id: "left.git",
        title: "Left Tab: Git",
        shortcut: Some("Alt+2"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Git)),
    },
    CommandDef {
        id: "left.diff",
        title: "Left Tab: Diff",
        shortcut: Some("Alt+3"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Diff)),
    },
    CommandDef {
        id: "left.gh",
        title: "Left Tab: GH",
        shortcut: Some("Alt+4"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Gh)),
    },
    CommandDef {
        id: "left.search",
        title: "Left Tab: Search",
        shortcut: Some("Alt+5"),
        context: CommandContext::Always,
        action: PaletteAction::Navigation(NavCommand::SelectLeftTab(LeftNavTab::Search)),
    },
];

#[must_use]
pub fn command_available(state: &AppState, command: &CommandDef) -> bool {
    match command.context {
        CommandContext::Always => true,
        CommandContext::RequiresGitRepo => state.workspace_meta.is_git_repo,
        CommandContext::AgentTab => state.navigation.main_tab == MainTab::Agent,
        CommandContext::HasEditorTarget => editor_target(state).is_some(),
    }
}

pub fn refresh_matches(state: &mut AppState) {
    let input = state.palette.input.trim();
    let mut scored = Vec::new();

    if input.is_empty() {
        for (index, _command) in COMMANDS.iter().enumerate() {
            scored.push((index, 0u32));
        }
        for command_id in state.palette.history.iter().rev() {
            if let Some(index) = COMMANDS.iter().position(|command| command.id == command_id) {
                if let Some(position) = scored.iter().position(|(candidate, _)| *candidate == index)
                {
                    scored.remove(position);
                }
                scored.insert(0, (index, 0));
            }
        }
    } else {
        for (index, command) in COMMANDS.iter().enumerate() {
            if let Some(score) = best_fuzzy_score(command.id, command.title, input) {
                scored.push((index, score));
            }
        }
        scored.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    }

    state.palette.matches = scored
        .into_iter()
        .take(MAX_VISIBLE_MATCHES)
        .map(|(index, _)| index)
        .collect();

    if state.palette.selected >= state.palette.matches.len() {
        state.palette.selected = state.palette.matches.len().saturating_sub(1);
    }
}

pub fn execute_command(state: &mut AppState, registry_index: usize) -> Vec<SideEffect> {
    let Some(command) = COMMANDS.get(registry_index) else {
        return Vec::new();
    };

    if !command_available(state, command) {
        return Vec::new();
    }

    state.palette.record_history(command.id);
    state.palette.close(&mut state.navigation.focus);

    match command.action {
        PaletteAction::Quit => {
            state.dirty = true;
            vec![SideEffect::Quit]
        }
        PaletteAction::RequestGitRefresh => {
            state.dirty = true;
            vec![SideEffect::SpawnGitRefresh]
        }
        PaletteAction::AgentRestart => {
            if state.navigation.main_tab != MainTab::Agent {
                return Vec::new();
            }
            state.dirty = true;
            vec![SideEffect::RestartAgent]
        }
        PaletteAction::Navigation(nav) => {
            let before = state.navigation.clone();
            state.navigation.apply(nav);
            if state.navigation != before {
                state.dirty = true;
            }
            crate::state::agent_spawn_effects_if_needed(state)
        }
        PaletteAction::LaunchEditor => {
            let Some(path) = editor_target(state) else {
                return Vec::new();
            };
            state.dirty = true;
            vec![SideEffect::LaunchEditor(path)]
        }
    }
}

fn editor_target(state: &AppState) -> Option<PathBuf> {
    state
        .preview
        .path
        .as_deref()
        .or(state.file_tree.selected_path.as_deref())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::navigation::MainTab;
    use crate::state::{AppState, SideEffect};
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            std::path::PathBuf::from("."),
            true,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        )
    }

    #[test]
    fn fuzzy_find_git_ref_matches_refresh_command() {
        let mut state = test_state();
        state.palette.input = "git ref".to_string();
        refresh_matches(&mut state);
        let first = state.palette.matches.first().copied().expect("match");
        assert_eq!(COMMANDS[first].id, "git.refresh");
    }

    #[test]
    fn unavailable_commands_still_listed() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = false;
        state.palette.input.clear();
        refresh_matches(&mut state);
        assert!(state
            .palette
            .matches
            .iter()
            .any(|index| COMMANDS[*index].id == "git.refresh"));
        let git_refresh = COMMANDS
            .iter()
            .find(|command| command.id == "git.refresh")
            .expect("git refresh");
        assert!(!command_available(&state, git_refresh));
    }

    #[test]
    fn execute_git_refresh_emits_side_effect() {
        let mut state = test_state();
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "git.refresh")
            .expect("git refresh");
        let effects = execute_command(&mut state, index);
        assert!(effects.contains(&SideEffect::SpawnGitRefresh));
        assert!(!state.palette.open);
    }

    #[test]
    fn execute_agent_restart_requires_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "agent.restart")
            .expect("agent restart");
        assert!(execute_command(&mut state, index).is_empty());
    }
}
