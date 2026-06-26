use super::focus::FocusTarget;
use super::tabs::{LeftNavTab, MainTab};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TabSlotState {
    pub scroll_offset: u16,
    pub selected_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationState {
    pub left_tab: LeftNavTab,
    pub main_tab: MainTab,
    pub focus: FocusTarget,
    left_slots: [TabSlotState; 4],
    main_slots: [TabSlotState; 8],
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            left_tab: LeftNavTab::Files,
            main_tab: MainTab::Agent,
            focus: FocusTarget::Main,
            left_slots: [TabSlotState::default(); 4],
            main_slots: [TabSlotState::default(); 8],
        }
    }
}

impl NavigationState {
    #[cfg_attr(not(test), allow(dead_code))]
    #[must_use]
    pub fn left_slot(&self) -> &TabSlotState {
        &self.left_slots[self.left_tab.index()]
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn left_slot_mut(&mut self) -> &mut TabSlotState {
        let index = self.left_tab.index();
        &mut self.left_slots[index]
    }

    #[must_use]
    pub fn main_slot(&self) -> &TabSlotState {
        &self.main_slots[self.main_tab.index()]
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn main_slot_mut(&mut self) -> &mut TabSlotState {
        let index = self.main_tab.index();
        &mut self.main_slots[index]
    }

    #[cfg_attr(not(test), allow(dead_code))]
    #[must_use]
    pub fn left_slot_for(&self, tab: LeftNavTab) -> &TabSlotState {
        &self.left_slots[tab.index()]
    }

    #[cfg_attr(not(test), allow(dead_code))]
    #[must_use]
    pub fn main_slot_for(&self, tab: MainTab) -> &TabSlotState {
        &self.main_slots[tab.index()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavCommand {
    SelectLeftTab(LeftNavTab),
    SelectMainTab(MainTab),
    SetFocus(FocusTarget),
    NextFocus,
    PreviousFocus,
}

impl NavigationState {
    pub fn apply(&mut self, command: NavCommand) {
        match command {
            NavCommand::SelectLeftTab(tab) => self.left_tab = tab,
            NavCommand::SelectMainTab(tab) => {
                self.main_tab = tab;
                if let Some(left) = tab.paired_left_tab() {
                    self.left_tab = left;
                }
            }
            NavCommand::SetFocus(focus) => self.focus = focus,
            NavCommand::NextFocus => {
                self.focus = self.focus.next();
            }
            NavCommand::PreviousFocus => {
                self.focus = self.focus.previous();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_files_and_agent_with_main_focus() {
        let nav = NavigationState::default();
        assert_eq!(nav.left_tab, LeftNavTab::Files);
        assert_eq!(nav.main_tab, MainTab::Agent);
        assert_eq!(nav.focus, FocusTarget::Main);
    }

    #[test]
    fn left_tab_select_does_not_change_main_tab() {
        let mut nav = NavigationState::default();
        nav.apply(NavCommand::SelectLeftTab(LeftNavTab::Git));
        assert_eq!(nav.left_tab, LeftNavTab::Git);
        assert_eq!(nav.main_tab, MainTab::Agent);
    }

    #[test]
    fn main_tab_select_pairs_left_tab() {
        let mut nav = NavigationState::default();
        nav.apply(NavCommand::SelectMainTab(MainTab::Diff));
        assert_eq!(nav.main_tab, MainTab::Diff);
        assert_eq!(nav.left_tab, LeftNavTab::Git);

        nav.apply(NavCommand::SelectMainTab(MainTab::Agent));
        assert_eq!(nav.left_tab, LeftNavTab::Git);
    }

    #[test]
    fn tab_slot_state_preserved_when_switching_back() {
        let mut nav = NavigationState::default();
        nav.left_slot_mut().selected_index = 7;
        nav.apply(NavCommand::SelectLeftTab(LeftNavTab::Search));
        nav.left_slot_mut().selected_index = 2;
        nav.apply(NavCommand::SelectLeftTab(LeftNavTab::Files));

        assert_eq!(nav.left_slot_for(LeftNavTab::Files).selected_index, 7);
        assert_eq!(nav.left_slot_for(LeftNavTab::Search).selected_index, 2);
    }

    #[test]
    fn main_tab_slot_state_preserved_when_switching_back() {
        let mut nav = NavigationState::default();
        nav.main_slot_mut().selected_index = 4;
        nav.apply(NavCommand::SelectMainTab(MainTab::Preview));
        nav.main_slot_mut().selected_index = 1;
        nav.apply(NavCommand::SelectMainTab(MainTab::Agent));

        assert_eq!(nav.main_slot_for(MainTab::Agent).selected_index, 4);
        assert_eq!(nav.main_slot_for(MainTab::Preview).selected_index, 1);
    }

    #[test]
    fn focus_cycles_forward_and_backward() {
        let mut nav = NavigationState::default();
        assert_eq!(nav.focus, FocusTarget::Main);

        nav.apply(NavCommand::NextFocus);
        assert_eq!(nav.focus, FocusTarget::CommandPalette);
        nav.apply(NavCommand::NextFocus);
        assert_eq!(nav.focus, FocusTarget::Shell);
        nav.apply(NavCommand::NextFocus);
        assert_eq!(nav.focus, FocusTarget::Left);

        nav.apply(NavCommand::PreviousFocus);
        assert_eq!(nav.focus, FocusTarget::Shell);
    }

    #[test]
    fn focus_cycle_matches_spec_order() {
        let mut focus = FocusTarget::CYCLE[0];
        for expected in FocusTarget::CYCLE.iter().skip(1) {
            focus = focus.next();
            assert_eq!(focus, *expected);
        }
    }
}
