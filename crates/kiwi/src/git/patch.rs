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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_detects_added_modified_and_removed_paths() {
        let previous = vec![
            GitFileEntry {
                path: "a.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "b.rs".to_string(),
                status: GitFileStatus::Modified,
            },
        ];
        let incoming = vec![
            GitFileEntry {
                path: "a.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "c.rs".to_string(),
                status: GitFileStatus::Untracked,
            },
        ];

        let patch = diff_git_file_entries(&previous, &incoming);
        assert_eq!(patch.removed, vec!["b.rs".to_string()]);
        assert_eq!(patch.changed.len(), 1);
        assert_eq!(patch.changed[0].path, "c.rs");
    }

    #[test]
    fn patch_updates_current_entries_incrementally() {
        let mut current = vec![GitFileEntry {
            path: "src/main.rs".to_string(),
            status: GitFileStatus::Modified,
        }];
        let incoming = vec![
            GitFileEntry {
                path: "src/main.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "src/new.rs".to_string(),
                status: GitFileStatus::Added,
            },
        ];

        let patch = patch_git_file_entries(&mut current, &incoming);
        assert_eq!(patch.changed.len(), 1);
        assert_eq!(current.len(), 2);
        assert!(current.iter().any(|entry| entry.path == "src/new.rs"));
    }

    #[test]
    fn patch_is_empty_when_entries_unchanged() {
        let entries = vec![GitFileEntry {
            path: "lib.rs".to_string(),
            status: GitFileStatus::Modified,
        }];
        let mut current = entries.clone();

        let patch = patch_git_file_entries(&mut current, &entries);
        assert!(patch.is_empty());
        assert_eq!(current, entries);
    }
}
