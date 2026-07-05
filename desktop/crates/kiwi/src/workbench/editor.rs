//! Editor area — open tabs and editable file content.

use egui::{pos2, Align, Button, Frame, Layout, Rect, RichText, ScrollArea, Stroke, TextEdit, Ui};
use nest_gui::{ActionButton, ButtonSize};

use crate::theme::PALETTE;
use crate::workbench::editor_diff::colorized_diff_editor;
use crate::workbench::editor_syntax::highlighted_code_editor_with_lines;
use nest_icon::{Icon, IconButton, icons::solid};

const TAB_BAR_HEIGHT: f32 = 38.0;
const TAB_LABEL_SIZE: f32 = 13.0;
const TAB_CLOSE_SIZE: f32 = 11.0;
const EDITOR_TOOLBAR_HEIGHT: f32 = 28.0;
const EDITOR_TOOLBAR_GAP: f32 = 4.0;

/// Drag payload when dropping an editor tab onto the agent panel.
#[derive(Clone, Debug)]
pub struct EditorTabDragPayload {
    /// Path relative to the project root.
    pub rel_path: String,
    /// Current buffer contents.
    pub content: String,
}

/// How the editor tab is presenting a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTabView {
    /// Editable file buffer.
    Source,
    /// Read-only unified git diff.
    Diff {
        /// True when diffing the index (staged) version.
        staged: bool,
    },
    /// Read-only GitHub issue.
    Issue {
        /// GitHub issue number.
        number: u64,
    },
    /// Read-only GitHub pull request.
    PullRequest {
        /// GitHub pull request number.
        number: u64,
    },
    /// Compose a new GitHub issue.
    NewIssue,
}

/// User action requested from the editor panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorPanelRequest {
    /// Save the source tab at this index.
    SaveFile(usize),
    /// Create a GitHub issue from the new-issue tab at this index.
    CreateIssue(usize),
    /// Edit labels and milestone for the issue tab at this index.
    EditIssueMetadata(usize),
    /// Merge the pull request tab at this index.
    MergePullRequest(usize),
}

/// Returns the GitHub issue number for the active editor tab, if any.
pub fn active_issue_number(editor: &EditorState) -> Option<u64> {
    editor.tabs.get(editor.active_tab).and_then(|tab| match tab.view {
        EditorTabView::Issue { number } => Some(number),
        _ => None,
    })
}

/// One open editor tab.
#[derive(Debug, Clone)]
pub struct EditorTab {
    /// Path relative to the project root.
    pub rel_path: String,
    /// Absolute path on disk.
    pub abs_path: std::path::PathBuf,
    /// Current buffer contents.
    pub content: String,
    /// Last saved contents on disk.
    pub saved_content: String,
    /// Whether the buffer differs from the last save.
    pub dirty: bool,
    /// Whether content is still loading.
    pub loading: bool,
    /// Whether a save is in progress.
    pub saving: bool,
    /// Load error message.
    pub error: Option<String>,
    /// Save error message.
    pub save_error: Option<String>,
    /// Source file vs git diff presentation.
    pub view: EditorTabView,
    /// Title field for [`EditorTabView::NewIssue`] tabs.
    pub issue_title: String,
}

impl EditorTab {
    /// Creates a tab with empty buffers and the given presentation mode.
    pub(crate) fn new(rel_path: String, abs_path: std::path::PathBuf, view: EditorTabView) -> Self {
        Self {
            rel_path,
            abs_path,
            content: String::new(),
            saved_content: String::new(),
            dirty: false,
            loading: false,
            saving: false,
            error: None,
            save_error: None,
            view,
            issue_title: String::new(),
        }
    }
}

/// Open editor tabs and active selection.
#[derive(Debug, Clone, Default)]
pub struct EditorState {
    /// Open tabs.
    pub tabs: Vec<EditorTab>,
    /// Index of the active tab.
    pub active_tab: usize,
}

impl EditorState {
    /// Empty editor for a fresh workspace.
    pub fn empty() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
        }
    }
}

