//! Debounced filesystem watcher for explorer refresh.

use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

/// Watches the project root and signals when the tree may need refreshing.
pub struct ProjectWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<()>,
    pending: bool,
    debounce_until: Option<Instant>,
}

impl ProjectWatcher {
    /// Starts watching `root` recursively for create/modify/delete events.
    pub fn new(root: &Path) -> notify::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| {
                if result.is_ok() {
                    let _ = tx.send(());
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;
        watcher.watch(root, RecursiveMode::Recursive)?;
        Ok(Self {
            _watcher: watcher,
            rx,
            pending: false,
            debounce_until: None,
        })
    }

    /// Returns true once filesystem activity has been quiet for the debounce window.
    pub fn poll(&mut self) -> bool {
        while self.rx.try_recv().is_ok() {
            self.pending = true;
            self.debounce_until = Some(Instant::now() + Duration::from_millis(750));
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

    /// Whether filesystem events are waiting for the debounce window to elapse.
    pub fn has_pending(&self) -> bool {
        self.pending
    }
}
