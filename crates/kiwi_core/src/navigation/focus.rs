#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusTarget {
    Left,
    Main,
    CommandPalette,
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    LeftContent,
    MainContent,
    Palette,
    Shell,
    #[cfg_attr(not(test), allow(dead_code))]
    LeftTabs,
    #[cfg_attr(not(test), allow(dead_code))]
    MainTabs,
    #[cfg_attr(not(test), allow(dead_code))]
    StatusBar,
}

impl FocusTarget {
    #[cfg_attr(not(test), allow(dead_code))]
    pub const CYCLE: [Self; 4] = [Self::Left, Self::Main, Self::CommandPalette, Self::Shell];

    #[must_use]
    pub const fn focused_region(self) -> Region {
        match self {
            Self::Left => Region::LeftContent,
            Self::Main => Region::MainContent,
            Self::CommandPalette => Region::Palette,
            Self::Shell => Region::Shell,
        }
    }

    #[must_use]
    pub fn is_focused(self, region: Region) -> bool {
        self.focused_region() == region
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Left => Self::Main,
            Self::Main => Self::CommandPalette,
            Self::CommandPalette => Self::Shell,
            Self::Shell => Self::Left,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Left => Self::Shell,
            Self::Main => Self::Left,
            Self::CommandPalette => Self::Main,
            Self::Shell => Self::CommandPalette,
        }
    }
}
