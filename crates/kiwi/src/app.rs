pub struct App;

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) {
        // Event loop will be implemented in later milestones.
    }
}

#[cfg(test)]
mod tests {
    use super::App;

    #[test]
    fn app_runs_without_panic() {
        App::new().run();
    }
}
