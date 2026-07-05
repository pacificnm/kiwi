//! Debounced filesystem watcher for explorer and git status refresh.

use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::project::path_is_ignored;

const DEBOUNCE: Duration = Duration::from_millis(750);

/// Watches the project root and signals when tracked sources may have changed.
pub struct ProjectWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<()>,
    pending: bool,
    debounce_until: Option<Instant>,
}

impl ProjectWatcher {
    /// Starts watching `root` recursively, ignoring build and VCS directories.
    pub fn new(root: &Path, ignored: Vec<String>) -> notify::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let watch_root = root.to_path_buf();
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<Event>| {
                let Ok(event) = result else {
                    return;
                };
                if event_affects_workspace(&event, &watch_root, &ignored) {
                    let _ = tx.send(());
                }
            },
            Config::default(),
        )?;
        watcher.watch(root, RecursiveMode::Recursive)?;
        Ok(Self {
            _watcher: watcher,
            rx,
            pending: false,
            debounce_until: None,
        })
    }

    /// Returns true once relevant filesystem activity has been quiet for the debounce window.
    pub fn poll(&mut self) -> bool {
        while self.rx.try_recv().is_ok() {
            self.pending = true;
            self.debounce_until = Some(Instant::now() + DEBOUNCE);
        }

        if !self.pending {
            return false;
        }

        let Some(deadline) = self.debounce_until else {
            return false;
        };

        if Instant::now() < deadline {
            return false;
        }

        self.pending = false;
        self.debounce_until = None;
        true
    }
}

fn event_affects_workspace(event: &Event, root: &Path, ignored: &[String]) -> bool {
    if matches!(event.kind, EventKind::Access(_)) {
        return false;
    }

    event
        .paths
        .iter()
        .any(|path| !path_is_ignored(path, root, ignored))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn ignores_events_under_target() {
        let root = PathBuf::from("/project");
        let ignored = vec!["target".into(), ".git".into()];
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Any,
            )),
            paths: vec![PathBuf::from("/project/target/debug/kiwi")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(!event_affects_workspace(&event, &root, &ignored));
    }

    #[test]
    fn accepts_events_for_source_files() {
        let root = PathBuf::from("/project");
        let ignored = vec!["target".into()];
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Any,
            )),
            paths: vec![PathBuf::from("/project/src/main.rs")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(event_affects_workspace(&event, &root, &ignored));
    }
}
