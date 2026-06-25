use std::io::Read;
use std::thread::{self, JoinHandle};

use crate::state::{AppEvent, EventSender};

pub fn spawn_output_reader(
    mut reader: Box<dyn Read + Send>,
    sender: EventSender,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count)
                    if sender
                        .send(AppEvent::ShellOutput(buffer[..count].to_vec()))
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
