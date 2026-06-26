use std::collections::HashMap;

use super::status::{GitFileEntry, GitFileStatus};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitFileStatusPatch {
    pub changed: Vec<GitFileEntry>,
    pub removed: Vec<String>,
}

impl GitFileStatusPatch {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.removed.is_empty()
    }
}

pub fn diff_git_file_entries(
    previous: &[GitFileEntry],
    incoming: &[GitFileEntry],
) -> GitFileStatusPatch {
    let previous_map: HashMap<&str, GitFileStatus> = previous
        .iter()
        .map(|entry| (entry.path.as_str(), entry.status))
        .collect();
    let incoming_map: HashMap<&str, GitFileStatus> = incoming
        .iter()
        .map(|entry| (entry.path.as_str(), entry.status))
        .collect();

    let mut changed = Vec::new();
    for entry in incoming {
        match previous_map.get(entry.path.as_str()) {
            Some(&status) if status == entry.status => {}
            _ => changed.push(entry.clone()),
        }
    }

    let mut removed = Vec::new();
    for entry in previous {
        if !incoming_map.contains_key(entry.path.as_str()) {
            removed.push(entry.path.clone());
        }
    }

    GitFileStatusPatch { changed, removed }
}

pub fn patch_git_file_entries(
    current: &mut Vec<GitFileEntry>,
    incoming: &[GitFileEntry],
) -> GitFileStatusPatch {
    let patch = diff_git_file_entries(current, incoming);
    if patch.is_empty() && current.len() == incoming.len() {
        return patch;
    }

    for path in &patch.removed {
        current.retain(|entry| &entry.path != path);
    }

    for changed in &patch.changed {
        if let Some(existing) = current.iter_mut().find(|entry| entry.path == changed.path) {
            existing.status = changed.status;
        } else {
            current.push(changed.clone());
        }
    }

    current.sort_by(|left, right| left.path.cmp(&right.path));
    patch
}
