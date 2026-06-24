use std::fmt;

use ratatui::layout::Rect;

pub const MIN_TERMINAL_WIDTH: u16 = 80;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;
pub const STATUS_BAR_HEIGHT: u16 = 1;
pub const TAB_BAR_HEIGHT: u16 = 1;
const BOTTOM_PANEL_MIN_HEIGHT: u16 = 5;
const BOTTOM_PANEL_HEIGHT_PERCENT: u16 = 25;
const MIN_CONTENT_HEIGHT: u16 = 1;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutRects {
    pub status_bar: Rect,
    pub left_tabs: Rect,
    pub left_content: Rect,
    pub main_tabs: Rect,
    pub main_content: Rect,
    pub palette: Rect,
    pub shell: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutState {
    pub rects: LayoutRects,
    pub terminal_size: (u16, u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutError {
    TerminalTooSmall {
        width: u16,
        height: u16,
        minimum: (u16, u16),
    },
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TerminalTooSmall {
                width,
                height,
                minimum,
            } => write!(
                f,
                "terminal size {width}x{height} is below minimum {}x{}",
                minimum.0, minimum.1
            ),
        }
    }
}

impl std::error::Error for LayoutError {}

pub fn compute_layout(
    width: u16,
    height: u16,
    left_width_percent: u8,
) -> Result<LayoutState, LayoutError> {
    if width < MIN_TERMINAL_WIDTH || height < MIN_TERMINAL_HEIGHT {
        return Err(LayoutError::TerminalTooSmall {
            width,
            height,
            minimum: (MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT),
        });
    }

    let left_width = left_panel_width(width, left_width_percent);
    let main_width = width.saturating_sub(left_width);

    let workspace_height = height.saturating_sub(STATUS_BAR_HEIGHT);
    let area_below_tabs = workspace_height.saturating_sub(TAB_BAR_HEIGHT);
    let bottom_height = bottom_panel_height(area_below_tabs);
    let content_height = area_below_tabs.saturating_sub(bottom_height);

    let content_top = TAB_BAR_HEIGHT;
    let bottom_top = content_top.saturating_add(content_height);

    let rects = LayoutRects {
        status_bar: Rect {
            x: 0,
            y: height.saturating_sub(STATUS_BAR_HEIGHT),
            width,
            height: STATUS_BAR_HEIGHT,
        },
        left_tabs: Rect {
            x: 0,
            y: 0,
            width: left_width,
            height: TAB_BAR_HEIGHT,
        },
        main_tabs: Rect {
            x: left_width,
            y: 0,
            width: main_width,
            height: TAB_BAR_HEIGHT,
        },
        left_content: Rect {
            x: 0,
            y: content_top,
            width: left_width,
            height: content_height,
        },
        main_content: Rect {
            x: left_width,
            y: content_top,
            width: main_width,
            height: content_height,
        },
        palette: Rect {
            x: 0,
            y: bottom_top,
            width: left_width,
            height: bottom_height,
        },
        shell: Rect {
            x: left_width,
            y: bottom_top,
            width: main_width,
            height: bottom_height,
        },
    };

    Ok(LayoutState {
        rects,
        terminal_size: (width, height),
    })
}

#[must_use]
#[cfg_attr(not(test), allow(dead_code))]
pub fn shell_pty_size(rects: &LayoutRects) -> (u16, u16) {
    pty_dimensions(rects.shell)
}

fn left_panel_width(width: u16, left_width_percent: u8) -> u16 {
    let percent = u32::from(left_width_percent.clamp(10, 50));
    let computed = (u32::from(width) * percent / 100) as u16;
    computed.clamp(1, width.saturating_sub(1))
}

fn bottom_panel_height(area_below_tabs: u16) -> u16 {
    let percent_height =
        (u32::from(area_below_tabs) * u32::from(BOTTOM_PANEL_HEIGHT_PERCENT) / 100) as u16;
    let desired = percent_height.max(BOTTOM_PANEL_MIN_HEIGHT);
    let max_bottom = area_below_tabs.saturating_sub(MIN_CONTENT_HEIGHT);
    desired.min(max_bottom.max(BOTTOM_PANEL_MIN_HEIGHT.min(area_below_tabs)))
}

fn pty_dimensions(area: Rect) -> (u16, u16) {
    let cols = area.width.saturating_sub(2).max(1);
    let rows = area.height.saturating_sub(2).max(1);
    (cols, rows)
}

#[cfg(test)]
mod tests {
    use super::super::focus::{PaneFocus, Region};
    use super::*;

    fn assert_rect(rect: Rect, x: u16, y: u16, width: u16, height: u16) {
        assert_eq!(rect.x, x, "x mismatch");
        assert_eq!(rect.y, y, "y mismatch");
        assert_eq!(rect.width, width, "width mismatch");
        assert_eq!(rect.height, height, "height mismatch");
    }

    fn all_regions_non_zero(rects: &LayoutRects) {
        let regions = [
            rects.status_bar,
            rects.left_tabs,
            rects.left_content,
            rects.main_tabs,
            rects.main_content,
            rects.palette,
            rects.shell,
        ];
        for rect in regions {
            assert!(rect.width > 0, "region width must be > 0: {rect:?}");
            assert!(rect.height > 0, "region height must be > 0: {rect:?}");
        }
    }

    fn regions_fit_terminal(rects: &LayoutRects, width: u16, height: u16) {
        for rect in [
            rects.status_bar,
            rects.left_tabs,
            rects.left_content,
            rects.main_tabs,
            rects.main_content,
            rects.palette,
            rects.shell,
        ] {
            assert!(
                rect.x + rect.width <= width,
                "region exceeds width: {rect:?}"
            );
            assert!(
                rect.y + rect.height <= height,
                "region exceeds height: {rect:?}"
            );
        }
    }

    #[test]
    fn default_terminal_120x40_all_regions_visible() {
        let layout = compute_layout(120, 40, 30).expect("layout");
        all_regions_non_zero(&layout.rects);
        regions_fit_terminal(&layout.rects, 120, 40);

        assert_rect(layout.rects.status_bar, 0, 39, 120, 1);
        assert_rect(layout.rects.left_tabs, 0, 0, 36, 1);
        assert_rect(layout.rects.main_tabs, 36, 0, 84, 1);
        assert_rect(layout.rects.left_content, 0, 1, 36, 29);
        assert_rect(layout.rects.main_content, 36, 1, 84, 29);
        assert_rect(layout.rects.palette, 0, 30, 36, 9);
        assert_rect(layout.rects.shell, 36, 30, 84, 9);
    }

    #[test]
    fn minimum_terminal_80x24_degrades_gracefully() {
        let layout = compute_layout(80, 24, 30).expect("layout");
        all_regions_non_zero(&layout.rects);
        regions_fit_terminal(&layout.rects, 80, 24);

        assert_eq!(layout.rects.shell.height, 5);
        assert_eq!(layout.rects.palette.height, 5);
        assert_eq!(layout.rects.left_content.height, 17);
    }

    #[test]
    fn below_minimum_returns_error_without_panic() {
        let err = compute_layout(79, 24, 30).expect_err("width too small");
        assert!(matches!(
            err,
            LayoutError::TerminalTooSmall {
                width: 79,
                height: 24,
                ..
            }
        ));

        let err = compute_layout(80, 23, 30).expect_err("height too small");
        assert!(matches!(
            err,
            LayoutError::TerminalTooSmall {
                width: 80,
                height: 23,
                ..
            }
        ));
    }

    #[test]
    fn left_width_config_reflected_in_rects() {
        let narrow = compute_layout(120, 40, 10).expect("narrow left");
        let wide = compute_layout(120, 40, 50).expect("wide left");

        assert_eq!(narrow.rects.left_content.width, 12);
        assert_eq!(wide.rects.left_content.width, 60);
    }

    #[test]
    fn layout_is_deterministic_for_same_inputs() {
        let first = compute_layout(120, 40, 30).expect("first");
        let second = compute_layout(120, 40, 30).expect("second");
        assert_eq!(first, second);
    }

    #[test]
    fn resize_updates_shell_pty_dimensions() {
        let small = compute_layout(120, 40, 30).expect("small");
        let large = compute_layout(160, 50, 30).expect("large");

        let (small_cols, small_rows) = shell_pty_size(&small.rects);
        let (large_cols, large_rows) = shell_pty_size(&large.rects);

        assert!(large_cols > small_cols);
        assert!(large_rows > small_rows);
    }

    #[test]
    fn focus_border_region_tracks_focus_changes() {
        let left = PaneFocus::Left.focused_region();
        let main = PaneFocus::Main.focused_region();
        let palette = PaneFocus::CommandPalette.focused_region();
        let shell = PaneFocus::Shell.focused_region();

        assert_eq!(left, Region::LeftContent);
        assert_eq!(main, Region::MainContent);
        assert_eq!(palette, Region::Palette);
        assert_eq!(shell, Region::Shell);

        assert!(PaneFocus::Shell.is_focused(Region::Shell));
        assert!(!PaneFocus::Shell.is_focused(Region::MainContent));
    }

    #[test]
    fn large_terminal_within_supported_bounds() {
        let layout = compute_layout(500, 200, 30).expect("large layout");
        all_regions_non_zero(&layout.rects);
        regions_fit_terminal(&layout.rects, 500, 200);
    }
}
