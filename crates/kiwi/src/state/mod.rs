mod app_state;
mod channel;
pub mod domains;
mod event;
mod reducer;

#[cfg(test)]
mod preservation;

pub use app_state::AppState;
pub use channel::EventChannel;
#[allow(unused_imports)]
pub use channel::EventSender;
#[cfg(test)]
pub use domains::{
    AgentState, BranchState, CommandPaletteState, DiffState, GitHubPrCreatePrompt,
    GitHubPrCreateStep, GitHubState, GitState, LogEntry, LogsState, PalettePrompt, SettingsState,
    ShellState, StatusBarState, WorkspaceMeta,
};
pub use domains::{LogLevel, PluginPaletteCommand, PluginsState};
pub use event::{AppCommand, AppEvent, SideEffect};
pub use reducer::agent_spawn_effects_if_needed;
pub use reducer::file_tree_startup_effects;
pub use reducer::reduce;
pub use reducer::workspace_restore_effects;
