//! Tab indices, focus targets, and navigation state (SPEC-004).

mod focus;
mod state;
mod tabs;

pub use focus::{FocusTarget, Region};
pub use state::{NavCommand, NavigationState, TabSlotState};
pub use tabs::{LeftNavTab, MainTab, LEFT_TAB_LABELS, MAIN_TAB_LABELS};
