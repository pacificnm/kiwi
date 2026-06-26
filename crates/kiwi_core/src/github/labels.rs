use super::types::{LabelPickerState, RepoLabelsLoadResult};

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