/// Renders the editor tab bar and content area.
///
/// Returns a save or create-issue request from toolbar actions or shortcuts.
pub fn editor_panel(
    ui: &mut Ui,
    ctx: &egui::Context,
    editor: &mut EditorState,
) -> Option<EditorPanelRequest> {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.set_min_height(ui.available_height());

    if editor.tabs.is_empty() {
        Frame::new()
            .fill(PALETTE.background_panel)
            .inner_margin(egui::Margin::symmetric(12, 12))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(
                        "Open a file from the explorer, a change in Source Control, or an issue in Issues.",
                    )
                    .weak()
                    .size(14.0),
                );
            });
        return None;
    }

    let save_shortcut = ctx.input(|input| {
        input.modifiers.command_only() && input.key_pressed(egui::Key::S)
    });

    editor_tabs(ui, editor);

    let content_height = ui.available_height().max(0.0);
    let mut panel_request = None;

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

            let active = editor.active_tab;
            let Some(tab) = editor.tabs.get(active) else {
                return;
            };

            if tab.loading {
                ui.label(RichText::new("Loading…").weak().italics().size(13.0));
                return;
            }

            if let Some(error) = &tab.error {
                ui.label(
                    RichText::new(error)
                        .color(ui.visuals().error_fg_color)
                        .size(13.0),
                );
                return;
            }

            match tab.view {
                EditorTabView::Source => {
                    editor_toolbar(ui, tab, active, &mut panel_request);
                    ui.add_space(EDITOR_TOOLBAR_GAP);

                    let rel_path = editor.tabs[active].rel_path.clone();
                    ScrollArea::vertical()
                        .id_salt("kiwi-editor")
                        .max_height(
                            (content_height - EDITOR_TOOLBAR_HEIGHT - EDITOR_TOOLBAR_GAP).max(40.0),
                        )
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let response = highlighted_code_editor_with_lines(
                                ui,
                                &mut editor.tabs[active].content,
                                &rel_path,
                            );
                            if response.changed() {
                                let tab = &mut editor.tabs[active];
                                tab.dirty = tab.content != tab.saved_content;
                                tab.save_error = None;
                            }
                        });
                }
                EditorTabView::Diff { staged } => {
                    diff_toolbar(ui, tab, staged);
                    ui.add_space(EDITOR_TOOLBAR_GAP);

                    ScrollArea::vertical()
                        .id_salt(("kiwi-editor-diff", &tab.rel_path, staged))
                        .max_height(
                            (content_height - EDITOR_TOOLBAR_HEIGHT - EDITOR_TOOLBAR_GAP).max(40.0),
                        )
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            colorized_diff_editor(ui, &mut editor.tabs[active].content);
                        });
                }
                EditorTabView::Issue { number } => {
                    issue_toolbar(ui, tab, number, active, &mut panel_request);
                    ui.add_space(EDITOR_TOOLBAR_GAP);

                    ScrollArea::vertical()
                        .id_salt(("kiwi-editor-issue", &tab.rel_path, number))
                        .max_height(
                            (content_height - EDITOR_TOOLBAR_HEIGHT - EDITOR_TOOLBAR_GAP).max(40.0),
                        )
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut editor.tabs[active].content)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false)
                                    .frame(false),
                            );
                        });
                }
                EditorTabView::PullRequest { number } => {
                    pr_toolbar(ui, tab, number, active, &mut panel_request);
                    ui.add_space(EDITOR_TOOLBAR_GAP);

                    ScrollArea::vertical()
                        .id_salt(("kiwi-editor-pr", &tab.rel_path, number))
                        .max_height(
                            (content_height - EDITOR_TOOLBAR_HEIGHT - EDITOR_TOOLBAR_GAP).max(40.0),
                        )
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut editor.tabs[active].content)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false)
                                    .frame(false),
                            );
                        });
                }
                EditorTabView::NewIssue => {
                    new_issue_form(
                        ui,
                        &mut editor.tabs[active],
                        active,
                        content_height,
                        &mut panel_request,
                    );
                }
            }
        });

    if save_shortcut {
        panel_request = Some(EditorPanelRequest::SaveFile(editor.active_tab));
    }

    panel_request.filter(|request| match request {
        EditorPanelRequest::SaveFile(index) => editor.tabs.get(*index).is_some_and(|tab| {
            tab.view == EditorTabView::Source
                && tab.dirty
                && !tab.loading
                && !tab.saving
                && tab.error.is_none()
        }),
        EditorPanelRequest::CreateIssue(index) => editor.tabs.get(*index).is_some_and(|tab| {
            tab.view == EditorTabView::NewIssue
                && !tab.issue_title.trim().is_empty()
                && !tab.saving
        }),
        EditorPanelRequest::EditIssueMetadata(index) => editor.tabs.get(*index).is_some_and(|tab| {
            matches!(tab.view, EditorTabView::Issue { number: _ }) && !tab.loading
        }),
        EditorPanelRequest::MergePullRequest(index) => editor.tabs.get(*index).is_some_and(|tab| {
            matches!(tab.view, EditorTabView::PullRequest { number: _ }) && !tab.loading
        }),
    })
}

