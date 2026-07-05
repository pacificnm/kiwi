//! GitHub personal access token load from Kiwi config (fallback after `gh` CLI).

use nest_config::ConfigService;
use serde::Deserialize;

/// `[github]` section in `config.toml`.
#[derive(Debug, Clone, Deserialize)]
struct GitHubSection {
    token: Option<String>,
}

/// Loads a saved GitHub token from the config file.
pub fn load_token_from_config(service: &ConfigService) -> Option<String> {
    service
        .section::<GitHubSection>("github")
        .ok()
        .and_then(|section| section.token)
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}
