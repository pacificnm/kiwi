//! Styled right-click / popup menus for dock list panels.

use egui::{CursorIcon, Frame, Margin, RichText, Stroke, TextWrapMode, Ui};

use super::layout::truncate_line;
use crate::theme::GuiTheme;

/// Minimum popup width (~2× default egui context menu).
pub const MENU_MIN_WIDTH: f32 = 280.0;

const MENU_ROW_HEIGHT: f32 = 22.0;
const MENU_TITLE_MAX_CHARS: usize = 56;

/// High-contrast popup shell: selection fill, accent border, truncated title header.
pub fn render_menu_shell(
    ui: &mut Ui,
    theme: &GuiTheme,
    title: &str,
    body: impl FnOnce(&mut Ui),
) {
    let title = truncate_line(title, MENU_TITLE_MAX_CHARS);
    Frame::new()
        .fill(theme.role(kiwi_core::theme::SemanticRole::Selection))
        .stroke(Stroke::new(
            1.5,
            theme.role(kiwi_core::theme::SemanticRole::Accent),
        ))
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_min_width(MENU_MIN_WIDTH);
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
            ui.add(
                egui::Label::new(
                    RichText::new(title)
                        .strong()
                        .color(theme.role(kiwi_core::theme::SemanticRole::Fg)),
                )
                .truncate(),
            );
            ui.separator();
            body(ui);
        });
}

/// Checkbox row matching [`menu_action`] layout (View menu tab toggles).
pub fn menu_checkbox(
    ui: &mut Ui,
    theme: &GuiTheme,
    icon: &str,
    label: &str,
    checked: &mut bool,
    enabled: bool,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.set_min_height(MENU_ROW_HEIGHT);
        ui.set_min_width((MENU_MIN_WIDTH - 20.0).max(0.0));
        ui.label(
            RichText::new(icon)
                .monospace()
                .color(theme.role(kiwi_core::theme::SemanticRole::Accent)),
        );
        ui.add_enabled(
            enabled,
            egui::Checkbox::new(
                checked,
                RichText::new(label).color(theme.role(kiwi_core::theme::SemanticRole::Fg)),
            ),
        )
        .on_hover_cursor(if enabled {
            CursorIcon::PointingHand
        } else {
            CursorIcon::Default
        })
    })
    .inner
}

/// One menu row: icon column + truncated action label.
pub fn menu_action(ui: &mut Ui, theme: &GuiTheme, icon: &str, label: &str) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.set_min_height(MENU_ROW_HEIGHT);
        ui.set_min_width((MENU_MIN_WIDTH - 20.0).max(0.0));
        ui.label(
            RichText::new(icon)
                .monospace()
                .color(theme.role(kiwi_core::theme::SemanticRole::Accent)),
        );
        let response = ui
            .add(
                egui::Label::new(
                    RichText::new(label).color(theme.role(kiwi_core::theme::SemanticRole::Fg)),
                )
                .truncate()
                .sense(egui::Sense::click()),
            )
            .on_hover_cursor(CursorIcon::PointingHand);
        if response.clicked() {
            clicked = true;
        }
    });
    clicked
}

/// Icon for [`kiwi_core::github::GhContextMenuAction`] (GUI-only).
#[must_use]
pub fn github_action_icon(action: kiwi_core::github::GhContextMenuAction) -> &'static str {
    use kiwi_core::github::GhContextMenuAction::{
        AddLabels, AssignMilestone, Comment, CreateBranch, Merge, OpenInBrowser, SendToAgent, View,
    };
    match action {
        View => "◎",
        CreateBranch => "⑂",
        Comment => "✎",
        AddLabels => "#",
        AssignMilestone => "◆",
        Merge => "⤵",
        OpenInBrowser => "↗",
        SendToAgent => "▷",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_title_truncates_long_issue_names() {
        let long = "a".repeat(80);
        let truncated = truncate_line(&long, MENU_TITLE_MAX_CHARS);
        assert!(truncated.chars().count() <= MENU_TITLE_MAX_CHARS + 1);
        assert!(truncated.ends_with('…'));
    }
}
