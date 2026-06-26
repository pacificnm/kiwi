mod classify;
mod error;
mod launch;
mod resolve;
mod target;

pub use classify::{editor_launch_mode, EditorLaunchMode};
pub use error::EditorLaunchError;
pub use launch::{
    launch_gui_editor, prepare_editor_launch, run_terminal_editor, EditorLaunchResult,
    PreparedEditorLaunch,
};
pub use resolve::{
    command_on_path, resolve_editor_command, resolve_editor_command_with_env, uses_vim_line_arg,
    EditorSource, ResolvedEditorCommand, RESOLUTION_HINT,
};
pub use target::{resolve_editor_target, EditorTarget};
