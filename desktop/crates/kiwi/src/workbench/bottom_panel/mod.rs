//! Bottom panel — tab selection and content dispatch.

mod debug;
mod git;
mod logs;
mod output;
mod problems;
mod terminal;
mod tool_activity;

use egui::{pos2, Align, Button, Frame, Layout, Rect, RichText, ScrollArea, Stroke, Ui};

use crate::theme::PALETTE;
use crate::workbench::state::WorkbenchState;

const BOTTOM_TAB_BAR_HEIGHT: f32 = 32.0;
const BOTTOM_TAB_LABEL_SIZE: f32 = 12.0;

/// Bottom panel tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BottomTab {
    /// Integrated terminal.
    #[default]
    Terminal,
    /// Compiler/linter problems.
    Problems,
    /// General output stream.
    Output,
    /// Application logs.
    Logs,
    /// Agent tool activity.
    ToolActivity,
    /// Debugger console.
    Debug,
    /// Git operations log.
    Git,
}

impl BottomTab {
    /// All bottom panel tabs.
    pub const ALL: [Self; 7] = [
        Self::Terminal,
        Self::Problems,
        Self::Output,
        Self::Logs,
        Self::ToolActivity,
        Self::Debug,
        Self::Git,
    ];

    /// Tab label for the tab bar.
    pub fn label(self) -> &'static str {
        match self {
            Self::Terminal => "Terminal",
            Self::Problems => "Problems",
            Self::Output => "Output",
            Self::Logs => "Logs",
            Self::ToolActivity => "Tool Activity",
            Self::Debug => "Debug",
            Self::Git => "Git",
        }
    }
}

/// Renders the bottom panel tab bar and active content.
pub fn bottom_panel(ui: &mut Ui, state: &mut WorkbenchState) {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.set_min_height(ui.available_height());

    tab_bar(ui, &mut state.bottom_tab);

    Frame::new()
        .fill(PALETTE.background_panel)
        .outer_margin(egui::Margin {
            top: -1,
            left: 0,
            right: 0,
            bottom: 0,
        })
        .inner_margin(egui::Margin::symmetric(8, 8))
        .show(ui, |ui| {
            let content_height = ui.available_height().max(0.0);
            ui.set_min_height(content_height);
            ScrollArea::vertical()
                .id_salt("kiwi-bottom-panel")
                .max_height(content_height)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    match state.bottom_tab {
                        BottomTab::Terminal => terminal::show(ui),
                        BottomTab::Problems => problems::show(ui),
                        BottomTab::Output => output::show(ui),
                        BottomTab::Logs => logs::show(ui),
                        BottomTab::ToolActivity => tool_activity::show(ui, state),
                        BottomTab::Debug => debug::show(ui),
                        BottomTab::Git => git::show(ui),
                    }
                });
        });
}

fn tab_bar(ui: &mut Ui, active: &mut BottomTab) {
    let border = tab_border_stroke();

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), BOTTOM_TAB_BAR_HEIGHT),
        Layout::top_down(Align::LEFT),
        |ui| {
            ui.set_min_height(BOTTOM_TAB_BAR_HEIGHT);
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, PALETTE.background_editor);

            let mut select_tab = None;
            let mut active_tab_rect = None;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.set_min_height(BOTTOM_TAB_BAR_HEIGHT);

                for tab in BottomTab::ALL {
                    let selected = *active == tab;
                    let (clicked, tab_rect) = tab_item(ui, tab.label(), selected);
                    if clicked {
                        select_tab = Some(tab);
                    }
                    if selected {
                        active_tab_rect = Some(tab_rect);
                    }
                }
            });

            let strip_rect = ui.max_rect();
            ui.painter().hline(
                strip_rect.x_range(),
                strip_rect.bottom() - 1.0,
                border,
            );

            if let Some(tab_rect) = active_tab_rect {
                paint_active_tab_chrome(ui, tab_rect, strip_rect, border);
            }

            if let Some(tab) = select_tab {
                *active = tab;
            }
        },
    );
}

fn tab_item(ui: &mut Ui, label: &str, selected: bool) -> (bool, Rect) {
    let fill = if selected {
        PALETTE.background_panel
    } else {
        PALETTE.background_editor
    };

    let text = if selected {
        RichText::new(label)
            .size(BOTTOM_TAB_LABEL_SIZE)
            .color(PALETTE.text_primary)
    } else {
        RichText::new(label)
            .size(BOTTOM_TAB_LABEL_SIZE)
            .color(PALETTE.text_secondary)
    };

    let mut clicked = false;
    let output = Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_height(BOTTOM_TAB_BAR_HEIGHT - 16.0);
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                clicked = ui.add(Button::new(text).frame(false)).clicked();
            })
        });

    (clicked, output.response.rect)
}

fn paint_active_tab_chrome(ui: &Ui, tab_rect: Rect, strip_rect: Rect, border: Stroke) {
    let painter = ui.painter();
    let bottom = strip_rect.bottom() + 1.0;

    if tab_rect.bottom() < bottom - 0.5 {
        painter.rect_filled(
            Rect::from_min_max(
                pos2(tab_rect.left(), tab_rect.bottom()),
                pos2(tab_rect.right(), bottom),
            ),
            0.0,
            PALETTE.background_panel,
        );
    }

    let outline = Rect::from_min_max(
        pos2(tab_rect.left(), strip_rect.top()),
        pos2(tab_rect.right(), bottom),
    );
    painter.vline(outline.left(), outline.top()..=outline.bottom(), border);
    painter.vline(outline.right(), outline.top()..=outline.bottom(), border);
    painter.hline(outline.x_range(), outline.top(), border);
}

fn tab_border_stroke() -> Stroke {
    Stroke::new(1.0, PALETTE.border_default)
}
