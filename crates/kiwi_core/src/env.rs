//! Repository `.env` loading for subprocesses (agents, MCP servers).

use std::collections::HashMap;
use std::path::Path;

/// Environment variables forwarded to agent processes when unset in the parent.
pub const AGENT_ENV_KEYS: &[&str] = &[
    "DATABASE_URL",
    "OPENAI_API_KEY",
    "EMBED_BACKEND",
    "OPENAI_EMBED_MODEL",
    "OPENAI_EMBED_DIMENSIONS",
    "OLLAMA_URL",
    "OLLAMA_MODEL",
    "OLLAMA_EMBED_MODEL",
    "GITHUB_TOKEN",
    "GITHUB_REPO",
    "GITHUB_API_URL",
    "GIT_REPO_PATH",
    "GITEA_TOKEN",
    "GITEA_URL",
    "GITEA_REPO",
];

/// Parse `KEY=VALUE` lines from a `.env` file (no variable expansion).
pub fn parse_dotenv(content: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            out.insert(key.to_string(), value.to_string());
        }
    }
    out
}

/// Load selected keys from `repo_root/.env` without overwriting existing process env.
pub fn repo_env_vars(repo_root: &Path) -> HashMap<String, String> {
    let path = repo_root.join(".env");
    let Ok(content) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };

    let parsed = parse_dotenv(&content);
    let mut out = HashMap::new();
    for key in AGENT_ENV_KEYS {
        if std::env::var(key).is_ok() {
            continue;
        }
        if let Some(value) = parsed.get(*key) {
            out.insert(key.to_string(), value.clone());
        }
    }

    if !out.contains_key("EMBED_BACKEND")
        && out
            .get("OPENAI_API_KEY")
            .is_some_and(|value| !value.trim().is_empty())
    {
        out.insert("EMBED_BACKEND".to_string(), "openai".to_string());
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dotenv_strips_quotes_and_comments() {
        let vars = parse_dotenv(
            "# comment\nDATABASE_URL=postgres://x\nOPENAI_API_KEY=\"sk-test\"\nEMPTY=\n",
        );
        assert_eq!(vars.get("DATABASE_URL").map(String::as_str), Some("postgres://x"));
        assert_eq!(vars.get("OPENAI_API_KEY").map(String::as_str), Some("sk-test"));
        assert!(!vars.contains_key("EMPTY"));
    }
}
