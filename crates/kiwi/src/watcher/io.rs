use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use crate::state::{AppEvent, EventSender};

use super::debounce::PathDebouncer;
use super::paths::{should_emit_fs_changed_event, should_ignore_watch_path, DEFAULT_DEBOUNCE_MS};

pub struct RepoWatcher {
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl RepoWatcher {
    pub fn spawn(repo_root: PathBuf, sender: EventSender) -> Result<Self, String> {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = Arc::clone(&shutdown);
        let (raw_tx, raw_rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |result: Result<notify::Event, notify::Error>| match result {
                Ok(event) => {
                    if !should_emit_fs_changed_event(&event.kind) {
                        return;
                    }
                    for path in event.paths {
                        if !should_ignore_watch_path(&path) {
                            let _ = raw_tx.send(path);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("file watcher warning: {err}");
                }
            },
            Config::default(),
        )
        .map_err(|err| err.to_string())?;

        watcher
            .watch(&repo_root, RecursiveMode::Recursive)
            .map_err(|err| err.to_string())?;

        let handle = thread::spawn(move || {
            let _watcher = watcher;
            run_debounce_loop(raw_rx, sender, shutdown_flag);
        });

        Ok(Self {
            shutdown,
            handle: Some(handle),
        })
    }
}

impl Drop for RepoWatcher {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_debounce_loop(
    raw_rx: mpsc::Receiver<PathBuf>,
    sender: EventSender,
    shutdown: Arc<AtomicBool>,
) {
    let mut debouncer = PathDebouncer::new(Duration::from_millis(DEFAULT_DEBOUNCE_MS));

    while !shutdown.load(Ordering::Relaxed) {
        while let Ok(path) = raw_rx.try_recv() {
            debouncer.push(path);
        }

        if let Some(paths) = debouncer.poll_ready() {
            let _ = sender.send(AppEvent::FsChanged { paths });
        }

        thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use super::*;
    use crate::state::EventChannel;

    #[test]
    fn watcher_emits_fs_changed_for_nested_file() {
        let temp = std::env::temp_dir().join(format!("kiwi-watcher-nested-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("src")).expect("mkdir");
        let file = temp.join("src/nested.rs");

        let mut channel = EventChannel::new();
        let watcher = RepoWatcher::spawn(temp.clone(), channel.sender()).expect("spawn watcher");

        fs::write(&file, "nested").expect("write");
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut changed = None;
        while std::time::Instant::now() < deadline {
            for event in channel.drain_coalesced() {
                if let AppEvent::FsChanged { paths } = event {
                    if paths.iter().any(|path| path.ends_with("nested.rs")) {
                        changed = Some(());
                        break;
                    }
                }
            }
            if changed.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        drop(watcher);
        changed.expect("nested fs changed event");
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn watcher_emits_fs_changed_for_git_head() {
        let temp = std::env::temp_dir().join(format!("kiwi-watcher-head-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let status = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&temp)
            .status()
            .expect("git init");
        assert!(status.success(), "git init failed");

        let head = temp.join(".git/HEAD");
        let mut channel = EventChannel::new();
        let watcher = RepoWatcher::spawn(temp.clone(), channel.sender()).expect("spawn watcher");

        fs::write(&head, "ref: refs/heads/main\n").expect("write head");
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut changed = None;
        while std::time::Instant::now() < deadline {
            for event in channel.drain_coalesced() {
                if let AppEvent::FsChanged { paths } = event {
                    if paths.iter().any(|path| path.ends_with(".git/HEAD")) {
                        changed = Some(());
                        break;
                    }
                }
            }
            if changed.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        drop(watcher);
        changed.expect("git head fs changed event");
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn watcher_emits_fs_changed_after_file_write() {
        let temp = std::env::temp_dir().join(format!("kiwi-watcher-io-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let file = temp.join("watch-me.txt");

        let mut channel = EventChannel::new();
        let watcher = RepoWatcher::spawn(temp.clone(), channel.sender()).expect("spawn watcher");

        fs::write(&file, "one").expect("write");
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut changed = None;
        while std::time::Instant::now() < deadline {
            for event in channel.drain_coalesced() {
                if let AppEvent::FsChanged { paths } = event {
                    if paths.iter().any(|path| path.ends_with("watch-me.txt")) {
                        changed = Some(());
                        break;
                    }
                }
            }
            if changed.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        drop(watcher);
        changed.expect("fs changed event");
        let _ = fs::remove_dir_all(temp);
    }
}
