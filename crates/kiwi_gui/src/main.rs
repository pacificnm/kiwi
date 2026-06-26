mod app;
mod cli;

use cli::Cli;

fn main() {
    let cli = Cli::parse_args();
    if let Err(err) = run(&cli) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> eframe::Result<()> {
    let title = window_title(&cli.path);
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
        Box::new(|cc| Ok(Box::new(app::KiwiApp::new(cc)))),
    )
}

fn window_title(path: &std::path::Path) -> String {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or(".");
    format!("Kiwi — {name}")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::window_title;

    #[test]
    fn window_title_uses_directory_name() {
        assert_eq!(window_title(Path::new("/tmp/my-repo")), "Kiwi — my-repo");
    }

    #[test]
    fn window_title_falls_back_for_dot_path() {
        assert_eq!(window_title(Path::new(".")), "Kiwi — .");
    }
}
