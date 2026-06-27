use std::io::Read;
use std::thread::{self, JoinHandle};

use crate::events::{AppEvent, EventSender};

use super::AgentId;

pub struct AgentOutputReader {
    handle: Option<JoinHandle<()>>,
}

impl AgentOutputReader {
    pub fn spawn(reader: Box<dyn Read + Send>, agent_id: AgentId, sender: EventSender) -> Self {
        Self {
            handle: Some(spawn_output_reader(reader, agent_id, sender)),
        }
    }

    pub fn abandon(mut self) {
        if let Some(handle) = self.handle.take() {
            std::mem::forget(handle);
        }
    }
}

fn spawn_output_reader(
    mut reader: Box<dyn Read + Send>,
    agent_id: AgentId,
    sender: EventSender,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count)
                    if sender
                        .send(AppEvent::AgentOutput {
                            agent_id,
                            data: buffer[..count].to_vec(),
                        })
                        .is_err() =>
                {
                    break;
                }
                Ok(_) => {}
                Err(e) => match e.kind() {
                    // Signal interruption — retry the read.
                    std::io::ErrorKind::Interrupted => continue,
                    // Expected PTY-close signals — runtime detects exit via poll_exits.
                    std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset => break,
                    // Unexpected mid-session error (e.g. resource exhaustion). Surface it
                    // so the agent pane shows a failure rather than silently going dead.
                    _ => {
                        let _ = sender.send(AppEvent::AgentExited { agent_id, code: 1 });
                        break;
                    }
                },
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn reader_thread_exits_when_write_side_closes() {
        use std::os::unix::net::UnixStream;

        let (reader, writer) = UnixStream::pair().expect("socket pair");
        let channel = crate::events::EventChannel::new();
        let handle = spawn_output_reader(Box::new(reader), AgentId::FIRST, channel.sender());

        drop(writer);
        handle.join().expect("reader thread should exit after EOF");
    }

    /// A reader that returns one `Interrupted` error then EOF.
    struct InterruptThenEof {
        interrupted_once: bool,
    }

    impl Read for InterruptThenEof {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            if !self.interrupted_once {
                self.interrupted_once = true;
                Err(std::io::Error::from(std::io::ErrorKind::Interrupted))
            } else {
                Ok(0)
            }
        }
    }

    #[test]
    fn interrupted_error_retries_and_exits_cleanly_on_eof() {
        let mut channel = crate::events::EventChannel::new();
        let handle = spawn_output_reader(
            Box::new(InterruptThenEof { interrupted_once: false }),
            AgentId::FIRST,
            channel.sender(),
        );
        handle.join().expect("thread should exit after retry");
        // No AgentExited event should have been sent — interrupted then EOF is clean.
        let events = channel.drain_coalesced();
        assert!(
            !events.iter().any(|e| matches!(e, AppEvent::AgentExited { .. })),
            "Interrupted+EOF should not produce AgentExited"
        );
    }

    /// A reader that immediately returns an unexpected I/O error.
    struct UnexpectedError;

    impl Read for UnexpectedError {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
        }
    }

    #[test]
    fn unexpected_io_error_sends_agent_exited_with_code_1() {
        let mut channel = crate::events::EventChannel::new();
        let handle = spawn_output_reader(
            Box::new(UnexpectedError),
            AgentId::FIRST,
            channel.sender(),
        );
        handle.join().expect("thread should exit");
        let events = channel.drain_coalesced();
        assert!(
            events.iter().any(|e| matches!(e, AppEvent::AgentExited { code: 1, .. })),
            "Unexpected I/O error should send AgentExited with code 1"
        );
    }
}
