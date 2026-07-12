#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod accounts;
mod agent;
mod agent_config;
mod commands;
mod config_host;
mod doc_sources;
mod docs;
mod git;
mod github;
mod kiwi_config;
mod mcp;
mod new_app;
mod ollama;
mod problems;
mod swift;
mod terminal;
mod workspace;

use std::path::PathBuf;

use crate::config_host::{is_dev_layout, resolve_config_path};
use nest_logging::{LogBuffer, LoggingConfig};
use nest_tauri::TauriApp;
use nest_theme::ThemeModule;

use crate::agent::AgentPty;
use crate::agent_config::AgentConfig;
use crate::problems::ProblemsState;
use crate::swift::SwiftDb;
use crate::terminal::TerminalManager;
use crate::workspace::Workspace;

/// Writable log directory for the Tauri host.
fn resolve_log_dir() -> PathBuf {
    if is_dev_layout() {
        return PathBuf::from("../desktop/logs");
    }
    dirs::data_dir()
        .map(|dir| dir.join("kiwi").join("logs"))
        .unwrap_or_else(|| std::env::temp_dir().join("kiwi").join("logs"))
}

fn main() {
    let config_path = resolve_config_path();
    if let Some(path) = &config_path {
        eprintln!("kiwi: using config {}", path.display());
    }

    let workspace = match Workspace::resolve(config_path.as_deref()) {
        Ok(workspace) => workspace,
        Err(error) => {
            eprintln!("kiwi: failed to resolve workspace root: {error}");
            std::process::exit(1);
        }
    };

    let log_buffer = LogBuffer::new(2_000);

    let mut app = TauriApp::new("kiwi")
        .with_logging(
            LoggingConfig::for_tauri("kiwi")
                .with_file(resolve_log_dir())
                .with_ui_buffer(log_buffer),
        )
        .module(ThemeModule::default().with_active("cursor-dark"));

    if let Some(path) = &config_path {
        app = app.with_config_path(path.clone());
    }

    app.with_builder(move |builder| {
        let agent_config = AgentConfig::load(config_path.clone());
        let swift_db = SwiftDb::new(config_path);
        builder
            .manage(AgentPty::default())
            .manage(TerminalManager::default())
            .manage(ProblemsState::default())
            .manage(swift_db)
            .manage(agent_config)
            .manage(workspace)
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_clipboard_manager::init())
            .plugin(commands::kiwi_plugin())
            .plugin(new_app::new_app_plugin())
    })
    .run(tauri::generate_context!());
}
