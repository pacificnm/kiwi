//! Shared Nest modules for Kiwi CLI and GUI hosts.

use std::time::Duration;

use nest_ai_ollama::OllamaModule;
use nest_cli::CliApp;
use nest_file::FileModule;
use nest_gui::GuiApp;
use nest_http_client::{HttpClientModule, TimeoutConfig};

use crate::project::ProjectConfig;

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

/// Registers HTTP client and Ollama AI modules on a CLI host.
pub fn with_cli_modules(app: CliApp) -> CliApp {
    app.module(ollama_http_module())
        .module(OllamaModule::new())
}

/// Registers HTTP client, Ollama AI, and scoped file I/O on a GUI host.
pub fn with_gui_modules(app: GuiApp, project: &ProjectConfig) -> GuiApp {
    app.module(ollama_http_module())
        .module(OllamaModule::new())
        .module(FileModule::scoped(project.root.clone()))
}
