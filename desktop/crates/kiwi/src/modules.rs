//! Shared Nest modules for Kiwi CLI.

use std::env;
use std::time::Duration;

use nest_ai_ollama::OllamaModule;
use nest_cli::CliApp;
use nest_file::FileModule;
use nest_http_client::{HttpClientModule, TimeoutConfig};

use crate::project::{project_root_from_args, ProjectConfig};

/// LLM inference can block while Ollama loads large models into VRAM.
const OLLAMA_HTTP_TIMEOUT: TimeoutConfig = TimeoutConfig {
    connect: Duration::from_secs(15),
    request: Duration::from_secs(300),
};

fn ollama_http_module() -> HttpClientModule {
    HttpClientModule::default()
        .with_user_agent("kiwi/0.1")
        .with_timeout(OLLAMA_HTTP_TIMEOUT)
}

fn cli_file_module() -> FileModule {
    let args: Vec<String> = env::args().collect();
    let project = ProjectConfig::resolve(project_root_from_args(&args), None)
        .or_else(|_| ProjectConfig::from_root(env::current_dir().unwrap_or_default()));
    match project {
        Ok(project) => FileModule::scoped(project.root),
        Err(_) => FileModule::new(),
    }
}

/// Registers HTTP client and Ollama AI modules on a CLI host.
pub fn with_cli_modules(app: CliApp) -> CliApp {
    app.module(ollama_http_module())
        .module(OllamaModule::new())
        .module(cli_file_module())
}
