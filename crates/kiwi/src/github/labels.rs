use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::issue::command_on_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoLabel {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoLabelsLoadResult {
    pub labels: Vec<RepoLabel>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelPickerState {
    pub issue_number: u32,
    pub existing_labels: Vec<String>,
    pub labels: Vec<RepoLabel>,
    pub cursor: usize,
    pub selected: Vec<bool>,
    pub loading: bool,
    pub applying: bool,
    pub error: Option<String>,
}

impl LabelPickerState {
    pub fn new(issue_number: u32, existing_labels: Vec<String>) -> Self {
        Self {
            issue_number,
            existing_labels,
            labels: Vec::new(),
            cursor: 0,
            selected: Vec::new(),
            loading: true,
            applying: false,
            error: None,
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        if self.labels.is_empty() {
            self.cursor = 0;
            return;
        }

        let len = self.labels.len() as i32;
        let next = (self.cursor as i32 + delta).rem_euclid(len);
        self.cursor = usize::try_from(next).unwrap_or(0);
    }

    pub fn toggle_cursor(&mut self) {
        if let Some(selected) = self.selected.get_mut(self.cursor) {
            *selected = !*selected;
        }
    }

    pub fn labels_to_add(&self) -> Vec<String> {
        self.labels
            .iter()
            .zip(self.selected.iter())
            .filter(|(label, selected)| {
                **selected && !self.existing_labels.contains(&label.name)
            })
            .map(|(label, _)| label.name.clone())
            .collect()
    }
}

pub fn load_repo_labels(repo_root: &Path, command: &str) -> RepoLabelsLoadResult {
    if !command_on_path(command) {
        return RepoLabelsLoadResult {
            labels: Vec::new(),
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
        };
    }

    let output = Command::new(command)
        .args([
            "label",
            "list",
            "--json",
            "name,description",
            "--limit",
            "200",
        ])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => match parse_repo_labels_json(&result.stdout) {
            Ok(labels) => RepoLabelsLoadResult {
                labels,
                error: None,
            },
            Err(message) => RepoLabelsLoadResult {
                labels: Vec::new(),
                error: Some(message),
            },
        },
        Ok(result) => RepoLabelsLoadResult {
            labels: Vec::new(),
            error: Some(format_label_list_failure(&result.stderr, &result.stdout)),
        },
        Err(err) => RepoLabelsLoadResult {
            labels: Vec::new(),
            error: Some(format!("Failed to run `{command} label list`: {err}")),
        },
    }
}

fn parse_repo_labels_json(bytes: &[u8]) -> Result<Vec<RepoLabel>, String> {
    let raw: Vec<GhLabel> =
        serde_json::from_slice(bytes).map_err(|err| format!("Invalid gh label JSON: {err}"))?;

    Ok(raw
        .into_iter()
        .map(|label| RepoLabel {
            name: label.name,
            description: label.description.unwrap_or_default(),
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct GhLabel {
    name: String,
    description: Option<String>,
}

fn format_label_list_failure(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }

    "gh label list failed".to_string()
}

pub fn apply_label_picker_load(
    picker: &mut LabelPickerState,
    result: RepoLabelsLoadResult,
    existing_labels: &[String],
) {
    picker.loading = false;
    picker.error = result.error;
    picker.labels = result.labels;
    picker.selected = picker
        .labels
        .iter()
        .map(|label| existing_labels.contains(&label.name))
        .collect();
    if picker.cursor >= picker.labels.len() {
        picker.cursor = picker.labels.len().saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_repo_labels_json_maps_fields() {
        let json = r#"[
            {"name": "bug", "description": "Something is wrong"},
            {"name": "enhancement"}
        ]"#;

        let labels = parse_repo_labels_json(json.as_bytes()).expect("parse");
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "bug");
        assert_eq!(labels[0].description, "Something is wrong");
        assert!(labels[1].description.is_empty());
    }

    #[test]
    fn label_picker_toggle_and_collect_names() {
        let mut picker = LabelPickerState {
            issue_number: 1,
            existing_labels: Vec::new(),
            labels: vec![
                RepoLabel {
                    name: "bug".to_string(),
                    description: String::new(),
                },
                RepoLabel {
                    name: "docs".to_string(),
                    description: String::new(),
                },
            ],
            cursor: 1,
            selected: vec![false, false],
            loading: false,
            applying: false,
            error: None,
        };

        picker.toggle_cursor();
        assert_eq!(picker.labels_to_add(), vec!["docs".to_string()]);
    }

    #[test]
    fn apply_label_picker_load_marks_existing_labels() {
        let mut picker = LabelPickerState::new(42, vec!["bug".to_string()]);
        apply_label_picker_load(
            &mut picker,
            RepoLabelsLoadResult {
                labels: vec![
                    RepoLabel {
                        name: "bug".to_string(),
                        description: String::new(),
                    },
                    RepoLabel {
                        name: "docs".to_string(),
                        description: String::new(),
                    },
                ],
                error: None,
            },
            &["bug".to_string()],
        );

        assert!(!picker.loading);
        assert_eq!(picker.selected, vec![true, false]);
    }
}
