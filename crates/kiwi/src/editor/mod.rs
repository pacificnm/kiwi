//! TUI-facing editor API; domain logic lives in [`kiwi_core::editor`].

mod target;

#[allow(unused_imports)]
pub use kiwi_core::editor::{
    command_on_path, editor_launch_mode, launch_gui_editor, prepare_editor_launch,
    resolve_editor_command, run_terminal_editor, EditorLaunchError, EditorLaunchMode,
    EditorLaunchResult, EditorSource, EditorTarget, PreparedEditorLaunch, ResolvedEditorCommand,
    RESOLUTION_HINT,
};
pub use target::{resolve_editor_target, resolve_editor_target_readonly};
