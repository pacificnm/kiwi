//! Kiwi IDE — AI-native Rust workspace (CLI).

mod agent;
mod chat;
mod cli;
mod modules;
mod project;

use nest_cli::CliApp;
use nest_logging::LoggingConfig;

use crate::cli::{AgentCommand, ChatCommand, FileCommand};
use crate::modules::with_cli_modules;

fn main() {
    with_cli_modules(
        CliApp::new("kiwi")
            .with_about("Kiwi IDE — AI-native Rust workspace")
            .with_logging(LoggingConfig::for_cli("kiwi"))
            .with_log_level_from_args(true)
            .async_command(ChatCommand)
            .async_command(AgentCommand)
            .command(FileCommand),
    )
    .run();
}
