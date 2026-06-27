mod app;
mod bootstrap;
mod chrome;
mod cli;
mod dock;
mod navigation_bridge;
mod runtime;
mod services;
mod theme;

use bootstrap::{init, window_title, GuiBootstrapContext};
use cli::Cli;
use runtime::GuiRuntime;

fn main() {
    let cli = Cli::parse_args();
    let context = match init(&cli) {
        Ok(context) => context,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = run_gui(context) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_gui(context: GuiBootstrapContext) -> eframe::Result<()> {
    let title = window_title(&context.repo_root);
    let (runtime, gui_snapshot) = GuiRuntime::build(context);
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        persist_window: true,
        ..Default::default()
    };

    eframe::run_native(
        "kiwi-gui",
        native_options,
        Box::new(move |cc| Ok(Box::new(app::KiwiApp::new(cc, runtime, gui_snapshot)))),
    )
}

#[cfg(test)]
mod tests {
    use crate::bootstrap::BootstrapError;

    #[test]
    fn bootstrap_error_display_includes_repo_message() {
        use kiwi_core::repo::RepoError;

        let err = BootstrapError::Repo(RepoError::NotFound(std::path::PathBuf::from("/missing")));
        assert!(err.to_string().contains("/missing"));
    }
}
