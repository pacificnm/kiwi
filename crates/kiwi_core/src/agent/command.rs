use std::path::Path;

use crate::config::AgentSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentLaunchSpec {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub agent_name: String,
}

pub fn agent_launch_spec(settings: &AgentSettings) -> AgentLaunchSpec {
    let command = settings.command.clone();
    AgentLaunchSpec {
        agent_name: agent_display_name(&command),
        args: settings.args.clone(),
        env: settings
            .env
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
        command,
    }
}

pub fn agent_display_name(command: &str) -> String {
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn agent_display_name_uses_basename() {
        assert_eq!(agent_display_name("/usr/bin/agent"), "agent");
        assert_eq!(agent_display_name("agent"), "agent");
    }

    #[test]
    fn launch_spec_uses_configured_command_args_and_env() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        let settings = AgentSettings {
            command: "agent".to_string(),
            args: vec!["--help".to_string()],
            env,
            ..AgentSettings::default()
        };
        let spec = agent_launch_spec(&settings);
        assert_eq!(spec.command, "agent");
        assert_eq!(spec.args, vec!["--help".to_string()]);
        assert_eq!(spec.env, vec![("FOO".to_string(), "bar".to_string())]);
        assert_eq!(spec.agent_name, "agent");
    }
}
