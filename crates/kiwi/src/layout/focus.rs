#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum PaneFocus {
    Left,
    Main,
    CommandPalette,
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum Region {
    LeftTabs,
    LeftContent,
    MainTabs,
    MainContent,
    Palette,
    Shell,
    StatusBar,
}

impl PaneFocus {
    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn focused_region(self) -> Region {
        match self {
            Self::Left => Region::LeftContent,
            Self::Main => Region::MainContent,
            Self::CommandPalette => Region::Palette,
            Self::Shell => Region::Shell,
        }
    }

    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_focused(self, region: Region) -> bool {
        self.focused_region() == region
    }
}
