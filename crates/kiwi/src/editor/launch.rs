use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::EditorSettings;

use super::classify::{editor_launch_mode, EditorLaunchMode};
use super::error::EditorLaunchError;
use super::resolve::{command_on_path, resolve_editor_command, uses_vim_line_arg, RESOLUTION_HINT};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLaunchResult {
    pub path: PathBuf,
    pub command: String,
    pub args: Vec<String>,
    pub mode: EditorLaunchMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedEditorLaunch {
    pub path: PathBuf,
    pub command: String,
    pub args: Vec<String>,
    pub mode: EditorLaunchMode,
}

pub fn prepare_editor_launch(
    repo_root: &Path,
    settings: &EditorSettings,
    path: &Path,
    line: Option<u32>,
) -> Result<PreparedEditorLaunch, EditorLaunchError> {
    let absolute_path = absolute_existing_path(repo_root, path)?;
    let resolved = resolve_editor_command(settings);

    if !command_on_path(&resolved.command) {
        return Err(EditorLaunchError::CommandNotFound {
            command: resolved.command.clone(),
            resolution_hint: RESOLUTION_HINT.to_string(),
        });
    }

    let mode = editor_launch_mode(&resolved.command, settings);
    let args = build_args(&resolved.command, &absolute_path, line);

    Ok(PreparedEditorLaunch {
        path: absolute_path,
        command: resolved.command,
        args,
        mode,
    })
}

pub fn launch_gui_editor(
    prepared: &PreparedEditorLaunch,
) -> Result<EditorLaunchResult, EditorLaunchError> {
    debug_assert_eq!(prepared.mode, EditorLaunchMode::Gui);

    let mut command = Command::new(&prepared.command);
    command
        .args(&prepared.args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = command
        .spawn()
        .map_err(|err| EditorLaunchError::SpawnFailed {
            message: err.to_string(),
        })?;

    std::thread::spawn(move || {
        let _ = child.wait();
    });

    Ok(prepared.to_result())
}

pub fn run_terminal_editor(
    repo_root: &Path,
    prepared: &PreparedEditorLaunch,
) -> Result<EditorLaunchResult, EditorLaunchError> {
    debug_assert_eq!(prepared.mode, EditorLaunchMode::Terminal);

    let mut command = Command::new(&prepared.command);
    command.args(&prepared.args).current_dir(repo_root);

    let status = command
        .status()
        .map_err(|err| EditorLaunchError::SpawnFailed {
            message: err.to_string(),
        })?;

    let _ = status;

    Ok(prepared.to_result())
}

impl PreparedEditorLaunch {
    fn to_result(&self) -> EditorLaunchResult {
        EditorLaunchResult {
            path: self.path.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            mode: self.mode,
        }
    }
}

fn absolute_existing_path(repo_root: &Path, path: &Path) -> Result<PathBuf, EditorLaunchError> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };

    let absolute = candidate
        .canonicalize()
        .unwrap_or_else(|_| candidate.clone());

    if !absolute.is_file() {
        return Err(EditorLaunchError::FileNotFound {
            path: candidate.display().to_string(),
        });
    }

    Ok(absolute)
}

fn build_args(command: &str, path: &Path, line: Option<u32>) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(line) = line.filter(|_| uses_vim_line_arg(command)) {
        args.push(format!("+{line}"));
    }
    args.push(path.display().to_string());
    args
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::config::EditorSettings;
    use crate::editor::classify::EditorLaunchMode;

    #[test]
    fn build_args_adds_vim_line_jump() {
        let args = build_args("nvim", Path::new("/tmp/a.rs"), Some(42));
        assert_eq!(args, vec!["+42".to_string(), "/tmp/a.rs".to_string()]);
    }

    #[test]
    fn build_args_skips_line_for_gui_editors() {
        let args = build_args("code", Path::new("/tmp/a.rs"), Some(42));
        assert_eq!(args, vec!["/tmp/a.rs".to_string()]);
    }

    #[test]
    fn rejects_missing_file() {
        let err = prepare_editor_launch(
            Path::new("/tmp"),
            &EditorSettings::default(),
            Path::new("missing-file-kiwi-test.txt"),
            None,
        )
        .expect_err("missing file");
        assert!(matches!(err, EditorLaunchError::FileNotFound { .. }));
    }

    #[test]
    fn prepare_honors_gui_override_for_terminal_command() {
        let temp = std::env::temp_dir().join(format!("kiwi-editor-mode-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let file = temp.join("sample.txt");
        fs::write(&file, "hello").expect("write");

        let command = if Path::new("/bin/true").exists() {
            "/bin/true"
        } else {
            "/usr/bin/true"
        };

        let settings = EditorSettings {
            configured_command: Some(command.to_string()),
            terminal: Some(false),
        };
        let prepared = prepare_editor_launch(&temp, &settings, &file, None).expect("prepare");
        assert_eq!(prepared.mode, EditorLaunchMode::Gui);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn prepare_selects_terminal_mode_for_forced_terminal_command() {
        let temp = std::env::temp_dir().join(format!("kiwi-editor-mode-n-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let file = temp.join("sample.txt");
        fs::write(&file, "hello").expect("write");

        let command = if Path::new("/bin/true").exists() {
            "/bin/true"
        } else {
            "/usr/bin/true"
        };

        let settings = EditorSettings {
            configured_command: Some(command.to_string()),
            terminal: Some(true),
        };
        let prepared = prepare_editor_launch(&temp, &settings, &file, None).expect("prepare");
        assert_eq!(prepared.mode, EditorLaunchMode::Terminal);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn launches_existing_file_with_configured_command() {
        let temp = std::env::temp_dir().join(format!("kiwi-editor-launch-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let file = temp.join("sample.txt");
        fs::write(&file, "hello").expect("write");

        let command = if Path::new("/bin/true").exists() {
            "/bin/true"
        } else if Path::new("/usr/bin/true").exists() {
            "/usr/bin/true"
        } else {
            return;
        };

        let settings = EditorSettings {
            configured_command: Some(command.to_string()),
            terminal: Some(false),
        };
        let prepared = prepare_editor_launch(&temp, &settings, &file, None).expect("prepare");
        let result = launch_gui_editor(&prepared).expect("launch");
        assert_eq!(result.command, command);

        let _ = fs::remove_dir_all(temp);
    }
}
