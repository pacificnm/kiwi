use std::io::Read;
use std::thread::{self, JoinHandle};

use crate::state::{AppEvent, EventSender};

pub struct AgentOutputReader {
    handle: Option<JoinHandle<()>>,
}

impl AgentOutputReader {
    pub fn spawn(reader: Box<dyn Read + Send>, sender: EventSender) -> Self {
        Self {
            handle: Some(spawn_output_reader(reader, sender)),
        }
    }

    pub fn abandon(mut self) {
        if let Some(handle) = self.handle.take() {
            std::mem::forget(handle);
        }
    }
}

fn spawn_output_reader(mut reader: Box<dyn Read + Send>, sender: EventSender) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count)
                    if sender
                        .send(AppEvent::AgentOutput(buffer[..count].to_vec()))
                        .is_err() =>
                {
                    break;
                }
                Ok(_) => {}
                Err(_) => break,
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
        let channel = crate::state::EventChannel::new();
        let handle = spawn_output_reader(Box::new(reader), channel.sender());

        drop(writer);
        handle.join().expect("reader thread should exit after EOF");
    }
}
