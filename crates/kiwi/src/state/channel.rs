//! Event channel for dispatching [`AppEvent`]s from background tasks into the main loop.

#![cfg_attr(not(test), allow(dead_code))]

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};

use super::event::AppEvent;
use crate::watcher::coalesce_paths;

pub const EVENT_CHANNEL_CAPACITY: usize = 1024;

#[derive(Debug)]
pub struct EventChannel {
    sender: SyncSender<AppEvent>,
    receiver: Receiver<AppEvent>,
}

#[derive(Debug, Clone)]
pub struct EventSender {
    inner: SyncSender<AppEvent>,
}

impl EventSender {
    #[allow(clippy::result_large_err)]
    pub fn send(&self, event: AppEvent) -> Result<(), mpsc::SendError<AppEvent>> {
        self.inner.send(event)
    }
}

impl EventChannel {
    #[must_use]
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::sync_channel(EVENT_CHANNEL_CAPACITY);
        Self { sender, receiver }
    }

    #[must_use]
    pub fn sender(&self) -> EventSender {
        EventSender {
            inner: self.sender.clone(),
        }
    }

    pub fn drain_coalesced(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        let mut pending_git_refresh = false;
        let mut pending_fs_paths: HashSet<PathBuf> = HashSet::new();

        loop {
            match self.receiver.try_recv() {
                Ok(AppEvent::GitRefreshRequested) => {
                    pending_git_refresh = true;
                }
                Ok(AppEvent::FsChanged { paths }) => {
                    pending_fs_paths.extend(paths);
                }
                Ok(event) => {
                    flush_pending(&mut events, &mut pending_git_refresh, &mut pending_fs_paths);
                    events.push(event);
                }
                Err(TryRecvError::Empty) => {
                    flush_pending(&mut events, &mut pending_git_refresh, &mut pending_fs_paths);
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    flush_pending(&mut events, &mut pending_git_refresh, &mut pending_fs_paths);
                    break;
                }
            }
        }

        events
    }
}

impl Default for EventChannel {
    fn default() -> Self {
        Self::new()
    }
}

fn flush_pending(
    events: &mut Vec<AppEvent>,
    pending_git_refresh: &mut bool,
    pending_fs_paths: &mut HashSet<PathBuf>,
) {
    if *pending_git_refresh {
        events.push(AppEvent::GitRefreshRequested);
        *pending_git_refresh = false;
    }
    if !pending_fs_paths.is_empty() {
        let paths = coalesce_paths(pending_fs_paths.drain());
        events.push(AppEvent::FsChanged { paths });
    }
}

#[cfg(test)]
mod tests {
    use crate::navigation::{LeftNavTab, NavCommand};
    use crate::state::event::AppCommand;

    use super::*;

    #[test]
    fn coalesces_duplicate_git_refresh_events() {
        let mut channel = EventChannel::new();
        let sender = channel.sender();

        sender.send(AppEvent::GitRefreshRequested).expect("send");
        sender.send(AppEvent::GitRefreshRequested).expect("send");
        sender
            .send(AppEvent::Command(AppCommand::Navigation(
                NavCommand::SelectLeftTab(LeftNavTab::Git),
            )))
            .expect("send");

        let drained = channel.drain_coalesced();
        let git_refreshes = drained
            .iter()
            .filter(|event| matches!(event, AppEvent::GitRefreshRequested))
            .count();

        assert_eq!(git_refreshes, 1);
        assert_eq!(drained.len(), 2);
    }

    #[test]
    fn coalesces_fs_changed_paths() {
        let mut channel = EventChannel::new();
        let sender = channel.sender();

        sender
            .send(AppEvent::FsChanged {
                paths: vec![PathBuf::from("/repo/a.rs")],
            })
            .expect("send");
        sender
            .send(AppEvent::FsChanged {
                paths: vec![PathBuf::from("/repo/b.rs"), PathBuf::from("/repo/a.rs")],
            })
            .expect("send");

        let drained = channel.drain_coalesced();
        assert_eq!(drained.len(), 1);
        let AppEvent::FsChanged { paths } = &drained[0] else {
            panic!("expected FsChanged");
        };
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&PathBuf::from("/repo/a.rs")));
        assert!(paths.contains(&PathBuf::from("/repo/b.rs")));
    }

    #[test]
    fn coalesces_fifty_fs_changed_batches() {
        let mut channel = EventChannel::new();
        let sender = channel.sender();

        for i in 0..50 {
            sender
                .send(AppEvent::FsChanged {
                    paths: vec![PathBuf::from(format!("/repo/file{i}.rs"))],
                })
                .expect("send");
            sender
                .send(AppEvent::FsChanged {
                    paths: vec![PathBuf::from(format!("/repo/file{i}.rs"))],
                })
                .expect("send");
        }

        let drained = channel.drain_coalesced();
        assert_eq!(drained.len(), 1);
        let AppEvent::FsChanged { paths } = &drained[0] else {
            panic!("expected FsChanged");
        };
        assert_eq!(paths.len(), 50);
    }

    #[test]
    fn channel_capacity_matches_spec() {
        assert_eq!(EVENT_CHANNEL_CAPACITY, 1024);
    }
}
