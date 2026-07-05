//! Issues sidebar — GitHub open issues list.

use egui::{Align, Label, Layout, RichText, ScrollArea, Sense, Ui};

use nest_core::AppContext;
use nest_http_client::HttpClientService;

use crate::project::ProjectConfig;
use crate::theme::PALETTE;
use crate::workbench::editor::EditorState;
use crate::workbench::editor_files::open_issue_tab;
use crate::workbench::issues::{IssuesState, read_github_repo};
use crate::workbench::sidebar::panel_width;
use crate::workbench::FileLoadPending;

const ROW_HEIGHT: f32 = 28.0;
const NUMBER_COLUMN_WIDTH: f32 = 52.0;
const ROW_ITEM_GAP: f32 = 6.0;

/// Renders the GitHub issues list for the current repository.
pub fn show(
    ui: &mut Ui,
    issues: &mut IssuesState,
    project: &ProjectConfig,
    app_ctx: &AppContext,
    editor: &mut EditorState,
    file_pending: &mut Option<FileLoadPending>,
) {
    toolbar(ui, issues, project.root.as_path(), app_ctx);

    if let Some(login) = &issues.auth_login {
        ui.label(
            RichText::new(format!("Signed in as @{login}"))
                .size(11.0)
                .color(PALETTE.info),
        );
        ui.add_space(4.0);
    } else if !issues.is_authenticated() {
        ui.label(
            RichText::new("Not signed in — run `gh auth login` in a terminal")
                .size(11.0)
                .color(ui.visuals().warn_fg_color),
        );
        ui.add_space(4.0);
    }

    if issues.loading && issues.issues.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new("Loading issues…").weak().size(12.0));
        return;
    }

    if let Some(error) = &issues.error {
        ui.add_space(4.0);
        ui.label(
            RichText::new(error)
                .color(ui.visuals().error_fg_color)
                .size(11.0),
        );
        ui.add_space(4.0);
    }

    if let Some((owner, repo)) = &issues.repo {
        ui.label(
            RichText::new(format!("{owner}/{repo}"))
                .weak()
                .size(11.0),
        );
        ui.add_space(4.0);
    }

    if issues.issues.is_empty() && !issues.loading {
        ui.label(
            RichText::new("No open issues.")
                .weak()
                .size(12.0),
        );
        return;
    }

    let mut open_number = None;
    let list_height = ui.available_height().max(80.0);
    ScrollArea::vertical()
        .id_salt((
            "kiwi-issues-list",
            project.root.display().to_string(),
        ))
        .auto_shrink([false; 2])
        .max_height(list_height)
        .show(ui, |ui| {
            ui.set_min_width(panel_width(ui));
            for item in &issues.issues {
                issue_row(ui, item, &mut open_number);
            }
        });

    if file_pending.is_some() {
        return;
    }

    let Some(number) = open_number else {
        return;
    };
    let Some((owner, repo)) = issues
        .repo
        .clone()
        .or_else(|| read_github_repo(project.root.as_path()).ok())
    else {
        issues.error = Some("Could not resolve GitHub repository from origin".into());
        return;
    };
    let Some(item) = issues.issues.iter().find(|issue| issue.number == number) else {
        return;
    };

    let Ok(http) = app_ctx.service::<HttpClientService>() else {
        issues.error = Some("HTTP client is not configured".into());
        return;
    };

    if let Some(tab_index) = open_issue_tab(
        editor,
        &owner,
        &repo,
        number,
        item.html_url.clone(),
    ) {
        *file_pending = Some(issues.spawn_open_issue(
            &owner,
            &repo,
            number,
            tab_index,
            http.clone(),
        ));
    }
}

fn toolbar(ui: &mut Ui, issues: &mut IssuesState, root: &std::path::Path, app_ctx: &AppContext) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Open issues").weak().size(11.0));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add_enabled(!issues.busy(), egui::Button::new(RichText::new("Refresh").size(11.0)))
                .clicked()
            {
                issues.sync_gh_auth();
                if let Ok(http) = app_ctx.service::<HttpClientService>() {
                    issues.request_list(root, http.clone());
                } else {
                    issues.error = Some("HTTP client is not configured".into());
                }
            }
        });
    });
}

fn issue_row(ui: &mut Ui, item: &crate::workbench::issues::IssueListItem, open: &mut Option<u64>) {
    full_width_row(ui, ROW_HEIGHT, |ui, width| {
        ui.spacing_mut().item_spacing.x = ROW_ITEM_GAP;

        left_cell(ui, egui::vec2(NUMBER_COLUMN_WIDTH, ROW_HEIGHT), |ui| {
            let response = ui.add(
                Label::new(
                    RichText::new(format!("#{}", item.number))
                        .monospace()
                        .size(11.0)
                        .color(PALETTE.info),
                )
                .halign(Align::LEFT)
                .sense(Sense::click()),
            );
            if response.clicked() {
                *open = Some(item.number);
            }
            response.on_hover_text(format!("@{} · Open in editor", item.author));
        });

        let title_width = (width - NUMBER_COLUMN_WIDTH - ROW_ITEM_GAP).max(40.0);
        left_cell(ui, egui::vec2(title_width, ROW_HEIGHT), |ui| {
            let response = ui.add(
                Label::new(RichText::new(&item.title).size(12.0))
                    .truncate()
                    .halign(Align::LEFT)
                    .sense(Sense::click()),
            );
            if response.clicked() {
                *open = Some(item.number);
            }
            response.on_hover_text(format!("@{} · Open in editor", item.author));
        });
    });
}

/// Left-aligned cell. Never use [`Ui::add_sized`] for list rows — it centers content.
fn left_cell<R>(ui: &mut Ui, size: egui::Vec2, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        ui.set_width(size.x);
        add_contents(ui)
    })
    .inner
}

fn full_width_row<R>(ui: &mut Ui, height: f32, add_contents: impl FnOnce(&mut Ui, f32) -> R) -> R {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, height),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_width(width);
            add_contents(ui, width)
        },
    )
    .inner
}
