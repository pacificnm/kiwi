//! Event channel for dispatching [`AppEvent`]s from background tasks into the main loop.

#![cfg_attr(not(test), allow(dead_code))]

use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};

use super::event::AppEvent;

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

        loop {
            match self.receiver.try_recv() {
                Ok(AppEvent::GitRefreshRequested) => {
                    pending_git_refresh = true;
                }
                Ok(event) => {
                    flush_git_refresh(&mut events, &mut pending_git_refresh);
                    events.push(event);
                }
                Err(TryRecvError::Empty) => {
                    flush_git_refresh(&mut events, &mut pending_git_refresh);
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    flush_git_refresh(&mut events, &mut pending_git_refresh);
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

fn flush_git_refresh(events: &mut Vec<AppEvent>, pending: &mut bool) {
    if *pending {
        events.push(AppEvent::GitRefreshRequested);
        *pending = false;
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
    fn channel_capacity_matches_spec() {
        assert_eq!(EVENT_CHANNEL_CAPACITY, 1024);
    }
}
