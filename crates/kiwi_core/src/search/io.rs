use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::events::{AppEvent, EventSender};

use super::cancel::SearchCancelHandle;
use super::content::{search_content, ContentSearchError};
use super::file::search_files;
use super::SearchMode;

#[derive(Debug, Clone)]
pub struct SearchJob {
    pub mode: SearchMode,
    pub query: String,
    pub generation: u64,
    pub repo_root: PathBuf,
    pub rg_command: String,
}

pub fn spawn_search(
    job: SearchJob,
    sender: EventSender,
    live_generation: Arc<AtomicU64>,
    cancel: SearchCancelHandle,
) {
    std::thread::spawn(move || {
        if live_generation.load(Ordering::Relaxed) != job.generation {
            return;
        }

        let event = match job.mode {
            SearchMode::Files => {
                let (results, truncated) = search_files(&job.repo_root, &job.query);
                if live_generation.load(Ordering::Relaxed) != job.generation {
                    return;
                }
                AppEvent::SearchCompleted {
                    generation: job.generation,
                    results,
                    truncated,
                    error: None,
                }
            }
            SearchMode::Content => {
                match search_content(&job.repo_root, &job.rg_command, &job.query, &cancel) {
                    Ok((results, truncated)) => {
                        if live_generation.load(Ordering::Relaxed) != job.generation {
                            return;
                        }
                        AppEvent::SearchCompleted {
                            generation: job.generation,
                            results,
                            truncated,
                            error: None,
                        }
                    }
                    Err(ContentSearchError::NotFound) => AppEvent::SearchCompleted {
                        generation: job.generation,
                        results: Vec::new(),
                        truncated: false,
                        error: Some(
                            "ripgrep (`rg`) and grep not found. Install ripgrep or set [search].command."
                                .to_string(),
                        ),
                    },
                    Err(ContentSearchError::Failed(message)) => AppEvent::SearchCompleted {
                        generation: job.generation,
                        results: Vec::new(),
                        truncated: false,
                        error: Some(message),
                    },
                }
            }
        };

        let _ = sender.send(event);
    });
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use super::SearchMode;
    use super::*;
    use crate::events::EventChannel;

    #[test]
    fn spawn_search_enqueues_completed_event() {
        let temp = std::env::temp_dir().join(format!("kiwi-search-io-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        fs::write(temp.join("hello.txt"), "hello\n").expect("write");

        let mut channel = EventChannel::new();
        let generation = Arc::new(AtomicU64::new(1));
        spawn_search(
            SearchJob {
                mode: SearchMode::Files,
                query: "hel".to_string(),
                generation: 1,
                repo_root: temp.clone(),
                rg_command: "rg".to_string(),
            },
            channel.sender(),
            generation,
            SearchCancelHandle::default(),
        );

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut completed = false;
        while std::time::Instant::now() < deadline {
            for event in channel.drain_coalesced() {
                if let AppEvent::SearchCompleted {
                    generation,
                    results,
                    error,
                    ..
                } = event
                {
                    assert_eq!(generation, 1);
                    assert!(error.is_none());
                    assert_eq!(results.len(), 1);
                    completed = true;
                    break;
                }
            }
            if completed {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        assert!(completed);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn spawn_search_drops_stale_generation_results() {
        let temp = std::env::temp_dir().join(format!("kiwi-search-stale-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        fs::write(temp.join("one.txt"), "one\n").expect("write");

        let mut channel = EventChannel::new();
        let generation = Arc::new(AtomicU64::new(2));
        spawn_search(
            SearchJob {
                mode: SearchMode::Files,
                query: "one".to_string(),
                generation: 1,
                repo_root: temp.clone(),
                rg_command: "rg".to_string(),
            },
            channel.sender(),
            generation,
            SearchCancelHandle::default(),
        );

        std::thread::sleep(Duration::from_millis(100));
        let events = channel.drain_coalesced();
        assert!(events.is_empty());

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn spawn_search_enqueues_content_results() {
        if std::process::Command::new("rg")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }

        let temp =
            std::env::temp_dir().join(format!("kiwi-search-content-io-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        fs::write(temp.join("sample.txt"), "searchable kiwi token\n").expect("write");

        let mut channel = EventChannel::new();
        let generation = Arc::new(AtomicU64::new(1));
        spawn_search(
            SearchJob {
                mode: SearchMode::Content,
                query: "kiwi token".to_string(),
                generation: 1,
                repo_root: temp.clone(),
                rg_command: "rg".to_string(),
            },
            channel.sender(),
            generation,
            SearchCancelHandle::default(),
        );

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut completed = false;
        while std::time::Instant::now() < deadline {
            for event in channel.drain_coalesced() {
                if let AppEvent::SearchCompleted {
                    generation,
                    results,
                    error,
                    ..
                } = event
                {
                    assert_eq!(generation, 1);
                    assert!(error.is_none());
                    assert_eq!(results.len(), 1);
                    assert!(results[0].preview.contains("kiwi token"));
                    completed = true;
                    break;
                }
            }
            if completed {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        assert!(completed);
        let _ = fs::remove_dir_all(temp);
    }
}
