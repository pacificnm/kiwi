use super::types::{MilestonePickerState, RepoMilestonesLoadResult};

pub fn apply_milestone_picker_load(picker: &mut MilestonePickerState, result: RepoMilestonesLoadResult) {
    picker.loading = false;
    picker.error = result.error;
    picker.milestones = result.milestones;
    if picker.cursor >= picker.milestones.len() {
        picker.cursor = picker.milestones.len().saturating_sub(1);
    }
}
