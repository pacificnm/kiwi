//! Shared context for dock panel rendering (ADR-022).

use kiwi_core::events::AppCommand;
use kiwi_core::state::AppState;

use crate::dock::tab::KiwiTab;
use crate::theme::GuiTheme;

/// Keyboard focus state for PTY dock surfaces (updated each frame during panel render).
#[derive(Debug, Default, Clone, Copy)]
pub struct PtySurfaceState {
    pub shell_keyboard_focus: bool,
    pub agent_keyboard_focus: bool,
}

/// Inputs available to dock panels.
pub struct PanelContext<'a> {
    pub state: &'a mut AppState,
    pub theme: &'a GuiTheme,
    pub dispatch: &'a mut dyn FnMut(AppCommand) -> bool,
    pub pty_surface: &'a mut PtySurfaceState,
    /// Active tab in the focused dock leaf (egui_dock focus), if any.
    pub focused_dock_tab: Option<KiwiTab>,
}

impl PanelContext<'_> {
    #[must_use]
    pub fn is_dock_tab_focused(&self, tab: KiwiTab) -> bool {
        self.focused_dock_tab == Some(tab)
    }
}
