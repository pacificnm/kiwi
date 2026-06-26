use std::path::PathBuf;

use crate::events::{AppEvent, EventSender};

use super::loader::read_directory_children;

pub fn spawn_directory_load(path: PathBuf, sender: EventSender) {
    std::thread::spawn(move || {
        let result = read_directory_children(&path);
        let _ = sender.send(AppEvent::FileTreeChildrenLoaded {
            parent: path,
            children: result.children,
            error: result.error,
        });
    });
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use super::*;
    use crate::events::EventChannel;

    #[test]
    fn spawn_directory_load_enqueues_children_loaded_event() {
        let temp = std::env::temp_dir().join(format!("kiwi-file-tree-io-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        fs::write(temp.join("one.txt"), "1").expect("write");

        let mut channel = EventChannel::new();
        spawn_directory_load(temp.clone(), channel.sender());

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut loaded = None;
        while std::time::Instant::now() < deadline {
            for event in channel.drain_coalesced() {
                if let AppEvent::FileTreeChildrenLoaded {
                    parent,
                    children,
                    error,
                } = event
                {
                    assert_eq!(parent, temp);
                    assert!(error.is_none());
                    loaded = Some(children);
                    break;
                }
            }
            if loaded.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        let children = loaded.expect("children loaded event");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "one.txt");

        let _ = fs::remove_dir_all(temp);
    }
}
