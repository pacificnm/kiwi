//! Issues sidebar — GitHub open issues list.

use egui::{Align, Label, Layout, Response, RichText, ScrollArea, Sense, Ui};

use nest_core::AppContext;
use nest_http_client::HttpClientService;

use crate::project::ProjectConfig;
use crate::theme::{context_menu, PALETTE};
use crate::workbench::editor::EditorState;
use crate::workbench::editor_files::{open_issue_tab, open_pr_tab};
use crate::workbench::issues::{IssuesSidebarView, IssuesState, read_github_repo, PullRequestListItem};
use crate::workbench::sidebar::panel_width;
use crate::workbench::source_control::SourceControlState;
use crate::workbench::FileLoadPending;

const ROW_HEIGHT: f32 = 28.0;
const NUMBER_COLUMN_WIDTH: f32 = 52.0;
const ROW_ITEM_GAP: f32 = 6.0;

/// Renders the GitHub issues list for the current repository.
pub fn show(
    ui: &mut Ui,
    issues: &mut IssuesState,
    source: &mut SourceControlState,
    project: &ProjectConfig,
    app_ctx: &AppContext,
    editor: &mut EditorState,
    file_pending: &mut Option<FileLoadPending>,
) {
    toolbar(ui, issues, project.root.as_path(), app_ctx);
    view_selector(ui, issues);

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

    if issues.sidebar_view == IssuesSidebarView::Issues {
        show_issues_list(ui, issues, source, project, app_ctx, editor, file_pending);
    } else {
        show_pull_requests_list(ui, issues, project, app_ctx, editor);
    }
}

fn view_selector(ui: &mut Ui, issues: &mut IssuesState) {
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut issues.sidebar_view,
            IssuesSidebarView::Issues,
            "Issues",
        );
        ui.selectable_value(
            &mut issues.sidebar_view,
            IssuesSidebarView::PullRequests,
            "Pull Requests",
        );
    });
    ui.add_space(4.0);
}

fn show_issues_list(
    ui: &mut Ui,
    issues: &mut IssuesState,
    source: &mut SourceControlState,
    project: &ProjectConfig,
    app_ctx: &AppContext,
    editor: &mut EditorState,
    file_pending: &mut Option<FileLoadPending>,
) {
    if issues.loading && issues.issues.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new("Loading issues…").weak().size(12.0));
        return;
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
    let issue_items = issues.issues.clone();
    ScrollArea::vertical()
        .id_salt((
            "kiwi-issues-list",
            project.root.display().to_string(),
        ))
        .auto_shrink([false; 2])
        .max_height(list_height)
        .show(ui, |ui| {
            ui.set_min_width(panel_width(ui));
            for item in &issue_items {
                issue_row(ui, item, issues, source, project, app_ctx, &mut open_number);
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

fn show_pull_requests_list(
    ui: &mut Ui,
    issues: &mut IssuesState,
    project: &ProjectConfig,
    app_ctx: &AppContext,
    editor: &mut EditorState,
) {
    if issues.pr_loading && issues.pull_requests.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new("Loading pull requests…").weak().size(12.0));
        return;
    }

    if issues.pull_requests.is_empty() && !issues.pr_loading {
        ui.label(
            RichText::new("No open pull requests.")
                .weak()
                .size(12.0),
        );
        return;
    }

    let mut open_number = None;
    let list_height = ui.available_height().max(80.0);
    let pr_items = issues.pull_requests.clone();
    ScrollArea::vertical()
        .id_salt((
            "kiwi-pr-list",
            project.root.display().to_string(),
        ))
        .auto_shrink([false; 2])
        .max_height(list_height)
        .show(ui, |ui| {
            ui.set_min_width(panel_width(ui));
            for item in &pr_items {
                pr_row(ui, item, issues, &mut open_number);
            }
        });

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
    let Some(item) = issues.pull_requests.iter().find(|pr| pr.number == number) else {
        return;
    };

    let Ok(http) = app_ctx.service::<HttpClientService>() else {
        issues.error = Some("HTTP client is not configured".into());
        return;
    };

    if let Some(tab_index) = open_pr_tab(
        editor,
        &owner,
        &repo,
        number,
        item.html_url.clone(),
    ) {
        issues.request_open_pr(&project.root, number, tab_index, http.clone());
    }
}

fn toolbar(ui: &mut Ui, issues: &mut IssuesState, root: &std::path::Path, app_ctx: &AppContext) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("GitHub").weak().size(11.0));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add_enabled(!issues.busy(), egui::Button::new(RichText::new("Refresh").size(11.0)))
                .clicked()
            {
                issues.sync_gh_auth();
                if let Ok(http) = app_ctx.service::<HttpClientService>() {
                    issues.request_list(root, http.clone());
                    issues.request_pr_list(root, http.clone());
                } else {
                    issues.error = Some("HTTP client is not configured".into());
                }
            }
        });
    });
}

