use std::path::PathBuf;
use std::time::{Duration, Instant};

pub const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoubleClickTarget {
    FileTree(PathBuf),
    SearchResult(usize),
    GitFile(usize),
    GitHubIssue(usize),
}

#[derive(Debug, Default)]
pub struct DoubleClickTracker {
    last: Option<(DoubleClickTarget, Instant)>,
}

impl DoubleClickTracker {
    /// Returns `true` when this click completes a double-click on `target`.
    pub fn register(&mut self, target: DoubleClickTarget) -> bool {
        let now = Instant::now();
        if let Some((prev_target, prev_at)) = &self.last {
            if *prev_target == target && now.duration_since(*prev_at) <= DOUBLE_CLICK_INTERVAL {
                self.last = None;
                return true;
            }
        }
        self.last = Some((target, now));
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_matching_click_within_interval_is_double_click() {
        let mut tracker = DoubleClickTracker::default();
        let target = DoubleClickTarget::FileTree(PathBuf::from("src/main.rs"));
        assert!(!tracker.register(target.clone()));
        assert!(tracker.register(target));
    }

    #[test]
    fn different_targets_are_not_double_click() {
        let mut tracker = DoubleClickTracker::default();
        assert!(!tracker.register(DoubleClickTarget::SearchResult(0)));
        assert!(!tracker.register(DoubleClickTarget::SearchResult(1)));
    }

    #[test]
    fn spaced_clicks_restart_sequence() {
        let mut tracker = DoubleClickTracker::default();
        let target = DoubleClickTarget::FileTree(PathBuf::from("a.rs"));
        assert!(!tracker.register(target.clone()));
        std::thread::sleep(DOUBLE_CLICK_INTERVAL + Duration::from_millis(20));
        assert!(!tracker.register(target));
    }
}
