mod fuzzy;
mod registry;

use crate::clipboard::resolve_copy_text_for_focus;
use crate::editor::resolve_editor_target;
use crate::navigation::{MainTab, NavCommand};
use crate::state::{
    apply_navigation, diff_move_file_effects, diff_set_source_effects, git_refresh_effects,
    github_refresh_effects, AppState, SideEffect,
};

pub use fuzzy::{best_fuzzy_score, filter_ranked};
pub use registry::COMMANDS;

pub const MAX_VISIBLE_MATCHES: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandContext {
    Always,
    RequiresGitRepo,
    AgentTab,
    DiffTab,
    HasEditorTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteAction {
    Quit,
    RequestGitRefresh,
    RequestGitHubRefresh,
    AgentRestart,
    Navigation(NavCommand),
    NavigationChain(&'static [NavCommand]),
    LaunchEditor,
    ClipboardCopy,
    ClipboardCut,
    ClipboardPaste,
    DiffToggleSource,
    DiffNextFile,
    DiffPrevFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDef {
    pub id: &'static str,
    pub title: &'static str,
    pub shortcut: Option<&'static str>,
    pub context: CommandContext,
    pub action: PaletteAction,
}

#[must_use]
pub fn command_by_id(id: &str) -> Option<&'static CommandDef> {
    COMMANDS.iter().find(|command| command.id == id)
}

#[must_use]
pub fn history_input_for_id(id: &str) -> String {
    command_by_id(id)
        .map(|command| command.title.to_string())
        .unwrap_or_else(|| id.to_string())
}

#[must_use]
pub fn command_available(state: &AppState, command: &CommandDef) -> bool {
    match command.context {
        CommandContext::Always => true,
        CommandContext::RequiresGitRepo => state.workspace_meta.is_git_repo,
        CommandContext::AgentTab => state.navigation.main_tab == MainTab::Agent,
        CommandContext::DiffTab => state.navigation.main_tab == MainTab::Diff,
        CommandContext::HasEditorTarget => resolve_editor_target(state).is_some(),
    }
}

pub fn refresh_matches(state: &mut AppState) {
    let input = state.palette.input.trim();

    if input.is_empty() {
        let mut matches: Vec<usize> = (0..COMMANDS.len()).collect();
        for command_id in state.palette.history.iter().rev() {
            if let Some(index) = COMMANDS.iter().position(|command| command.id == command_id) {
                matches.retain(|candidate| *candidate != index);
                matches.insert(0, index);
            }
        }
        state.palette.matches = matches.into_iter().take(MAX_VISIBLE_MATCHES).collect();
    } else {
        let ranked = filter_ranked(COMMANDS.len(), input, |index| {
            let command = &COMMANDS[index];
            best_fuzzy_score(command.id, command.title, input)
        });
        state.palette.matches = ranked
            .into_iter()
            .take(MAX_VISIBLE_MATCHES)
            .map(|(index, _)| index)
            .collect();
    }

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

    let palette_clipboard_text = match command.action {
        PaletteAction::ClipboardCopy | PaletteAction::ClipboardCut if state.palette.open => {
            if state.palette.input.is_empty() {
                resolve_copy_text_for_focus(state, state.palette.focus_before_open)
            } else {
                Some(state.palette.input.clone())
            }
        }
        _ => None,
    };

    state.palette.record_history(command.id);
    state.palette.close(&mut state.navigation.focus);

    let mut effects = match command.action {
        PaletteAction::Quit => {
            state.dirty = true;
            vec![SideEffect::Quit]
        }
        PaletteAction::RequestGitRefresh => git_refresh_effects(state),
        PaletteAction::RequestGitHubRefresh => github_refresh_effects(state),
        PaletteAction::AgentRestart => {
            if state.navigation.main_tab != MainTab::Agent {
                return Vec::new();
            }
            state.dirty = true;
            vec![SideEffect::RestartAgent]
        }
        PaletteAction::Navigation(nav) => {
            apply_navigation(state, nav);
            crate::state::agent_spawn_effects_if_needed(state)
        }
        PaletteAction::NavigationChain(chain) => {
            for nav in chain {
                apply_navigation(state, *nav);
            }
            crate::state::agent_spawn_effects_if_needed(state)
        }
        PaletteAction::LaunchEditor => {
            let Some(target) = resolve_editor_target(state) else {
                return Vec::new();
            };
            state.dirty = true;
            vec![SideEffect::LaunchEditor {
                path: target.path,
                line: target.line,
            }]
        }
        PaletteAction::ClipboardCopy => {
            clipboard_palette_effects_from_text(state, palette_clipboard_text, true)
        }
        PaletteAction::ClipboardCut => {
            clipboard_palette_effects_from_text(state, palette_clipboard_text, false)
        }
        PaletteAction::ClipboardPaste => {
            state.dirty = true;
            vec![SideEffect::PasteFromClipboard]
        }
        PaletteAction::DiffToggleSource => {
            diff_set_source_effects(state, state.diff.source.toggle())
        }
        PaletteAction::DiffNextFile => diff_move_file_effects(state, 1),
        PaletteAction::DiffPrevFile => diff_move_file_effects(state, -1),
    };

    if state.config.workspace.persist {
        effects.push(SideEffect::SavePaletteHistory);
    }

    effects
}

fn clipboard_palette_effects_from_text(
    state: &mut AppState,
    text: Option<String>,
    copy_only: bool,
) -> Vec<SideEffect> {
    let Some(text) = text.filter(|value| !value.is_empty()) else {
        state.notifications.show_toast(if copy_only {
            "Nothing to copy"
        } else {
            "Nothing to cut"
        });
        state.dirty = true;
        return Vec::new();
    };

    if !copy_only {
        refresh_matches(state);
    }

    state.dirty = true;
    vec![SideEffect::CopyToClipboard(text)]
}

#[cfg(test)]
mod tests {
    use crate::config::ResolvedConfig;
    use crate::layout::compute_layout;
    use crate::layout::FocusTarget;
    use crate::navigation::{MainTab, NavCommand};
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
        assert!(effects.contains(&SideEffect::SavePaletteHistory));
        assert!(state.git.loading);
        assert!(!state.palette.open);
    }

    #[test]
    fn execute_github_refresh_emits_side_effect() {
        let mut state = test_state();
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "github.refresh")
            .expect("github refresh");
        let effects = execute_command(&mut state, index);
        assert!(effects.contains(&SideEffect::SpawnGitHubAuthCheck));
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

    #[test]
    fn goto_agent_selects_tab_and_focuses_main() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        state.navigation.focus = FocusTarget::Shell;
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "goto.agent")
            .expect("goto agent");
        execute_command(&mut state, index);
        assert_eq!(state.navigation.main_tab, MainTab::Agent);
        assert_eq!(state.navigation.focus, FocusTarget::Main);
    }

    #[test]
    fn execute_editor_open_uses_preview_line() {
        let mut state = test_state();
        state.preview.path = Some(std::path::PathBuf::from("src/main.rs"));
        state.preview.scroll_offset = 4;
        state
            .navigation
            .apply(NavCommand::SetFocus(FocusTarget::Main));
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Preview));
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "editor.open")
            .expect("editor open");
        let effects = execute_command(&mut state, index);
        assert!(effects.contains(&SideEffect::LaunchEditor {
            path: std::path::PathBuf::from("src/main.rs"),
            line: Some(5),
        }));
    }

    #[test]
    fn history_input_uses_command_title() {
        assert_eq!(history_input_for_id("git.refresh"), "Git: Refresh Status");
    }
}
