#[allow(unused_imports)]
pub use kiwi_core::commands::{best_fuzzy_score, CommandContext, COMMANDS, MAX_VISIBLE_MATCHES};

use crate::editor::resolve_editor_target_readonly;
use crate::state::AppState;
use kiwi_core::navigation::MainTab;

pub fn command_title_at(state: &AppState, index: usize) -> Option<&str> {
    if index < COMMANDS.len() {
        Some(COMMANDS[index].title)
    } else {
        state
            .plugins
            .commands
            .get(index - COMMANDS.len())
            .map(|command| command.title.as_str())
    }
}

pub fn refresh_matches(state: &mut AppState) {
    kiwi_core::commands::refresh_matches(&mut state.reduce_view());
}

pub fn command_available_at(state: &AppState, index: usize) -> bool {
    if let Some(command) = COMMANDS.get(index) {
        match command.context {
            CommandContext::Always => true,
            CommandContext::RequiresGitRepo => state.workspace_meta.is_git_repo,
            CommandContext::AgentTab => state.navigation.main_tab == MainTab::Agent,
            CommandContext::DiffTab => state.navigation.main_tab == MainTab::Diff,
            CommandContext::HasEditorTarget => resolve_editor_target_readonly(state).is_some(),
        }
    } else if let Some(command) = state.plugins.commands.get(index - COMMANDS.len()) {
        command.enabled
    } else {
        false
    }
}

pub fn command_shortcut_at(_state: &AppState, index: usize) -> Option<&'static str> {
    COMMANDS.get(index).and_then(|command| command.shortcut)
}
