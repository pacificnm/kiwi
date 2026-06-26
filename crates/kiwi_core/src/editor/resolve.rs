use std::path::Path;

use crate::config::EditorSettings;

pub const RESOLUTION_HINT: &str =
    "Set [editor] command in config, or export $VISUAL / $EDITOR (fallback: nano).";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorSource {
    Config,
    Visual,
    Editor,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEditorCommand {
    pub command: String,
    pub source: EditorSource,
}

pub fn resolve_editor_command(settings: &EditorSettings) -> ResolvedEditorCommand {
    resolve_editor_command_with_env(
        settings,
        std::env::var("VISUAL").ok(),
        std::env::var("EDITOR").ok(),
    )
}

pub fn resolve_editor_command_with_env(
    settings: &EditorSettings,
    visual: Option<String>,
    editor: Option<String>,
) -> ResolvedEditorCommand {
    if let Some(command) = settings.configured_command.as_ref() {
        return ResolvedEditorCommand {
            command: command.clone(),
            source: EditorSource::Config,
        };
    }

    if let Some(visual) = visual {
        let visual = visual.trim();
        if !visual.is_empty() {
            return ResolvedEditorCommand {
                command: visual.to_string(),
                source: EditorSource::Visual,
            };
        }
    }

    if let Some(editor) = editor {
        let editor = editor.trim();
        if !editor.is_empty() {
            return ResolvedEditorCommand {
                command: editor.to_string(),
                source: EditorSource::Editor,
            };
        }
    }

    ResolvedEditorCommand {
        command: "nano".to_string(),
        source: EditorSource::Fallback,
    }
}

pub fn command_on_path(command: &str) -> bool {
    let path = Path::new(command);
    if path.components().count() > 1 {
        return path.is_file();
    }

    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&paths).any(|dir| dir.join(command).is_file())
}

pub fn uses_vim_line_arg(command: &str) -> bool {
    let base = Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command);
    matches!(
        base,
        "vim" | "nvim" | "vi" | "gvim" | "nvim-qt" | "nano" | "micro"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_command_takes_precedence() {
        let settings = EditorSettings {
            configured_command: Some("nvim".to_string()),
            terminal: None,
        };
        let resolved = resolve_editor_command_with_env(
            &settings,
            Some("code".to_string()),
            Some("vim".to_string()),
        );
        assert_eq!(resolved.command, "nvim");
        assert_eq!(resolved.source, EditorSource::Config);
    }

    #[test]
    fn visual_env_beats_editor_env() {
        let settings = EditorSettings::default();
        let resolved = resolve_editor_command_with_env(
            &settings,
            Some("code".to_string()),
            Some("vim".to_string()),
        );
        assert_eq!(resolved.command, "code");
        assert_eq!(resolved.source, EditorSource::Visual);
    }

    #[test]
    fn editor_env_used_when_visual_missing() {
        let settings = EditorSettings::default();
        let resolved = resolve_editor_command_with_env(&settings, None, Some("vim".to_string()));
        assert_eq!(resolved.command, "vim");
        assert_eq!(resolved.source, EditorSource::Editor);
    }

    #[test]
    fn falls_back_to_nano_when_unset() {
        let settings = EditorSettings::default();
        let resolved = resolve_editor_command_with_env(&settings, None, None);
        assert_eq!(resolved.command, "nano");
        assert_eq!(resolved.source, EditorSource::Fallback);
    }

    #[test]
    fn command_on_path_finds_bash() {
        if Path::new("/bin/bash").exists() || Path::new("/usr/bin/bash").exists() {
            assert!(command_on_path("bash"));
        }
    }

    #[test]
    fn vim_family_detection() {
        assert!(uses_vim_line_arg("nvim"));
        assert!(uses_vim_line_arg("/usr/bin/vim"));
        assert!(!uses_vim_line_arg("code"));
    }
}
