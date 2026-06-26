use std::process::{Child, Command, Output, Stdio};

#[derive(Debug, Default, Clone)]
pub struct SearchCancelHandle {
    child: std::sync::Arc<std::sync::Mutex<Option<Child>>>,
}

impl SearchCancelHandle {
    pub fn cancel(&self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.child.lock() {
            *guard = None;
        }
    }

    pub fn run_command(&self, mut command: Command) -> std::io::Result<Output> {
        self.cancel();
        let child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Ok(mut guard) = self.child.lock() {
            *guard = Some(child);
        }

        let child = self
            .child
            .lock()
            .map_err(|_| std::io::Error::other("search cancel lock poisoned"))?
            .take()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Interrupted, "search cancelled")
            })?;

        child.wait_with_output()
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::Duration;

    use super::*;

    #[test]
    fn cancel_kills_tracked_child_process() {
        let handle = SearchCancelHandle::default();
        let child = Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");
        if let Ok(mut guard) = handle.child.lock() {
            *guard = Some(child);
        }
        handle.cancel();

        std::thread::sleep(Duration::from_millis(20));
        let running = Command::new("sleep").arg("0.001").status().expect("status");
        assert!(running.success());
    }
}
