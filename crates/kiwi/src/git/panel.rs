use super::status::{GitFileEntry, GitFileStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitPanelRow {
    Header { label: &'static str, count: usize },
    File { path: String, status: GitFileStatus },
}

const GROUP_ORDER: [(GitFileStatus, &str); 4] = [
    (GitFileStatus::Modified, "Modified"),
    (GitFileStatus::Added, "Added"),
    (GitFileStatus::Deleted, "Deleted"),
    (GitFileStatus::Untracked, "Untracked"),
];

pub fn build_panel_rows(entries: &[GitFileEntry], show_untracked: bool) -> Vec<GitPanelRow> {
    let mut rows = Vec::new();

    for (status, label) in GROUP_ORDER {
        if !show_untracked && status == GitFileStatus::Untracked {
            continue;
        }

        let files: Vec<&GitFileEntry> = entries
            .iter()
            .filter(|entry| entry.status == status)
            .collect();
        if files.is_empty() {
            continue;
        }

        rows.push(GitPanelRow::Header {
            label,
            count: files.len(),
        });
        for entry in files {
            rows.push(GitPanelRow::File {
                path: entry.path.clone(),
                status: entry.status,
            });
        }
    }

    rows
}

pub fn selectable_row_indices(rows: &[GitPanelRow]) -> Vec<usize> {
    rows.iter()
        .enumerate()
        .filter_map(|(index, row)| matches!(row, GitPanelRow::File { .. }).then_some(index))
        .collect()
}

pub fn row_for_path(rows: &[GitPanelRow], path: &str) -> Option<usize> {
    rows.iter().enumerate().find_map(|(index, row)| match row {
        GitPanelRow::File { path: row_path, .. } if row_path == path => Some(index),
        _ => None,
    })
}

pub fn path_for_row(rows: &[GitPanelRow], row_index: usize) -> Option<&str> {
    match rows.get(row_index)? {
        GitPanelRow::File { path, .. } => Some(path.as_str()),
        GitPanelRow::Header { .. } => None,
    }
}

pub fn scroll_offset_for_row(
    selected_row: usize,
    scroll_offset: usize,
    viewport_rows: usize,
) -> usize {
    if viewport_rows == 0 {
        return 0;
    }

    if selected_row < scroll_offset {
        selected_row
    } else if selected_row >= scroll_offset.saturating_add(viewport_rows) {
        selected_row.saturating_sub(viewport_rows.saturating_sub(1))
    } else {
        scroll_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_panel_rows_groups_by_status() {
        let entries = vec![
            GitFileEntry {
                path: "b.rs".to_string(),
                status: GitFileStatus::Added,
            },
            GitFileEntry {
                path: "a.rs".to_string(),
                status: GitFileStatus::Modified,
            },
            GitFileEntry {
                path: "c.rs".to_string(),
                status: GitFileStatus::Untracked,
            },
        ];

        let rows = build_panel_rows(&entries, true);
        assert!(matches!(
            rows.first(),
            Some(GitPanelRow::Header {
                label: "Modified",
                count: 1
            })
        ));
        assert!(rows.iter().any(|row| matches!(
            row,
            GitPanelRow::File {
                path,
                status: GitFileStatus::Modified
            } if path == "a.rs"
        )));
        assert!(rows.iter().any(|row| matches!(
            row,
            GitPanelRow::Header {
                label: "Untracked",
                ..
            }
        )));
    }

    #[test]
    fn build_panel_rows_hides_untracked_when_disabled() {
        let entries = vec![GitFileEntry {
            path: "new.rs".to_string(),
            status: GitFileStatus::Untracked,
        }];

        let rows = build_panel_rows(&entries, false);
        assert!(rows.is_empty());
    }
}
