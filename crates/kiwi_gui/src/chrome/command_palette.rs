//! Centered command palette modal (SPEC-013 / ADR-014 / #187).

use egui::{Align2, Context, Key, RichText, TextEdit, Vec2};
use kiwi_core::commands::{
    command_available_at, command_shortcut_at, command_title_at, refresh_matches,
    MAX_VISIBLE_MATCHES,
};
use kiwi_core::events::AppCommand;
use kiwi_core::state::{AppState, ReduceView};
use kiwi_core::theme::SemanticRole;

use crate::theme::GuiTheme;

const PALETTE_WIDTH: f32 = 600.0;

/// Global shortcut to open the palette when it is closed.
#[must_use]
pub fn palette_open_shortcut_action(ctx: &Context) -> Option<AppCommand> {
    let open = ctx.input(|input| {
        let cmd = input.modifiers.command;
        (cmd && input.modifiers.shift && input.key_pressed(Key::P))
            || (cmd && input.key_pressed(Key::K))
    });
    open.then_some(AppCommand::PaletteOpen)
}

/// Keyboard action while the palette is open.
#[must_use]
pub fn palette_keyboard_action(
    ctx: &Context,
    prompt_mode: bool,
    input_empty: bool,
) -> Option<AppCommand> {
    ctx.input_mut(|input| {
        if input.key_pressed(Key::Escape) {
            return Some(AppCommand::PaletteClose);
        }
        if input.key_pressed(Key::Enter) {
            return Some(AppCommand::PaletteExecuteSelected);
        }
        if !prompt_mode {
            if input.key_pressed(Key::ArrowUp) {
                if input_empty {
                    return Some(AppCommand::PaletteHistoryUp);
                }
                return Some(AppCommand::PaletteMoveSelection(-1));
            }
            if input.key_pressed(Key::ArrowDown) {
                if input_empty {
                    return Some(AppCommand::PaletteHistoryDown);
                }
                return Some(AppCommand::PaletteMoveSelection(1));
            }
        }
        None
    })
}

/// Render the palette modal. Returns a command to dispatch when the user clicks a row.
pub fn render_command_palette(
    ctx: &Context,
    theme: &GuiTheme,
    state: &mut AppState,
) -> Option<AppCommand> {
    if !state.palette.open {
        return None;
    }

    let title = state
        .palette
        .prompt
        .as_ref()
        .map(|prompt| prompt.title())
        .unwrap_or_else(|| "Command Palette".to_string());

    let mut clicked_match = None;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .title_bar(true)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .default_width(PALETTE_WIDTH)
        .max_width(PALETTE_WIDTH)
        .show(ctx, |ui| {
            ui.set_width(PALETTE_WIDTH - 32.0);

            let response = ui.add(
                TextEdit::singleline(&mut state.palette.input)
                    .hint_text("Type a command…")
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(8.0, 6.0)),
            );
            response.request_focus();

            if response.changed() {
                state.palette.history_cursor = None;
                let mut view = ReduceView::from_app_state(state);
                refresh_matches(&mut view);
                view.set_dirty();
            }

            if let Some(prompt) = &state.palette.prompt {
                ui.label(
                    RichText::new(prompt.hint())
                        .color(theme.role(SemanticRole::Muted))
                        .size(theme.font_size),
                );
                return;
            }

            let match_count = state.palette.matches.len().min(MAX_VISIBLE_MATCHES);

            ui.add_space(4.0);
            for match_index in 0..match_count {
                let Some(registry_index) = state.palette.matches.get(match_index).copied() else {
                    break;
                };
                let selected = state.palette.selected == match_index;
                let view = ReduceView::from_app_state(state);
                let available = command_available_at(&view, registry_index);
                let title = command_title_at(&view, registry_index).unwrap_or("?");
                let shortcut = command_shortcut_at(&view, registry_index)
                    .map(|value| format!("  {value}"))
                    .unwrap_or_default();

                let role = if available {
                    SemanticRole::Fg
                } else {
                    SemanticRole::Muted
                };
                let label = RichText::new(format!("{title}{shortcut}"))
                    .color(theme.role(role))
                    .size(theme.font_size);

                if ui.selectable_label(selected, label).clicked() {
                    clicked_match = Some(match_index);
                }
            }
        });

    clicked_match.map(AppCommand::PaletteExecuteMatch)
}
