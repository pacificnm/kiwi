use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DebounceTimer {
    deadline: Option<Instant>,
}

impl DebounceTimer {
    pub fn schedule(&mut self, debounce: Duration) {
        self.deadline = Some(Instant::now() + debounce);
    }

    pub fn clear(&mut self) {
        self.deadline = None;
    }

    #[must_use]
    pub fn poll_ready(&mut self) -> bool {
        let Some(deadline) = self.deadline else {
            return false;
        };

        if Instant::now() >= deadline {
            self.deadline = None;
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn remaining(&self) -> Option<Duration> {
        self.deadline.map(|deadline| {
            deadline
                .checked_duration_since(Instant::now())
                .unwrap_or(Duration::ZERO)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn poll_ready_waits_for_debounce_window() {
        let mut timer = DebounceTimer::default();
        timer.schedule(Duration::from_millis(50));
        assert!(!timer.poll_ready());
        thread::sleep(Duration::from_millis(60));
        assert!(timer.poll_ready());
        assert!(timer.remaining().is_none());
    }

    #[test]
    fn reschedule_extends_deadline() {
        let mut timer = DebounceTimer::default();
        timer.schedule(Duration::from_millis(80));
        thread::sleep(Duration::from_millis(40));
        timer.schedule(Duration::from_millis(80));
        thread::sleep(Duration::from_millis(50));
        assert!(!timer.poll_ready());
        thread::sleep(Duration::from_millis(40));
        assert!(timer.poll_ready());
    }
}
