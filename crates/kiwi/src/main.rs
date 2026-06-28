mod agent;
mod ansi;
mod app;
mod bootstrap;
mod cli;
mod clipboard;
mod commands;
mod config;
mod diff;
mod editor;
mod file_tree;
mod git;
mod github;
mod layout;
mod navigation;
mod plugins;
mod preview;
mod search;
mod selection;
mod settings;
mod shell;
mod shutdown;
mod state;
mod terminal;
mod theme;
mod ui;
mod watcher;
mod workspace;

use cli::{Cli, CliCommand};

fn main() {
    let cli = Cli::parse_args();

    // Dispatch non-TUI subcommands before any terminal/bootstrap setup.
    if let Some(CliCommand::Plugin(ref sub)) = cli.command {
        std::process::exit(commands::plugin::run(sub));
    }

    let mut app = match bootstrap::init(&cli) {
        Ok(context) => app::App::new(context),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    };
    app.run();
}
