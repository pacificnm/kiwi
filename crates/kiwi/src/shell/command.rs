use std::path::Path;

use crate::config::ShellSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellLaunchSpec {
    pub command: String,
    pub args: Vec<String>,
    pub shell_name: String,
}

pub fn shell_launch_spec(settings: &ShellSettings) -> ShellLaunchSpec {
    let command = settings.command.clone();
    ShellLaunchSpec {
        shell_name: shell_display_name(&command),
        args: settings.args.clone(),
        command,
    }
}

pub fn shell_display_name(command: &str) -> String {
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_display_name_uses_basename() {
        assert_eq!(shell_display_name("/bin/zsh"), "zsh");
        assert_eq!(shell_display_name("bash"), "bash");
    }

    #[test]
    fn launch_spec_uses_configured_command_and_args() {
        let settings = ShellSettings {
            command: "/usr/bin/fish".to_string(),
            args: vec!["-l".to_string()],
        };
        let spec = shell_launch_spec(&settings);
        assert_eq!(
            spec,
            ShellLaunchSpec {
                command: "/usr/bin/fish".to_string(),
                args: vec!["-l".to_string()],
                shell_name: "fish".to_string(),
            }
        );
    }
}
