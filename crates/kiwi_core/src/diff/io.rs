use std::path::PathBuf;

use crate::events::{AppEvent, EventSender};

use super::generate::load_file_diff;
use crate::diff::DiffSource;

pub fn spawn_file_diff_load(
    repo_root: PathBuf,
    path: String,
    source: DiffSource,
    context_lines: u32,
    sender: EventSender,
) {
    std::thread::spawn(move || {
        let result = load_file_diff(&repo_root, &path, source, context_lines);
        let _ = sender.send(AppEvent::DiffLoaded { result });
    });
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;
    use std::time::Duration;

    use super::*;
    use crate::diff::DiffLineKind;
    use crate::events::EventChannel;

    #[test]
    fn spawn_file_diff_load_enqueues_loaded_event() {
        let temp = std::env::temp_dir().join(format!("kiwi-diff-io-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        let status = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&temp)
            .status()
            .expect("git init");
        assert!(status.success());
        Command::new("git")
            .args(["config", "user.email", "kiwi@test.local"])
            .current_dir(&temp)
            .status()
            .expect("git config email");
        Command::new("git")
            .args(["config", "user.name", "Kiwi Test"])
            .current_dir(&temp)
            .status()
            .expect("git config name");
        fs::write(temp.join("one.txt"), "v1\n").expect("write");
        Command::new("git")
            .args(["add", "one.txt"])
            .current_dir(&temp)
            .status()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "init", "-q"])
            .current_dir(&temp)
            .status()
            .expect("git commit");
        fs::write(temp.join("one.txt"), "v2\n").expect("modify");

        let mut channel = EventChannel::new();
        spawn_file_diff_load(
            temp.clone(),
            "one.txt".to_string(),
            DiffSource::Unstaged,
            3,
            channel.sender(),
        );

        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut loaded = None;
        while std::time::Instant::now() < deadline {
            for event in channel.drain_coalesced() {
                if let AppEvent::DiffLoaded { result } = event {
                    assert_eq!(result.path, "one.txt");
                    assert!(result
                        .lines
                        .iter()
                        .any(|line| line.kind == DiffLineKind::Addition && line.content == "v2"));
                    loaded = Some(());
                    break;
                }
            }
            if loaded.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        loaded.expect("diff loaded event");
        let _ = fs::remove_dir_all(temp);
    }
}
