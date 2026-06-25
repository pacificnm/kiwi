use crate::state::{AppEvent, EventSender};

use super::auth::check_github_auth;

pub fn spawn_github_auth_check(command: String, sender: EventSender) {
    std::thread::spawn(move || {
        let result = check_github_auth(&command);
        let _ = sender.send(AppEvent::GitHubAuthChecked { result });
    });
}
