//! Bottom panel — tab selection, resize, and content dispatch.

mod debug;
mod git;
mod logs;
mod output;
mod problems;
pub mod terminal;
pub mod tool_activity;

use egui::containers::panel::PanelState;
use egui::{pos2, Align, Button, Context, Frame, Id, Layout, Rect, RichText, ScrollArea, Stroke, TopBottomPanel, Ui};

use nest_icon::{Icon, IconButton};

use crate::theme::PALETTE;
use crate::workbench::state::WorkbenchState;

/// egui panel id for the resizable bottom dock.
pub const PANEL_ID: &str = "kiwi-bottom-panel";
/// Default bottom panel height in points.
pub const DEFAULT_HEIGHT: f32 = 200.0;
const MIN_HEIGHT: f32 = 80.0;
const MIN_EDITOR_HEIGHT: f32 = 120.0;
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

    /// Tab label including live badges (tool call counts, etc.).
    pub fn label_with_state(self, state: &WorkbenchState) -> String {
        match self {
            Self::ToolActivity => {
                let total = state.tool_activity.len();
                if total == 0 {
                    return self.label().to_string();
                }
                let running = state
                    .tool_activity
                    .iter()
                    .any(|entry| entry.status == crate::workbench::state::ToolActivityStatus::Running);
                if running {
                    format!("Tool Activity ({total}…)")
                } else {
                    format!("Tool Activity ({total})")
                }
            }
            _ => self.label().to_string(),
        }
    }
}

/// Returns the persisted egui id for the bottom panel.
pub fn panel_id() -> Id {
    Id::new(PANEL_ID)
}

/// Maximum height the bottom panel may occupy above the editor.
pub fn max_height(ctx: &Context) -> f32 {
    let available = ctx.available_rect();
    (available.height() - MIN_EDITOR_HEIGHT).max(MIN_HEIGHT)
}

/// Shows the resizable bottom dock. Call before [`egui::CentralPanel`].
pub fn show_panel(ctx: &Context, state: &mut WorkbenchState) {
    let max_h = max_height(ctx);

    if state.bottom_panel_toggle_requested {
        state.bottom_panel_toggle_requested = false;
        apply_panel_toggle(ctx, state, max_h);
    }

    TopBottomPanel::bottom(PANEL_ID)
        .resizable(true)
        .default_height(DEFAULT_HEIGHT)
        .min_height(MIN_HEIGHT)
        .max_height(max_h)
        .show_separator_line(true)
        .frame(
            Frame::new()
                .fill(PALETTE.background_panel)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ctx, |ui| bottom_panel(ui, state));

    sync_maximized_state(ctx, state, max_h);
}

fn apply_panel_toggle(ctx: &Context, state: &mut WorkbenchState, max_h: f32) {
    if state.bottom_panel_maximized {
        set_panel_height(ctx, state.bottom_panel_restored_height);
        state.bottom_panel_maximized = false;
    } else {
        state.bottom_panel_restored_height = current_panel_height(ctx).unwrap_or(DEFAULT_HEIGHT);
        set_panel_height(ctx, max_h);
        state.bottom_panel_maximized = true;
    }
}

fn sync_maximized_state(ctx: &Context, state: &mut WorkbenchState, max_h: f32) {
    let Some(height) = current_panel_height(ctx) else {
        return;
    };

    if state.bottom_panel_maximized && height < max_h - 2.0 {
        state.bottom_panel_maximized = false;
        state.bottom_panel_restored_height = height;
    }
}

fn current_panel_height(ctx: &Context) -> Option<f32> {
    PanelState::load(ctx, panel_id()).map(|panel| panel.rect.height())
}

fn set_panel_height(ctx: &Context, height: f32) {
    let id = panel_id();
    let height = height.max(MIN_HEIGHT);

    let mut rect = if let Some(panel) = PanelState::load(ctx, id) {
        panel.rect
    } else {
        let available = ctx.available_rect();
        Rect::from_min_max(
            pos2(available.min.x, available.max.y - height),
            pos2(available.max.x, available.max.y),
        )
    };

    rect.min.y = rect.max.y - height;
    ctx.data_mut(|data| data.insert_persisted(id, PanelState { rect }));
}

/// Renders the bottom panel tab bar and active content.
fn bottom_panel(ui: &mut Ui, state: &mut WorkbenchState) {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.set_min_height(ui.available_height());

    tab_bar(ui, state);

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
            match state.bottom_tab {
                BottomTab::Terminal => {
                    terminal::show(ui, state);
                }
                active => {
                    ui.set_max_height(content_height);
                    ScrollArea::vertical()
                        .id_salt("kiwi-bottom-panel")
                        .max_height(content_height)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| match active {
                            BottomTab::Problems => problems::show(ui),
                            BottomTab::Output => output::show(ui),
                            BottomTab::Logs => logs::show(ui),
                            BottomTab::ToolActivity => tool_activity::show(ui, state),
                            BottomTab::Debug => debug::show(ui),
                            BottomTab::Git => git::show(ui, state),
                            BottomTab::Terminal => unreachable!(),
                        });
                }
            }
        });
}

fn tab_bar(ui: &mut Ui, state: &mut WorkbenchState) {
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
                    let selected = state.bottom_tab == tab;
                    let tab_label = tab.label_with_state(state);
                    let (clicked, tab_rect) = tab_item(ui, &tab_label, selected);
                    if clicked {
                        select_tab = Some(tab);
                    }
                    if selected {
                        active_tab_rect = Some(tab_rect);
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.set_width(ui.available_width());
                    let (icon, tooltip) = if state.bottom_panel_maximized {
                        (Icon::CHEVRON_DOWN, "Restore bottom panel height")
                    } else {
                        (Icon::CHEVRON_UP, "Expand bottom panel")
                    };
                    if ui
                        .add(
                            IconButton::new(icon)
                                .size(12.0)
                                .min_size(egui::vec2(28.0, BOTTOM_TAB_BAR_HEIGHT - 4.0))
                                .tooltip(tooltip),
                        )
                        .clicked()
                    {
                        state.bottom_panel_toggle_requested = true;
                        ui.ctx().request_repaint();
                    }
                });
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
                state.bottom_tab = tab;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_height_is_reasonable() {
        assert!(DEFAULT_HEIGHT >= MIN_HEIGHT);
    }
}
