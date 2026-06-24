mod app;
mod bootstrap;
mod cli;
mod config;
mod layout;
mod navigation;
mod repo;
mod shutdown;
mod terminal;
mod theme;
mod ui;

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
