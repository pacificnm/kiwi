//! Shared read-only context for dock panel rendering (ADR-022).

use kiwi_core::state::AppState;

use crate::theme::GuiTheme;

/// Inputs available to dock panels; panels must not mutate [`AppState`] directly.
pub struct PanelContext<'a> {
    pub state: &'a AppState,
    pub theme: &'a GuiTheme,
}
