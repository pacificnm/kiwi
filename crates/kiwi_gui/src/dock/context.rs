//! Shared context for dock panel rendering (ADR-022).

use kiwi_core::events::AppCommand;
use kiwi_core::state::AppState;

use crate::theme::GuiTheme;

/// Inputs available to dock panels.
///
/// Domain state changes must go through [`Self::dispatch`]. Panels may update
/// [`AppState::viewport`] row counts for keyboard scroll clamping.
pub struct PanelContext<'a> {
    pub state: &'a mut AppState,
    pub theme: &'a GuiTheme,
    pub dispatch: &'a mut dyn FnMut(AppCommand) -> bool,
}
