use std::path::Path;

use crate::config::EditorSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorLaunchMode {
    /// Detached spawn; Kiwi keeps the terminal.
    Gui,
    /// Suspend Kiwi, run editor on the controlling TTY, then resume.
    Terminal,
}

pub fn editor_launch_mode(command: &str, settings: &EditorSettings) -> EditorLaunchMode {
    if let Some(terminal) = settings.terminal {
        return if terminal {
            EditorLaunchMode::Terminal
        } else {
            EditorLaunchMode::Gui
        };
    }

    if is_known_gui_editor(command) {
        EditorLaunchMode::Gui
    } else {
        EditorLaunchMode::Terminal
    }
}

fn editor_basename(command: &str) -> &str {
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
}

fn is_known_gui_editor(command: &str) -> bool {
    matches!(
        editor_basename(command),
        "code" | "cursor" | "zed" | "subl" | "atom" | "windsurf" | "fleet" | "idea" | "pycharm"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EditorSettings;

    #[test]
    fn gui_editors_launch_detached() {
        let settings = EditorSettings::default();
        assert_eq!(
            editor_launch_mode("cursor", &settings),
            EditorLaunchMode::Gui
        );
        assert_eq!(editor_launch_mode("code", &settings), EditorLaunchMode::Gui);
    }

    #[test]
    fn terminal_editors_suspend_kiwi() {
        let settings = EditorSettings::default();
        assert_eq!(
            editor_launch_mode("nvim", &settings),
            EditorLaunchMode::Terminal
        );
        assert_eq!(
            editor_launch_mode("nano", &settings),
            EditorLaunchMode::Terminal
        );
    }

    #[test]
    fn config_terminal_override_wins() {
        let settings = EditorSettings {
            configured_command: Some("cursor".to_string()),
            terminal: Some(true),
        };
        assert_eq!(
            editor_launch_mode("cursor", &settings),
            EditorLaunchMode::Terminal
        );
    }
}
