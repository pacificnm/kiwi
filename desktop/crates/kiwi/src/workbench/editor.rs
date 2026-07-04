//! Editor area — open tabs and content placeholder.

use egui::{pos2, Align, Button, Frame, Layout, Rect, RichText, ScrollArea, Stroke, Ui};

use crate::theme::PALETTE;
use nest_icon::{Icon, IconButton};

const TAB_BAR_HEIGHT: f32 = 38.0;
const TAB_LABEL_SIZE: f32 = 13.0;
const TAB_CLOSE_SIZE: f32 = 11.0;

/// Open editor tabs and active selection.
#[derive(Debug, Clone, Default)]
pub struct EditorState {
    /// Open tab paths or titles.
    pub tabs: Vec<String>,
    /// Index of the active tab.
    pub active_tab: usize,
}

impl EditorState {
    /// Demo tabs for the layout shell.
    pub fn demo() -> Self {
        Self {
            tabs: vec![
                "src/main.rs".into(),
                "Cargo.toml".into(),
                "README.md".into(),
            ],
            active_tab: 0,
        }
    }
}

/// Renders the editor tab bar and content area.
pub fn editor_panel(ui: &mut Ui, editor: &mut EditorState) {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.set_min_height(ui.available_height());

    editor_tabs(ui, editor);

    let content_height = ui.available_height().max(0.0);
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
            ui.set_min_height(content_height);
            ScrollArea::vertical()
                .id_salt("kiwi-editor")
                .max_height(content_height)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let tab = editor
                        .tabs
                        .get(editor.active_tab)
                        .map(String::as_str)
                        .unwrap_or("untitled");
                    ui.label(
                        RichText::new(format!("Editor — {tab}"))
                            .size(16.0)
                            .weak(),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(
                            "// Kiwi editor workspace\n// Open a file from the explorer.",
                        )
                        .monospace()
                        .size(13.0),
                    );
                });
        });
}

fn editor_tabs(ui: &mut Ui, editor: &mut EditorState) {
    let border = tab_border_stroke();

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), TAB_BAR_HEIGHT),
        Layout::top_down(Align::LEFT),
        |ui| {
            ui.set_min_height(TAB_BAR_HEIGHT);
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, PALETTE.background_editor);

            let mut select_tab = None;
            let mut close_tab_idx = None;
            let mut active_tab_rect = None;
            let paths = editor.tabs.clone();

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.set_min_height(TAB_BAR_HEIGHT);

                for (index, path) in paths.iter().enumerate() {
                    let selected = editor.active_tab == index;
                    let (tab_clicked, close_clicked, tab_rect) =
                        tab_item(ui, path, selected);
                    if tab_clicked {
                        select_tab = Some(index);
                    }
                    if close_clicked {
                        close_tab_idx = Some(index);
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

            if let Some(index) = select_tab {
                editor.active_tab = index;
            }
            if let Some(index) = close_tab_idx {
                close_tab(editor, index);
            }
        },
    );
}

fn tab_item(ui: &mut Ui, path: &str, selected: bool) -> (bool, bool, Rect) {
    let fill = if selected {
        PALETTE.background_panel
    } else {
        PALETTE.background_editor
    };

    let mut tab_clicked = false;
    let mut close_clicked = false;

    let output = Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_height(TAB_BAR_HEIGHT - 20.0);
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                tab_clicked = ui
                    .add(
                        Button::new(RichText::new(path).size(TAB_LABEL_SIZE)).frame(false),
                    )
                    .clicked();
                close_clicked = ui
                    .add(
                        IconButton::new(Icon::XMARK)
                            .size(TAB_CLOSE_SIZE)
                            .min_size(egui::vec2(18.0, 18.0))
                            .tooltip("Close"),
                    )
                    .clicked();
            });
        });

    (tab_clicked, close_clicked, output.response.rect)
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

fn close_tab(editor: &mut EditorState, index: usize) {
    if index >= editor.tabs.len() {
        return;
    }
    editor.tabs.remove(index);
    if editor.tabs.is_empty() {
        editor.active_tab = 0;
        return;
    }
    if editor.active_tab >= editor.tabs.len() {
        editor.active_tab = editor.tabs.len() - 1;
    } else if index < editor.active_tab {
        editor.active_tab -= 1;
    }
}
