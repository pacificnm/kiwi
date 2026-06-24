use crate::layout::PaneFocus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiState {
    pub left_tab: usize,
    pub main_tab: usize,
    pub focus: PaneFocus,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            left_tab: 0,
            main_tab: 0,
            focus: PaneFocus::Main,
        }
    }
}
