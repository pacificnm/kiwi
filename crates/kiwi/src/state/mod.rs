mod app_state;
mod channel;
mod domains;
mod event;
mod reducer;

#[cfg(test)]
mod preservation;

pub use app_state::AppState;
pub use channel::EventChannel;
#[allow(unused_imports)]
pub use channel::EventSender;
#[cfg(test)]
pub use domains::DiffState;
pub use domains::GitHubState;
pub use domains::GitState;
pub use domains::LogLevel;
#[cfg(test)]
pub use domains::{LogEntry, LogsState};
pub use event::{AppCommand, AppEvent, SideEffect};
pub use reducer::agent_spawn_effects_if_needed;
pub use reducer::apply_navigation;
pub use reducer::diff_move_file_effects;
pub use reducer::diff_set_source_effects;
pub use reducer::git_refresh_effects;
pub use reducer::github_refresh_effects;
pub use reducer::reduce;