fn new_issue_form(
    ui: &mut Ui,
    tab: &mut EditorTab,
    tab_index: usize,
    content_height: f32,
    panel_request: &mut Option<EditorPanelRequest>,
) {
    new_issue_toolbar(ui, tab, tab_index, panel_request);

    ui.add_space(EDITOR_TOOLBAR_GAP);
    ui.label(RichText::new("Title").strong().size(12.0));
    ui.add_space(2.0);
    ui.add(
        TextEdit::singleline(&mut tab.issue_title)
            .desired_width(f32::INFINITY)
            .hint_text("Issue title"),
    );

    ui.add_space(8.0);
    ui.label(RichText::new("Description").strong().size(12.0));
    ui.add_space(2.0);

    let body_height = (content_height - EDITOR_TOOLBAR_HEIGHT - EDITOR_TOOLBAR_GAP - 88.0).max(120.0);
    ScrollArea::vertical()
        .id_salt(("kiwi-editor-new-issue", &tab.rel_path))
        .max_height(body_height)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add(
                TextEdit::multiline(&mut tab.content)
                    .desired_width(f32::INFINITY)
                    .hint_text("Describe the issue…")
                    .frame(true),
            );
        });
}

fn new_issue_toolbar(
    ui: &mut Ui,
    tab: &EditorTab,
    tab_index: usize,
    panel_request: &mut Option<EditorPanelRequest>,
) {
    let can_create = !tab.issue_title.trim().is_empty() && !tab.saving;

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), EDITOR_TOOLBAR_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            ui.label(
                RichText::new("New GitHub Issue")
                    .strong()
                    .size(12.0)
                    .color(PALETTE.text_secondary),
            );

            let mut create_button = ActionButton::new(Icon::solid(solid::PLUS), "Create Issue")
                .size(ButtonSize::Small)
                .enabled(can_create)
                .tooltip("Create issue on GitHub");

            if can_create {
                create_button = create_button
                    .fill(PALETTE.accent_primary)
                    .text_color(egui::Color32::WHITE);
            }

            if ui.add(create_button).clicked() {
                *panel_request = Some(EditorPanelRequest::CreateIssue(tab_index));
            }

            if tab.saving {
                ui.label(RichText::new("Creating…").weak().italics().size(12.0));
            }

            if let Some(error) = &tab.save_error {
                ui.label(
                    RichText::new(error)
                        .color(ui.visuals().error_fg_color)
                        .size(12.0),
                );
            }

            ui.label(
                RichText::new(&tab.rel_path)
                    .size(12.0)
                    .color(PALETTE.text_muted),
            );
        },
    );
}

fn issue_toolbar(
    ui: &mut Ui,
    tab: &EditorTab,
    number: u64,
    tab_index: usize,
    panel_request: &mut Option<EditorPanelRequest>,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), EDITOR_TOOLBAR_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            ui.label(
                RichText::new(format!("GitHub issue #{number}"))
                    .strong()
                    .size(12.0)
                    .color(PALETTE.text_secondary),
            );

            let labels_button = ActionButton::new(Icon::solid(solid::PEN_TO_SQUARE), "Labels & Milestone")
                .size(ButtonSize::Small)
                .tooltip("Edit labels and milestone");

            if ui.add(labels_button).clicked() {
                *panel_request = Some(EditorPanelRequest::EditIssueMetadata(tab_index));
            }

            ui.label(
                RichText::new(tab.abs_path.display().to_string())
                    .size(12.0)
                    .color(PALETTE.info),
            )
            .on_hover_text("Open in browser");
        },
    );
}

fn pr_toolbar(
    ui: &mut Ui,
    tab: &EditorTab,
    number: u64,
    tab_index: usize,
    panel_request: &mut Option<EditorPanelRequest>,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), EDITOR_TOOLBAR_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            ui.label(
                RichText::new(format!("GitHub pull request #{number}"))
                    .strong()
                    .size(12.0)
                    .color(PALETTE.text_secondary),
            );

            let merge_button = ActionButton::new(Icon::solid(solid::CHECK), "Merge…")
                .size(ButtonSize::Small)
                .tooltip("Merge this pull request into its base branch");

            if ui.add(merge_button).clicked() {
                *panel_request = Some(EditorPanelRequest::MergePullRequest(tab_index));
            }

            ui.label(
                RichText::new(tab.abs_path.display().to_string())
                    .size(12.0)
                    .color(PALETTE.info),
            )
            .on_hover_text("Open in browser");
        },
    );
}

