mod app;
mod bootstrap;
mod shutdown;

fn main() {
    bootstrap::init();
    app::App::new().run();
    shutdown::cleanup();
}
