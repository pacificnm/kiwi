use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::bootstrap::StartupContext;

pub struct App {
    context: StartupContext,
}

impl App {
    #[must_use]
    pub fn new(context: StartupContext) -> Self {
        Self { context }
    }

    pub fn run(&mut self) {
        loop {
            if event::poll(Duration::from_millis(100)).expect("poll keyboard events") {
                if let Event::Key(key) = event::read().expect("read keyboard event") {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    let quit = matches!(key.code, KeyCode::Char('q'))
                        || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL));

                    if quit {
                        break;
                    }
                }
            }
        }

        crate::shutdown::cleanup(&mut self.context);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::bootstrap::StartupContext;
    use crate::config::ResolvedConfig;
    use crate::terminal::TerminalGuard;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;
    use crate::theme::ThemePalette;

    use super::App;

    fn test_palette() -> ThemePalette {
        load_theme_with_capabilities(
            &ResolvedConfig::default().theme,
            TerminalCapabilities::TrueColor,
        )
        .expect("load default theme")
    }

    #[test]
    fn app_constructs_without_panic() {
        let context = StartupContext {
            repo_root: PathBuf::from("."),
            is_git_repo: false,
            config: ResolvedConfig::default(),
            theme: test_palette(),
            terminal: TerminalGuard::inactive(),
        };
        let _app = App::new(context);
    }
}
