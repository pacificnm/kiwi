mod app;
mod bootstrap;
mod cli;
mod shutdown;

use cli::Cli;

fn main() {
    let cli = Cli::parse_args();
    bootstrap::init(&cli);
    app::App::new(cli).run();
    shutdown::cleanup();
}
