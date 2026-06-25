mod classify;
mod error;
mod launch;
mod resolve;

pub use classify::EditorLaunchMode;
pub use launch::{launch_gui_editor, prepare_editor_launch, run_terminal_editor};
