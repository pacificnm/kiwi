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
mod layout;
mod navigation;
mod preview;
mod repo;
mod search;
mod selection;
mod shell;
mod shutdown;
mod state;
mod terminal;
mod theme;
mod ui;
mod watcher;
mod workspace;

use cli::Cli;

fn main() {
    let cli = Cli::parse_args();
    let mut app = match bootstrap::init(&cli) {
        Ok(context) => app::App::new(context),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    };
    app.run();
}
