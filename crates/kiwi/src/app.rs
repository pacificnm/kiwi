pub struct App {
    #[allow(dead_code)]
    cli: crate::cli::Cli,
}

impl App {
    #[must_use]
    pub fn new(cli: crate::cli::Cli) -> Self {
        Self { cli }
    }

    pub fn run(&self) {
        // Event loop will be implemented in later milestones.
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::Cli;

    use super::App;

    #[test]
    fn app_runs_without_panic() {
        let cli = Cli::parse_from(["kiwi"]);
        App::new(cli).run();
    }
}