fn diff_toolbar(ui: &mut Ui, tab: &EditorTab, staged: bool) {
    let label = if staged {
        "Git diff (staged)"
    } else {
        "Git diff (working tree)"
    };

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), EDITOR_TOOLBAR_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.label(
                RichText::new(label)
                    .strong()
                    .size(12.0)
                    .color(PALETTE.text_secondary),
            );
            ui.label(
                RichText::new(tab.abs_path.display().to_string())
                    .size(12.0)
                    .color(PALETTE.text_muted),
            )
            .on_hover_text(&tab.rel_path);
        },
    );
}

fn editor_toolbar(
    ui: &mut Ui,
    tab: &EditorTab,
    tab_index: usize,
    panel_request: &mut Option<EditorPanelRequest>,
) {
    let can_save = tab.dirty && !tab.saving;
    let abs_path = tab.abs_path.display().to_string();

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), EDITOR_TOOLBAR_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            let mut save_button = ActionButton::new(Icon::solid(solid::FLOPPY_DISK), "Save")
                .size(ButtonSize::Small)
                .enabled(can_save)
                .tooltip("Save (Ctrl+S)");

            if can_save {
                save_button = save_button
                    .fill(PALETTE.accent_primary)
                    .text_color(egui::Color32::WHITE);
            }

            if ui.add(save_button).clicked() {
                *panel_request = Some(EditorPanelRequest::SaveFile(tab_index));
            }

            if tab.saving {
                ui.label(RichText::new("Saving…").weak().italics().size(12.0));
            }

            if let Some(error) = &tab.save_error {
                ui.label(
                    RichText::new(error)
                        .color(ui.visuals().error_fg_color)
                        .size(12.0),
                );
            }

            ui.label(
                RichText::new(abs_path)
                    .size(12.0)
                    .color(PALETTE.text_muted),
            )
            .on_hover_text(tab.rel_path.as_str());
        },
    );
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

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.set_min_height(TAB_BAR_HEIGHT);

                for (index, tab) in editor.tabs.iter().enumerate() {
                    let selected = editor.active_tab == index;
                    let (tab_clicked, close_clicked, tab_rect) =
                        tab_item(ui, tab, selected);
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

fn tab_file_name(rel_path: &str) -> &str {
    std::path::Path::new(rel_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(rel_path)
}

fn tab_label(tab: &EditorTab) -> String {
    match tab.view {
        EditorTabView::Source => {
            let name = tab_file_name(&tab.rel_path);
            if tab.dirty {
                format!("* {name}")
            } else {
                name.to_string()
            }
        }
        EditorTabView::Diff { .. } => format!("{} (diff)", tab_file_name(&tab.rel_path)),
        EditorTabView::Issue { number } => format!("#{number}"),
        EditorTabView::PullRequest { number } => format!("PR #{number}"),
        EditorTabView::NewIssue => "New Issue".into(),
    }
}

fn tab_item(ui: &mut Ui, tab: &EditorTab, selected: bool) -> (bool, bool, Rect) {
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

                if tab.loading {
                    tab_clicked = tab_label_button(ui, tab, selected).clicked();
                } else {
                    let payload = EditorTabDragPayload {
                        rel_path: tab.rel_path.clone(),
                        content: tab.content.clone(),
                    };
                    let drag_id = ui.id().with(("editor_tab_dnd", &tab.rel_path));
                    let dnd = ui.dnd_drag_source(drag_id, payload, |ui| {
                        tab_label_button(ui, tab, selected)
                    });
                    tab_clicked = dnd.response.clicked();
                }

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

fn tab_label_button(ui: &mut Ui, tab: &EditorTab, _selected: bool) -> egui::Response {
    ui.add(
        Button::new(RichText::new(tab_label(tab)).size(TAB_LABEL_SIZE).monospace())
            .frame(false),
    )
    .on_hover_text(format!(
        "{}\n{}\n\nDrag to Agent panel to attach file contents.",
        tab.rel_path,
        tab.abs_path.display()
    ))
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
