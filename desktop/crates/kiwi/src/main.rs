//! Kiwi IDE — AI-native Rust workspace.

mod agent;
mod chat;
mod cli;
mod fonts;
mod modules;
mod project;
mod theme;
mod workbench;

use std::env;

use nest_cli::CliApp;
use nest_gui::{GuiApp, GuiStartupOptions, StatusBarModule, ToastModule};
use nest_icon::IconModule;
use nest_logging::{LogBuffer, LoggingConfig};

use crate::cli::{AgentCommand, ChatCommand};
use crate::modules::{with_cli_modules, with_gui_modules};
use crate::project::{project_root_from_args, ProjectConfig};
use crate::theme::KiwiThemeModule;
use crate::workbench::KiwiWorkbench;

/// Retains recent log lines for the bottom-panel Logs tab.
const UI_LOG_BUFFER_CAPACITY: usize = 2_000;

fn main() {
    let args: Vec<String> = env::args().collect();
    if should_run_cli(&args) {
        with_cli_modules(
            CliApp::new("kiwi")
                .with_about("Kiwi IDE — AI-native Rust workspace")
                .with_logging(LoggingConfig::for_cli("kiwi"))
                .with_log_level_from_args(true)
                .async_command(ChatCommand)
                .async_command(AgentCommand),
        )
        .run();
    } else {
        let startup = GuiStartupOptions::from_args_iter(&args).ok();
        let config_path = startup.as_ref().and_then(|options| options.config_path.as_deref());
        let project = ProjectConfig::resolve(project_root_from_args(&args), config_path)
            .unwrap_or_else(|error| {
                eprintln!("kiwi: failed to resolve project root: {error}");
                std::process::exit(1);
            });

        with_gui_modules(
            GuiApp::new("kiwi")
                .with_task_runtime(true)
                .with_logging(
                    LoggingConfig::for_gui("kiwi")
                        .with_ui_buffer(LogBuffer::new(UI_LOG_BUFFER_CAPACITY)),
                )
                .module(KiwiThemeModule)
                .module(IconModule::new())
                .module(ToastModule::new())
                .module(StatusBarModule::new())
                .workbench(KiwiWorkbench::default()),
            &project,
        )
        .run();
    }
}

fn should_run_cli(args: &[String]) -> bool {
    args.iter()
        .skip(1)
        .any(|arg| arg == "chat" || arg == "agent")
}
