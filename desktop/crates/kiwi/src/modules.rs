//! Shared Nest modules for Kiwi CLI and GUI hosts.

use nest_ai_ollama::OllamaModule;
use nest_cli::CliApp;
use nest_gui::GuiApp;
use nest_http_client::HttpClientModule;

/// Registers HTTP client and Ollama AI modules on a CLI host.
pub fn with_cli_modules(app: CliApp) -> CliApp {
    app.module(HttpClientModule::default())
        .module(OllamaModule::new())
}

/// Registers HTTP client and Ollama AI modules on a GUI host.
pub fn with_gui_modules(app: GuiApp) -> GuiApp {
    app.module(HttpClientModule::default())
        .module(OllamaModule::new())
}