fn issue_row(
    ui: &mut Ui,
    item: &crate::workbench::issues::IssueListItem,
    issues: &mut IssuesState,
    source: &mut SourceControlState,
    project: &ProjectConfig,
    app_ctx: &AppContext,
    open: &mut Option<u64>,
) {
    full_width_row(ui, ROW_HEIGHT, |ui, width| {
        ui.spacing_mut().item_spacing.x = ROW_ITEM_GAP;

        let number_response = issue_number_cell(ui, item, open);
        let title_width = (width - NUMBER_COLUMN_WIDTH - ROW_ITEM_GAP).max(40.0);
        let title_response = issue_title_cell(ui, item, title_width, open);

        context_menu(&number_response.union(title_response), |ui| {
            if ui.button("Labels & Milestone…").clicked() {
                issues.open_issue_metadata(item.number);
                ui.close_menu();
            }
            if ui.button("New Comment…").clicked() {
                issues.open_new_comment(Some(item.number));
                ui.close_menu();
            }
            if ui.button("Create Branch from Issue…").clicked() {
                source.request_branch_list(&project.root);
                source.open_create_branch_from_issue(item.number, &item.title);
                ui.close_menu();
            }
            if ui.button("Send Issue to Agent…").clicked() {
                if let Ok(http) = app_ctx.service::<HttpClientService>() {
                    issues.request_send_to_agent(&project.root, item.number, http.clone());
                } else {
                    issues.error = Some("HTTP client is not configured".into());
                }
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Manage Labels…").clicked() {
                issues.open_manage_labels();
                ui.close_menu();
            }
            if ui.button("Manage Milestones…").clicked() {
                issues.open_manage_milestones();
                ui.close_menu();
            }
        });
    });
}

fn pr_row(
    ui: &mut Ui,
    item: &PullRequestListItem,
    issues: &mut IssuesState,
    open: &mut Option<u64>,
) {
    full_width_row(ui, ROW_HEIGHT, |ui, width| {
        ui.spacing_mut().item_spacing.x = ROW_ITEM_GAP;

        let branch_hint = format!(
            "{} → {} · @{}",
            item.head_branch, item.base_branch, item.author
        );
        let number_response = left_cell(ui, egui::vec2(NUMBER_COLUMN_WIDTH, ROW_HEIGHT), |ui| {
            let response = ui
                .add(
                    Label::new(
                        RichText::new(format!("#{}", item.number))
                            .monospace()
                            .size(11.0)
                            .color(PALETTE.info),
                    )
                    .halign(Align::LEFT)
                    .sense(Sense::click()),
                )
                .on_hover_text(&branch_hint);
            if response.clicked() {
                *open = Some(item.number);
            }
            response
        });

        let title_width = (width - NUMBER_COLUMN_WIDTH - ROW_ITEM_GAP).max(40.0);
        let title_response = left_cell(ui, egui::vec2(title_width, ROW_HEIGHT), |ui| {
            let mut title = item.title.clone();
            if item.draft {
                title = format!("Draft: {title}");
            }
            let response = ui
                .add(
                    Label::new(RichText::new(title).size(12.0))
                        .truncate()
                        .halign(Align::LEFT)
                        .sense(Sense::click()),
                )
                .on_hover_text(&branch_hint);
            if response.clicked() {
                *open = Some(item.number);
            }
            response
        });

        context_menu(&number_response.union(title_response), |ui| {
            if ui.button("Merge Pull Request…").clicked() {
                issues.open_merge_pr(item.number);
                ui.close_menu();
            }
        });
    });
}

fn issue_number_cell(
    ui: &mut Ui,
    item: &crate::workbench::issues::IssueListItem,
    open: &mut Option<u64>,
) -> Response {
    left_cell(ui, egui::vec2(NUMBER_COLUMN_WIDTH, ROW_HEIGHT), |ui| {
        let response = ui
            .add(
                Label::new(
                    RichText::new(format!("#{}", item.number))
                        .monospace()
                        .size(11.0)
                        .color(PALETTE.info),
                )
                .halign(Align::LEFT)
                .sense(Sense::click()),
            )
            .on_hover_text(format!("@{} · Open in editor", item.author));
        if response.clicked() {
            *open = Some(item.number);
        }
        response
    })
}

fn issue_title_cell(
    ui: &mut Ui,
    item: &crate::workbench::issues::IssueListItem,
    width: f32,
    open: &mut Option<u64>,
) -> Response {
    left_cell(ui, egui::vec2(width, ROW_HEIGHT), |ui| {
        let response = ui
            .add(
                Label::new(RichText::new(&item.title).size(12.0))
                    .truncate()
                    .halign(Align::LEFT)
                    .sense(Sense::click()),
            )
            .on_hover_text(format!("@{} · Open in editor", item.author));
        if response.clicked() {
            *open = Some(item.number);
        }
        response
    })
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
