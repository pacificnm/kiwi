mod fuzzy;
mod registry;

use kiwi_plugin_api::PluginResult;
use kiwi_plugin_loader::{invoke_plugin_command, PluginInvokeOutcome};

use crate::clipboard::resolve_copy_text_for_focus;
use crate::editor::resolve_editor_target;
use crate::events::{AgentEffect, FsEffect, GitEffect, GitHubEffect, SideEffect};
use crate::github::{missing_browser_target_message, resolve_browser_target, LabelPickerState};
use crate::navigation::{FocusTarget, LeftNavTab, MainTab, NavCommand};
use crate::reducer::{
    agent_cycle_effects, agent_new_effects, agent_spawn_effects_if_needed, apply_navigation,
    diff_move_file_effects, diff_set_source_effects, git_refresh_effects, github_refresh_effects,
};
use crate::state::{GitHubPrCreatePrompt, PalettePrompt, ReduceView};

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
    AgentNew,
    AgentCycleNext,
    Navigation(NavCommand),
    NavigationChain(&'static [NavCommand]),
    LaunchEditor,
    ClipboardCopy,
    ClipboardCut,
    ClipboardPaste,
    DiffToggleSource,
    DiffNextFile,
    DiffPrevFile,
    GitHubIssueCommentPrompt,
    GitHubIssueLabelPicker,
    GitHubIssueCreateBranch,
    GitHubOpenInBrowser,
    GitHubPrCreatePrompt,
    GitHubPrMerge,
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
pub fn command_index_by_id(state: &ReduceView<'_>, id: &str) -> Option<usize> {
    COMMANDS
        .iter()
        .position(|command| command.id == id)
        .or_else(|| {
            state
                .plugins
                .commands
                .iter()
                .position(|command| command.id == id)
                .map(|index| COMMANDS.len() + index)
        })
}

#[must_use]
#[allow(dead_code)]
pub fn command_by_id<'a>(state: &'a ReduceView<'_>, id: &str) -> Option<&'a CommandDef> {
    command_index_by_id(state, id).and_then(|index| COMMANDS.get(index))
}

#[must_use]
pub fn history_input_for_id(state: &ReduceView<'_>, id: &str) -> String {
    command_index_by_id(state, id)
        .and_then(|index| command_title_at(state, index).map(str::to_string))
        .unwrap_or_else(|| id.to_string())
}

#[must_use]
pub fn command_count(state: &ReduceView<'_>) -> usize {
    COMMANDS.len() + state.plugins.commands.len()
}

#[must_use]
pub fn command_title_at<'a>(state: &'a ReduceView<'a>, index: usize) -> Option<&'a str> {
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

#[must_use]
pub fn command_shortcut_at(_state: &ReduceView<'_>, index: usize) -> Option<&'static str> {
    COMMANDS.get(index).and_then(|command| command.shortcut)
}

#[must_use]
pub fn command_available_at(state: &ReduceView<'_>, index: usize) -> bool {
    if let Some(command) = COMMANDS.get(index) {
        command_available(state, command)
    } else if let Some(command) = state.plugins.commands.get(index - COMMANDS.len()) {
        command.enabled
    } else {
        false
    }
}

#[must_use]
pub fn command_available(state: &ReduceView<'_>, command: &CommandDef) -> bool {
    match command.context {
        CommandContext::Always => true,
        CommandContext::RequiresGitRepo => state.workspace_meta.is_git_repo,
        CommandContext::AgentTab => state.navigation.main_tab == MainTab::Agent,
        CommandContext::DiffTab => state.navigation.main_tab == MainTab::Diff,
        CommandContext::HasEditorTarget => resolve_editor_target(state).is_some(),
    }
}

pub fn refresh_matches(state: &mut ReduceView<'_>) {
    let input = state.palette.input.trim();
    let total = command_count(state);

    if input.is_empty() {
        let mut matches: Vec<usize> = (0..total).collect();
        for command_id in &state.palette.history {
            if let Some(index) = command_index_by_id(state, command_id) {
                matches.retain(|candidate| *candidate != index);
                matches.insert(0, index);
            }
        }
        prioritize_github_issue_commands(state, &mut matches);
        state.palette.matches = matches.into_iter().take(MAX_VISIBLE_MATCHES).collect();
    } else {
        let ranked = filter_ranked(total, input, |index| {
            let (id, title) = match COMMANDS.get(index) {
                Some(command) => (command.id, command.title),
                None => {
                    let command = &state.plugins.commands[index - COMMANDS.len()];
                    (command.id.as_str(), command.title.as_str())
                }
            };
            best_fuzzy_score(id, title, input)
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

fn prioritize_github_issue_commands(state: &ReduceView<'_>, matches: &mut Vec<usize>) {
    let pr_surface = state.navigation.main_tab == MainTab::Prs
        || (state.navigation.left_tab == LeftNavTab::Gh
            && state.github.left_pane == crate::github::GitHubLeftPane::Prs);
    let issue_surface = state.navigation.main_tab == MainTab::Issues
        || (state.navigation.left_tab == LeftNavTab::Gh
            && state.github.left_pane == crate::github::GitHubLeftPane::Issues);

    let command_ids: &[&str] = if pr_surface {
        &[
            "github.pr.create",
            "github.pr.merge",
            "github.open.browser",
            "github.refresh",
        ]
    } else if issue_surface {
        &[
            "github.open.browser",
            "github.issue.branch",
            "github.issue.comment",
            "github.issue.label",
            "github.refresh",
        ]
    } else {
        return;
    };

    for command_id in command_ids {
        let Some(index) = COMMANDS
            .iter()
            .position(|command| command.id == *command_id)
        else {
            continue;
        };
        matches.retain(|candidate| *candidate != index);
        matches.insert(0, index);
    }
}

pub fn execute_command(state: &mut ReduceView<'_>, registry_index: usize) -> Vec<SideEffect> {
    if registry_index < COMMANDS.len() {
        execute_builtin_command(state, registry_index)
    } else {
        execute_plugin_command(state, registry_index - COMMANDS.len())
    }
}

fn execute_builtin_command(state: &mut ReduceView<'_>, registry_index: usize) -> Vec<SideEffect> {
    let Some(command) = COMMANDS.get(registry_index) else {
        return Vec::new();
    };

    if !command_available(state, command) {
        if let Some(message) = unavailable_command_message(state, command) {
            state.notifications.show_toast(message);
            state.set_dirty();
        }
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
    if !matches!(
        command.action,
        PaletteAction::GitHubIssueCommentPrompt | PaletteAction::GitHubPrCreatePrompt
    ) {
        state.navigation.focus = state.palette.close();
    }

    let mut effects = match command.action {
        PaletteAction::Quit => {
            state.set_dirty();
            vec![SideEffect::Quit]
        }
        PaletteAction::RequestGitRefresh => git_refresh_effects(state),
        PaletteAction::RequestGitHubRefresh => github_refresh_effects(state),
        PaletteAction::AgentRestart => {
            if state.navigation.main_tab != MainTab::Agent {
                return Vec::new();
            }
            state.set_dirty();
            vec![SideEffect::Agent(AgentEffect::Restart(state.agent_manager.active_id()))]
        }
        PaletteAction::AgentNew => agent_new_effects(state),
        PaletteAction::AgentCycleNext => agent_cycle_effects(state, 1),
        PaletteAction::Navigation(nav) => {
            apply_navigation(state, nav);
            agent_spawn_effects_if_needed(state)
        }
        PaletteAction::NavigationChain(chain) => {
            for nav in chain {
                apply_navigation(state, *nav);
            }
            agent_spawn_effects_if_needed(state)
        }
        PaletteAction::LaunchEditor => {
            let Some(target) = resolve_editor_target(state) else {
                return Vec::new();
            };
            state.set_dirty();
            vec![SideEffect::Fs(FsEffect::LaunchEditor {
                path: target.path,
                line: target.line,
            })]
        }
        PaletteAction::ClipboardCopy => {
            clipboard_palette_effects_from_text(state, palette_clipboard_text, true)
        }
        PaletteAction::ClipboardCut => {
            clipboard_palette_effects_from_text(state, palette_clipboard_text, false)
        }
        PaletteAction::ClipboardPaste => {
            state.set_dirty();
            vec![SideEffect::PasteFromClipboard]
        }
        PaletteAction::DiffToggleSource => {
            diff_set_source_effects(state, state.diff.source.toggle())
        }
        PaletteAction::DiffNextFile => diff_move_file_effects(state, 1),
        PaletteAction::DiffPrevFile => diff_move_file_effects(state, -1),
        PaletteAction::GitHubIssueCommentPrompt => github_issue_comment_prompt_effects(state),
        PaletteAction::GitHubIssueLabelPicker => github_issue_label_picker_effects(state),
        PaletteAction::GitHubIssueCreateBranch => github_issue_create_branch_effects(state),
        PaletteAction::GitHubOpenInBrowser => github_open_in_browser_effects(state),
        PaletteAction::GitHubPrCreatePrompt => github_pr_create_prompt_effects(state),
        PaletteAction::GitHubPrMerge => github_pr_merge_effects(state),
    };

    if state.config.workspace.persist {
        effects.push(SideEffect::SaveWorkspace);
    }

    effects
}

fn execute_plugin_command(state: &mut ReduceView<'_>, plugin_index: usize) -> Vec<SideEffect> {
    let Some(command) = state.plugins.commands.get(plugin_index) else {
        return Vec::new();
    };

    if !command.enabled {
        state
            .notifications
            .show_toast("Plugin command is disabled for this session");
        state.set_dirty();
        return Vec::new();
    }

    let plugin_name = command.plugin_name.clone();
    let command_id = command.id.clone();
    let callback = command.callback;

    state.palette.record_history(&command_id);
    state.navigation.focus = state.palette.close();

    match invoke_plugin_command(callback) {
        PluginInvokeOutcome::Completed(PluginResult::Ok) => {}
        PluginInvokeOutcome::Completed(PluginResult::Err) => {
            state
                .notifications
                .show_toast(format!("Plugin command `{command_id}` failed"));
        }
        PluginInvokeOutcome::Panicked => {
            disable_plugin_commands(state, &plugin_name);
            state.notifications.show_toast(format!(
                "Plugin `{plugin_name}` panicked; its commands are disabled for this session"
            ));
        }
    }
    state.set_dirty();

    let mut effects = Vec::new();
    if state.config.workspace.persist {
        effects.push(SideEffect::SaveWorkspace);
    }
    effects
}

fn disable_plugin_commands(state: &mut ReduceView<'_>, plugin_name: &str) {
    for command in &mut state.plugins.commands {
        if command.plugin_name == plugin_name {
            command.enabled = false;
        }
    }
}

fn clipboard_palette_effects_from_text(
    state: &mut ReduceView<'_>,
    text: Option<String>,
    copy_only: bool,
) -> Vec<SideEffect> {
    let Some(text) = text.filter(|value| !value.is_empty()) else {
        state.notifications.show_toast(if copy_only {
            "Nothing to copy"
        } else {
            "Nothing to cut"
        });
        state.set_dirty();
        return Vec::new();
    };

    if !copy_only {
        refresh_matches(state);
    }

    state.set_dirty();
    vec![SideEffect::CopyToClipboard(text)]
}

fn unavailable_command_message(
    state: &ReduceView<'_>,
    command: &CommandDef,
) -> Option<&'static str> {
    match command.context {
        CommandContext::RequiresGitRepo if !state.workspace_meta.is_git_repo => {
            Some("Not in a git repository")
        }
        CommandContext::AgentTab if state.navigation.main_tab != MainTab::Agent => {
            Some("Switch to the Agent tab first")
        }
        CommandContext::DiffTab if state.navigation.main_tab != MainTab::Diff => {
            Some("Switch to the Diff tab first")
        }
        CommandContext::HasEditorTarget if resolve_editor_target(state).is_none() => {
            Some("No file selected to open in an editor")
        }
        _ => Some("Command unavailable in current context"),
    }
}

fn github_issue_comment_prompt_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(number) = selected_issue_number(state) else {
        state
            .notifications
            .show_toast("Select an issue in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    let focus = state.navigation.focus;
    state
        .palette
        .begin_prompt(PalettePrompt::GitHubIssueComment { number }, focus);
    state.navigation.focus = FocusTarget::CommandPalette;
    state.set_dirty();
    Vec::new()
}

fn github_issue_label_picker_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(number) = selected_issue_number(state) else {
        state
            .notifications
            .show_toast("Select an issue in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    let existing_labels = issue_labels_for_number(state, number);
    state.github.label_picker = Some(LabelPickerState::new(number, existing_labels));
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnRepoLabels)]
}

fn github_issue_create_branch_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(number) = selected_issue_number(state) else {
        state
            .notifications
            .show_toast("Select an issue in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    state.github.issue_action_message = Some(format!("Creating branch for issue #{number}..."));
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnIssueCreateBranch { number })]
}

fn github_open_in_browser_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(target) = resolve_browser_target(state.navigation, state.github) else {
        state
            .notifications
            .show_toast(missing_browser_target_message(
                state.navigation,
                state.github,
            ));
        state.set_dirty();
        return Vec::new();
    };

    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnOpenBrowser { target })]
}

fn github_pr_create_prompt_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let focus = state.navigation.focus;
    state.palette.begin_prompt(
        PalettePrompt::GitHubPrCreate(GitHubPrCreatePrompt::default()),
        focus,
    );
    state.navigation.focus = FocusTarget::CommandPalette;
    state.set_dirty();
    Vec::new()
}

fn github_pr_merge_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if !state.github.auth_ok {
        state
            .notifications
            .show_toast("GitHub authentication required");
        state.set_dirty();
        return Vec::new();
    }

    let Some(number) = state
        .github
        .selected_pr
        .and_then(|value| u32::try_from(value).ok())
    else {
        state
            .notifications
            .show_toast("Select a PR in the GH left list first");
        state.set_dirty();
        return Vec::new();
    };

    let mergeable = crate::github::selected_pull_request(state.github)
        .is_some_and(crate::github::pull_request_is_mergeable);
    if !mergeable {
        state
            .notifications
            .show_toast("Only open, non-draft pull requests can be merged");
        state.set_dirty();
        return Vec::new();
    }

    state.github.issue_action_message = Some(format!("Merging pull request #{number}..."));
    state.set_dirty();
    vec![SideEffect::GitHub(GitHubEffect::SpawnPrMerge { number })]
}

fn selected_issue_number(state: &ReduceView<'_>) -> Option<u32> {
    state
        .github
        .selected_issue
        .and_then(|value| u32::try_from(value).ok())
}

fn issue_labels_for_number(state: &ReduceView<'_>, number: u32) -> Vec<String> {
    if state
        .github
        .issue_detail
        .as_ref()
        .is_some_and(|detail| detail.number == number)
    {
        return state
            .github
            .issue_detail
            .as_ref()
            .map(|detail| detail.labels.clone())
            .unwrap_or_default();
    }

    state
        .github
        .issues
        .iter()
        .find(|issue| issue.number == number)
        .map(|issue| issue.labels.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ResolvedConfig;
    use crate::events::{AgentEffect, FsEffect, GitEffect, GitHubEffect, SideEffect};
    use crate::navigation::FocusTarget;
    use crate::navigation::{MainTab, NavCommand};
    use crate::state::{AppState, ReduceView, ViewportMetrics};
    use crate::theme::{load_theme_with_capabilities, TerminalCapabilities};

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
            TerminalCapabilities::TrueColor,
            ViewportMetrics::default(),
        )
    }

    fn view<'a>(state: &'a mut AppState) -> ReduceView<'a> {
        ReduceView::from_app_state(state)
    }

    #[test]
    fn fuzzy_find_git_ref_matches_refresh_command() {
        let mut state = test_state();
        state.palette.input = "git ref".to_string();
        refresh_matches(&mut view(&mut state));
        let first = state.palette.matches.first().copied().expect("match");
        assert_eq!(COMMANDS[first].id, "git.refresh");
    }

    #[test]
    fn unavailable_commands_still_listed() {
        let mut state = test_state();
        state.workspace_meta.is_git_repo = false;
        state.palette.input.clear();
        refresh_matches(&mut view(&mut state));
        assert!(state
            .palette
            .matches
            .iter()
            .any(|index| COMMANDS[*index].id == "git.refresh"));
        let git_refresh = COMMANDS
            .iter()
            .find(|command| command.id == "git.refresh")
            .expect("git refresh");
        assert!(!command_available(&view(&mut state), git_refresh));
    }

    #[test]
    fn execute_git_refresh_emits_side_effect() {
        let mut state = test_state();
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "git.refresh")
            .expect("git refresh");
        let effects = execute_command(&mut view(&mut state), index);
        assert!(effects.contains(&SideEffect::Git(GitEffect::SpawnRefresh)));
        assert!(effects.contains(&SideEffect::SaveWorkspace));
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
        let effects = execute_command(&mut view(&mut state), index);
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnAuthCheck)));
    }

    #[test]
    fn execute_agent_restart_requires_agent_tab() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "agent.restart")
            .expect("agent restart");
        assert!(execute_command(&mut view(&mut state), index).is_empty());
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
        execute_command(&mut view(&mut state), index);
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
        let effects = execute_command(&mut view(&mut state), index);
        assert!(effects.contains(&SideEffect::Fs(FsEffect::LaunchEditor {
            path: std::path::PathBuf::from("src/main.rs"),
            line: Some(5),
        })));
    }

    #[test]
    fn github_issue_commands_surface_on_issues_tab() {
        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));
        refresh_matches(&mut view(&mut state));
        let ids: Vec<_> = state
            .palette
            .matches
            .iter()
            .map(|index| COMMANDS[*index].id)
            .collect();
        assert!(ids.contains(&"github.issue.comment"));
        assert!(ids.contains(&"github.open.browser"));
        assert!(ids.contains(&"github.issue.label"));
        assert!(ids.contains(&"github.issue.branch"));
    }

    #[test]
    fn fuzzy_find_issue_comment_command() {
        let mut state = test_state();
        state.palette.input = "issue comment".to_string();
        refresh_matches(&mut view(&mut state));
        assert!(state
            .palette
            .matches
            .iter()
            .any(|index| { COMMANDS[*index].id == "github.issue.comment" }));
    }

    #[test]
    fn execute_issue_comment_prompt_opens_palette_prompt() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.selected_issue = Some(7);
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "github.issue.comment")
            .expect("github issue comment");
        let effects = execute_command(&mut view(&mut state), index);
        assert!(!effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::GitHub(GitHubEffect::SpawnIssueComment { .. })
                    | SideEffect::GitHub(GitHubEffect::SpawnIssueList)
                    | SideEffect::GitHub(GitHubEffect::SpawnPrList)
                    | SideEffect::GitHub(GitHubEffect::SpawnIssueDetail { .. })
                    | SideEffect::GitHub(GitHubEffect::SpawnPrDetail { .. })
            )
        }));
        assert!(state.palette.open);
        assert!(state.palette.prompt.is_some());
        assert_eq!(state.navigation.focus, FocusTarget::CommandPalette);
    }

    #[test]
    fn execute_open_in_browser_spawns_side_effect() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.selected_issue = Some(7);
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Issues));
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "github.open.browser")
            .expect("github open browser");
        let effects = execute_command(&mut view(&mut state), index);
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::GitHub(GitHubEffect::SpawnOpenBrowser {
                    target: crate::github::GitHubBrowserTarget::Issue(7)
                })
            )
        }));
    }

    #[test]
    fn execute_issue_label_picker_spawns_label_load() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.selected_issue = Some(7);
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "github.issue.label")
            .expect("github issue label");
        let effects = execute_command(&mut view(&mut state), index);
        assert!(state.github.label_picker.is_some());
        assert!(effects.contains(&SideEffect::GitHub(GitHubEffect::SpawnRepoLabels)));
        assert!(!state.palette.open);
    }

    #[test]
    fn execute_issue_create_branch_spawns_side_effect() {
        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.selected_issue = Some(7);
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "github.issue.branch")
            .expect("github issue branch");
        let effects = execute_command(&mut view(&mut state), index);
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                SideEffect::GitHub(GitHubEffect::SpawnIssueCreateBranch { number: 7 })
            )
        }));
        assert!(state
            .github
            .issue_action_message
            .as_ref()
            .is_some_and(|message| message.contains("Creating branch")));
        assert!(!state.palette.open);
    }

    #[test]
    fn execute_pr_merge_spawns_side_effect_for_open_pr() {
        use crate::github::{PrState, PullRequest};

        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.prs = vec![PullRequest {
            number: 17,
            title: "Merge me".to_string(),
            state: PrState::Open,
            author: "dev".to_string(),
            is_draft: false,
        }];
        state.github.selected_pr = Some(17);
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "github.pr.merge")
            .expect("github pr merge");
        let effects = execute_command(&mut view(&mut state), index);
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, SideEffect::GitHub(GitHubEffect::SpawnPrMerge { number: 17 }))));
        assert!(state
            .github
            .issue_action_message
            .as_ref()
            .is_some_and(|message| message.contains("Merging pull request #17")));
    }

    #[test]
    fn execute_pr_merge_rejects_draft_without_spawn() {
        use crate::github::{PrState, PullRequest};

        let mut state = test_state();
        state.github.auth_ok = true;
        state.github.prs = vec![PullRequest {
            number: 17,
            title: "Draft".to_string(),
            state: PrState::Draft,
            author: "dev".to_string(),
            is_draft: true,
        }];
        state.github.selected_pr = Some(17);
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "github.pr.merge")
            .expect("github pr merge");
        let effects = execute_command(&mut view(&mut state), index);
        assert!(!effects
            .iter()
            .any(|effect| matches!(effect, SideEffect::GitHub(GitHubEffect::SpawnPrMerge { .. }))));
    }

    #[test]
    fn execute_pr_create_prompt_opens_palette_prompt() {
        let mut state = test_state();
        state.github.auth_ok = true;
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "github.pr.create")
            .expect("github pr create");
        let effects = execute_command(&mut view(&mut state), index);
        assert!(!effects
            .iter()
            .any(|effect| { matches!(effect, SideEffect::GitHub(GitHubEffect::SpawnPrCreate { .. })) }));
        assert!(state.palette.open);
        assert!(state.palette.prompt.is_some());
        assert_eq!(state.navigation.focus, FocusTarget::CommandPalette);
    }

    #[test]
    fn github_pr_create_command_surfaces_on_prs_tab() {
        let mut state = test_state();
        state
            .navigation
            .apply(NavCommand::SelectMainTab(MainTab::Prs));
        refresh_matches(&mut view(&mut state));
        let ids: Vec<_> = state
            .palette
            .matches
            .iter()
            .map(|index| COMMANDS[*index].id)
            .collect();
        assert!(ids.contains(&"github.pr.create"));
        assert!(ids.contains(&"github.pr.merge"));
        assert!(ids.contains(&"github.open.browser"));
    }

    #[test]
    fn history_input_uses_command_title() {
        let mut state = test_state();
        assert_eq!(
            history_input_for_id(&view(&mut state), "git.refresh"),
            "Git: Refresh Status"
        );
    }

    #[test]
    fn empty_input_surfaces_recent_history_first() {
        let mut state = test_state();
        state.palette.history = vec!["goto.agent".to_string(), "git.refresh".to_string()];
        state.palette.input.clear();
        refresh_matches(&mut view(&mut state));

        assert!(state.palette.matches.len() >= 2);
        assert_eq!(COMMANDS[state.palette.matches[0]].id, "git.refresh");
        assert_eq!(COMMANDS[state.palette.matches[1]].id, "goto.agent");
    }

    #[test]
    fn execute_command_records_history() {
        let mut state = test_state();
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "goto.agent")
            .expect("goto agent");
        execute_command(&mut view(&mut state), index);
        assert_eq!(
            state.palette.history.last().map(String::as_str),
            Some("goto.agent")
        );
    }

    #[test]
    fn execute_skips_save_when_persistence_disabled() {
        let mut state = test_state();
        state.config.workspace.persist = false;
        let index = COMMANDS
            .iter()
            .position(|command| command.id == "git.refresh")
            .expect("git refresh");
        let effects = execute_command(&mut view(&mut state), index);
        assert!(!effects.contains(&SideEffect::SaveWorkspace));
        assert_eq!(
            state.palette.history.last().map(String::as_str),
            Some("git.refresh")
        );
    }
}
