use std::path::PathBuf;

use crate::state::{AppEvent, EventSender};

use super::auth::check_github_auth;
use super::issue::load_issue_list;

pub fn spawn_github_auth_check(command: String, sender: EventSender) {
    std::thread::spawn(move || {
        let result = check_github_auth(&command);
        let _ = sender.send(AppEvent::GitHubAuthChecked { result });
    });
}

pub fn spawn_github_issue_list_load(repo_root: PathBuf, command: String, sender: EventSender) {
    std::thread::spawn(move || {
        let result = load_issue_list(&repo_root, &command);
        let _ = sender.send(AppEvent::GitHubIssuesLoaded { result });
    });
}
