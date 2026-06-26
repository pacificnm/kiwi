use std::path::PathBuf;

use crate::events::{AppEvent, EventSender};

use super::loader::load_preview_file;

pub fn spawn_preview_load(path: PathBuf, max_size_bytes: u64, sender: EventSender) {
    std::thread::spawn(move || {
        let result = load_preview_file(&path, max_size_bytes);
        let _ = sender.send(AppEvent::PreviewLoaded { path, result });
    });
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use super::*;
    use crate::events::EventChannel;

    #[test]
    fn spawn_preview_load_enqueues_loaded_event() {
        let temp = std::env::temp_dir().join(format!("kiwi-preview-io-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let file = temp.join("one.txt");
        fs::write(&file, "hello\n").expect("write");

        let mut channel = EventChannel::new();
        spawn_preview_load(file.clone(), 1_048_576, channel.sender());

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut loaded = None;
        while std::time::Instant::now() < deadline {
            for event in channel.drain_coalesced() {
                if let AppEvent::PreviewLoaded { path, result } = event {
                    assert_eq!(path, file);
                    assert_eq!(result.lines, vec!["hello".to_string()]);
                    loaded = Some(());
                    break;
                }
            }
            if loaded.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        loaded.expect("preview loaded event");
        let _ = fs::remove_dir_all(temp);
    }
}
