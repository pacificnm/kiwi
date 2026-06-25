use std::path::PathBuf;

use crate::state::{AppEvent, EventSender};

use super::actions::{add_issue_labels, post_issue_comment};
use super::auth::check_github_auth;
use super::browser::{open_in_browser, GitHubBrowserTarget};
use super::detail::load_issue_detail;
use super::issue::load_issue_list;
use super::labels::load_repo_labels;
use super::pr_create::{create_pull_request, PrCreateRequest};

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

pub fn spawn_github_issue_detail_load(
    repo_root: PathBuf,
    command: String,
    number: u32,
    sender: EventSender,
) {
    std::thread::spawn(move || {
        let result = load_issue_detail(&repo_root, &command, number);
        let _ = sender.send(AppEvent::GitHubIssueDetailLoaded { number, result });
    });
}

pub fn spawn_github_issue_comment(
    repo_root: PathBuf,
    command: String,
    number: u32,
    body: String,
    sender: EventSender,
) {
    std::thread::spawn(move || {
        let result = post_issue_comment(&repo_root, &command, number, &body);
        let _ = sender.send(AppEvent::GitHubIssueCommentCompleted { number, result });
    });
}

pub fn spawn_github_repo_labels_load(repo_root: PathBuf, command: String, sender: EventSender) {
    std::thread::spawn(move || {
        let result = load_repo_labels(&repo_root, &command);
        let _ = sender.send(AppEvent::GitHubRepoLabelsLoaded { result });
    });
}

pub fn spawn_github_issue_label_apply(
    repo_root: PathBuf,
    command: String,
    number: u32,
    labels: Vec<String>,
    sender: EventSender,
) {
    std::thread::spawn(move || {
        let result = add_issue_labels(&repo_root, &command, number, &labels);
        let _ = sender.send(AppEvent::GitHubIssueLabelsApplied { number, result });
    });
}

pub fn spawn_github_open_browser(
    repo_root: PathBuf,
    command: String,
    target: GitHubBrowserTarget,
    sender: EventSender,
) {
    std::thread::spawn(move || {
        let result = open_in_browser(&repo_root, &command, target);
        let _ = sender.send(AppEvent::GitHubOpenBrowserCompleted { target, result });
    });
}

pub fn spawn_github_pr_create(
    repo_root: PathBuf,
    command: String,
    request: PrCreateRequest,
    sender: EventSender,
) {
    std::thread::spawn(move || {
        let result = create_pull_request(&repo_root, &command, &request);
        let _ = sender.send(AppEvent::GitHubPrCreateCompleted { result });
    });
}
