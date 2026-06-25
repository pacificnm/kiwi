use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::paths::should_ignore_watch_path;

#[derive(Debug, Clone)]
pub struct PathDebouncer {
    debounce: Duration,
    pending: HashSet<PathBuf>,
    deadline: Option<Instant>,
}

impl PathDebouncer {
    pub fn new(debounce: Duration) -> Self {
        Self {
            debounce,
            pending: HashSet::new(),
            deadline: None,
        }
    }

    pub fn push(&mut self, path: PathBuf) {
        if should_ignore_watch_path(&path) {
            return;
        }
        self.pending.insert(path);
        self.deadline = Some(Instant::now() + self.debounce);
    }

    #[must_use]
    pub fn poll_ready(&mut self) -> Option<Vec<PathBuf>> {
        if self.pending.is_empty() {
            return None;
        }

        let deadline = self.deadline?;
        if Instant::now() < deadline {
            return None;
        }

        let paths: Vec<PathBuf> = self.pending.drain().collect();
        self.deadline = None;
        Some(paths)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn coalesces_paths_after_debounce() {
        let mut debouncer = PathDebouncer::new(Duration::from_millis(40));
        debouncer.push(PathBuf::from("/repo/a.rs"));
        debouncer.push(PathBuf::from("/repo/b.rs"));
        assert!(debouncer.poll_ready().is_none());
        thread::sleep(Duration::from_millis(50));
        let paths = debouncer.poll_ready().expect("ready");
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn allows_git_metadata_paths_in_debouncer() {
        let mut debouncer = PathDebouncer::new(Duration::from_millis(1));
        debouncer.push(PathBuf::from("/repo/.git/index"));
        debouncer.push(PathBuf::from("/repo/.git/HEAD"));
        thread::sleep(Duration::from_millis(5));
        let paths = debouncer.poll_ready().expect("ready");
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn ignores_other_git_paths() {
        let mut debouncer = PathDebouncer::new(Duration::from_millis(1));
        debouncer.push(PathBuf::from("/repo/.git/objects/pack/foo"));
        thread::sleep(Duration::from_millis(5));
        assert!(debouncer.poll_ready().is_none());
    }
}
