mod keys;
mod state;
mod tabs;

pub use keys::map_key;
pub use state::{NavCommand, NavigationState};
pub use tabs::{LEFT_TAB_LABELS, MAIN_TAB_LABELS};

#[cfg(test)]
pub use tabs::{LeftNavTab, MainTab};
