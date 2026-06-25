mod classify;
mod error;
mod launch;
mod resolve;
mod target;

pub use classify::EditorLaunchMode;
pub use launch::{launch_gui_editor, prepare_editor_launch, run_terminal_editor};
pub use target::resolve_editor_target;
