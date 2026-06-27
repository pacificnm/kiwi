//! Dock layout serialization for workspace persistence (SPEC-022 / #186).

use egui_dock::DockState;
use kiwi_core::workspace::GuiWorkspaceSnapshot;

use super::layout::initial_dock_state;
use super::tab::KiwiTab;

/// Serialize the current dock tree for workspace storage.
#[must_use]
pub fn snapshot_from_dock(dock: &DockState<KiwiTab>) -> GuiWorkspaceSnapshot {
    let open_tabs = dock
        .iter_all_tabs()
        .map(|(_, tab)| tab_storage_name(*tab))
        .collect();
    let dock_layout = serde_json::to_value(dock).unwrap_or_else(|err| {
        eprintln!("workspace: failed to serialize dock layout: {err}");
        serde_json::Value::Null
    });
    GuiWorkspaceSnapshot {
        dock_layout,
        open_tabs,
    }
}

/// Restore dock tree from persisted GUI workspace data; falls back to factory layout.
#[must_use]
pub fn restore_dock(gui: &GuiWorkspaceSnapshot) -> DockState<KiwiTab> {
    if gui.dock_layout.is_null() {
        eprintln!("workspace: missing gui dock layout; using default layout");
        return initial_dock_state();
    }

    match serde_json::from_value::<DockState<KiwiTab>>(gui.dock_layout.clone()) {
        Ok(dock) if dock.iter_all_tabs().next().is_some() => dock,
        Ok(_) => {
            eprintln!("workspace: empty gui dock layout; using default layout");
            initial_dock_state()
        }
        Err(err) => {
            eprintln!("workspace: failed to restore gui dock layout: {err}");
            initial_dock_state()
        }
    }
}

fn tab_storage_name(tab: KiwiTab) -> String {
    serde_json::to_value(tab)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| format!("{tab:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::layout::tabs_in_dock;

    #[test]
    fn factory_dock_round_trips_through_workspace_json() {
        let dock = initial_dock_state();
        let snapshot = snapshot_from_dock(&dock);
        let restored = restore_dock(&snapshot);
        assert_eq!(
            tabs_in_dock(&restored),
            tabs_in_dock(&dock),
            "tab set should match after round-trip"
        );
    }

    #[test]
    fn corrupt_dock_layout_falls_back_to_factory() {
        let gui = GuiWorkspaceSnapshot {
            dock_layout: serde_json::json!("not-a-dock"),
            open_tabs: Vec::new(),
        };
        let restored = restore_dock(&gui);
        assert_eq!(tabs_in_dock(&restored).len(), KiwiTab::factory_tabs().len());
    }

    #[test]
    fn null_dock_layout_falls_back_to_factory() {
        let gui = GuiWorkspaceSnapshot {
            dock_layout: serde_json::Value::Null,
            open_tabs: Vec::new(),
        };
        let restored = restore_dock(&gui);
        assert!(restored.find_main_surface_tab(&KiwiTab::Agent).is_some());
    }
}
