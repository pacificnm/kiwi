//! Kiwi IDE — AI-native Rust workspace.

mod agent;
mod chat;
mod cli;
mod fonts;
mod modules;
mod theme;
mod workbench;

use std::env;

use nest_cli::CliApp;
use nest_gui::{GuiApp, StatusBarModule, ToastModule};
use nest_icon::IconModule;
use nest_logging::LoggingConfig;

use crate::cli::{AgentCommand, ChatCommand};
use crate::modules::{with_cli_modules, with_gui_modules};
use crate::theme::KiwiThemeModule;
use crate::workbench::KiwiWorkbench;

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
        with_gui_modules(
            GuiApp::new("kiwi")
                .with_task_runtime(true)
                .module(KiwiThemeModule)
                .module(IconModule::new())
                .module(ToastModule::new())
                .module(StatusBarModule::new())
                .workbench(KiwiWorkbench::default()),
        )
        .run();
    }
}

fn should_run_cli(args: &[String]) -> bool {
    args.iter()
        .skip(1)
        .any(|arg| arg == "chat" || arg == "agent")
}
