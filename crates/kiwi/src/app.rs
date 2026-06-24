use crate::bootstrap::StartupContext;

pub struct App {
    #[allow(dead_code)]
    context: StartupContext,
}

impl App {
    #[must_use]
    pub fn new(context: StartupContext) -> Self {
        Self { context }
    }

    pub fn run(&self) {
        // Event loop will be implemented in later milestones.
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::bootstrap::StartupContext;
    use crate::config::ResolvedConfig;

    use super::App;

    #[test]
    fn app_runs_without_panic() {
        let context = StartupContext {
            repo_root: PathBuf::from("."),
            config: ResolvedConfig::default(),
        };
        App::new(context).run();
    }
}
