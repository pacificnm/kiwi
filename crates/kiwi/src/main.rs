mod app;
mod bootstrap;
mod cli;
mod config;
mod shutdown;

use cli::Cli;

fn main() {
    let cli = Cli::parse_args();
    let context = match bootstrap::init(&cli) {
        Ok(context) => context,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    };
    app::App::new(context).run();
    shutdown::cleanup();
}
