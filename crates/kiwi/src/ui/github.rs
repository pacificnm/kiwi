use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::github::{
    issue_at_viewport, issue_selected_row_index, pr_at_viewport, pr_selected_row_index,
    GitHubAuthErrorKind, GitHubLeftPane, Issue, IssueState, PrState, PullRequest,
};
use crate::selection::{line_spans_with_selection, SelectionPane};
use crate::state::AppState;
use crate::theme::SemanticRole;
use crate::theme::ThemePalette;

use super::scrollbar::{render_vertical_scrollbar, split_for_scrollbar};
use super::tabs::separator_span;

const GH_HUB_ROWS: u16 = 1;
const ISSUES_STATUS_ROWS: u16 = 1;
const ISSUE_DETAIL_STATUS_ROWS: u16 = 1;

pub fn issues_viewport_rows(area: Rect) -> usize {
    pane_inner(area)
        .map(|inner| {
            inner
                .height
                .saturating_sub(ISSUES_STATUS_ROWS + GH_HUB_ROWS) as usize
        })
        .unwrap_or(0)
}

pub fn issue_detail_viewport_rows(area: Rect) -> usize {
    pane_inner(area)
        .map(|inner| inner.height.saturating_sub(ISSUE_DETAIL_STATUS_ROWS) as usize)
        .unwrap_or(0)
}

pub fn pr_detail_viewport_rows(area: Rect) -> usize {
    issue_detail_viewport_rows(area)
}

pub fn prs_viewport_rows(area: Rect) -> usize {
    issues_viewport_rows(area)
}

pub fn github_pr_interaction_at(
    state: &AppState,
    area: Rect,
    column: u16,
    row: u16,
) -> Option<usize> {
    if state.github.left_pane != GitHubLeftPane::Prs {
        return None;
    }

    if state.github.loading && !state.github.auth_checked {
        return None;
    }

    if github_auth_message(state).is_some() {
        return None;
    }

    if area.width == 0 || area.height == 0 {
        return None;
    }

    if column < area.x
        || column >= area.x.saturating_add(area.width)
        || row < area.y
        || row >= area.y.saturating_add(area.height)
    {
        return None;
    }

    let list_area = github_issues_list_area(area)?;
    if column < list_area.x
        || column >= list_area.x.saturating_add(list_area.width)
        || row < list_area.y
        || row >= list_area.y.saturating_add(list_area.height)
    {
        return None;
    }

    let viewport_index = usize::from(row.saturating_sub(list_area.y));
    pr_at_viewport(&state.github, viewport_index)?;

    Some(
        state
            .github
            .prs_scroll_offset
            .saturating_add(viewport_index),
    )
}

pub fn github_issue_interaction_at(
    state: &AppState,
    area: Rect,
    column: u16,
    row: u16,
) -> Option<usize> {
    if state.github.left_pane != GitHubLeftPane::Issues {
        return None;
    }

    if state.github.loading && !state.github.auth_checked {
        return None;
    }

    if github_auth_message(state).is_some() {
        return None;
    }

    if area.width == 0 || area.height == 0 {
        return None;
    }

    if column < area.x
        || column >= area.x.saturating_add(area.width)
        || row < area.y
        || row >= area.y.saturating_add(area.height)
    {
        return None;
    }

    let list_area = github_issues_list_area(area)?;
    if column < list_area.x
        || column >= list_area.x.saturating_add(list_area.width)
        || row < list_area.y
        || row >= list_area.y.saturating_add(list_area.height)
    {
        return None;
    }

    let viewport_index = usize::from(row.saturating_sub(list_area.y));
    issue_at_viewport(&state.github, viewport_index)?;

    Some(
        state
            .github
            .issues_scroll_offset
            .saturating_add(viewport_index),
    )
}

fn github_issues_list_area(area: Rect) -> Option<Rect> {
    let inner = pane_inner(area)?;
    let status_y = inner
        .y
        .saturating_add(inner.height.saturating_sub(ISSUES_STATUS_ROWS));
    let status_area = Rect {
        x: inner.x,
        y: status_y,
        width: inner.width,
        height: ISSUES_STATUS_ROWS.min(inner.height),
    };
    let hub_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: GH_HUB_ROWS.min(inner.height.saturating_sub(status_area.height)),
    };

    Some(Rect {
        x: inner.x,
        y: hub_area.y.saturating_add(hub_area.height),
        width: inner.width,
        height: inner
            .height
            .saturating_sub(hub_area.height + status_area.height),
    })
}

pub fn render_github_left_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if state.github.loading && !state.github.auth_checked {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "GitHub",
            vec![Line::from("Checking GitHub authentication…")],
        );
        return;
    }

    if let Some(message) = github_auth_message(state) {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "GitHub",
            auth_error_lines(message, state.github.error_kind),
        );
        return;
    }

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(github_left_title(state))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let Some(inner) = pane_inner(area) else {
        return;
    };

    let status_y = inner
        .y
        .saturating_add(inner.height.saturating_sub(ISSUES_STATUS_ROWS));
    let status_area = Rect {
        x: inner.x,
        y: status_y,
        width: inner.width,
        height: ISSUES_STATUS_ROWS.min(inner.height),
    };
    let Some(list_area) = github_issues_list_area(area) else {
        return;
    };
    let hub_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: list_area.y.saturating_sub(inner.y),
    };

    if hub_area.height > 0 && hub_area.width > 0 {
        render_github_hub_line(frame, hub_area, focused, theme, state);
    }

    if list_area.height > 0 && list_area.width > 0 {
        match state.github.left_pane {
            GitHubLeftPane::Issues => {
                render_issue_list(frame, list_area, focused, theme, state);
            }
            GitHubLeftPane::Prs => render_pr_list(frame, list_area, focused, theme, state),
        }
    }

    if status_area.height > 0 {
        match state.github.left_pane {
            GitHubLeftPane::Issues => {
                render_issues_list_status_line(frame, status_area, state, theme);
            }
            GitHubLeftPane::Prs => render_prs_list_status_line(frame, status_area, state, theme),
        }
    }
}

pub fn render_issue_detail_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if state.github.loading && !state.github.auth_checked {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "Issues",
            vec![Line::from("Checking GitHub authentication…")],
        );
        return;
    }

    if let Some(message) = github_auth_message(state) {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "Issues",
            auth_error_lines(message, state.github.error_kind),
        );
        return;
    }

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(issue_detail_title(state))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let Some(inner) = pane_inner(area) else {
        return;
    };

    let status_y = inner
        .y
        .saturating_add(inner.height.saturating_sub(ISSUE_DETAIL_STATUS_ROWS));
    let status_area = Rect {
        x: inner.x,
        y: status_y,
        width: inner.width,
        height: ISSUE_DETAIL_STATUS_ROWS.min(inner.height),
    };
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(status_area.height),
    };

    if content_area.height > 0 && content_area.width > 0 {
        render_issue_detail_content(frame, content_area, focused, theme, state);
    }

    if status_area.height > 0 {
        render_issue_detail_status_line(frame, status_area, focused, state, theme);
    }
}

pub fn render_pr_detail_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if state.github.loading && !state.github.auth_checked {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "PRs",
            vec![Line::from("Checking GitHub authentication…")],
        );
        return;
    }

    if let Some(message) = github_auth_message(state) {
        render_github_pane(
            frame,
            area,
            focused,
            theme,
            state,
            "PRs",
            auth_error_lines(message, state.github.error_kind),
        );
        return;
    }

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(pr_detail_title(state))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let Some(inner) = pane_inner(area) else {
        return;
    };

    let status_y = inner
        .y
        .saturating_add(inner.height.saturating_sub(ISSUE_DETAIL_STATUS_ROWS));
    let status_area = Rect {
        x: inner.x,
        y: status_y,
        width: inner.width,
        height: ISSUE_DETAIL_STATUS_ROWS.min(inner.height),
    };
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(status_area.height),
    };

    if content_area.height > 0 && content_area.width > 0 {
        render_pr_detail_content(frame, content_area, focused, theme, state);
    }

    if status_area.height > 0 {
        render_pr_detail_status_line(frame, status_area, focused, state, theme);
    }
}

fn github_left_title(state: &AppState) -> String {
    title_with_auth_status("GitHub", state)
}

fn github_hub_label(state: &AppState, pane: GitHubLeftPane) -> String {
    match pane {
        GitHubLeftPane::Issues => {
            let mut label = String::from("Issues");
            if state.github.issues_loading {
                label.push_str(" …");
            } else if !state.github.issues.is_empty() {
                label.push_str(&format!(" · {}", state.github.issues.len()));
            }
            label
        }
        GitHubLeftPane::Prs => {
            let mut label = String::from("PRs");
            if state.github.prs_loading {
                label.push_str(" …");
            } else if !state.github.prs.is_empty() {
                label.push_str(&format!(" · {}", state.github.prs.len()));
            }
            label
        }
    }
}

fn render_github_hub_line(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let selected = state.github.left_pane.index();
    let mut spans = Vec::new();

    for (index, pane) in [GitHubLeftPane::Issues, GitHubLeftPane::Prs]
        .iter()
        .enumerate()
    {
        if index > 0 {
            spans.push(separator_span(theme));
        }

        let active = focused && index == selected;
        let mut style = if active {
            theme.get(SemanticRole::Accent).add_modifier(Modifier::BOLD)
        } else {
            theme.get(SemanticRole::Muted)
        };
        if active {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        spans.push(Span::styled(github_hub_label(state, *pane), style));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_pr_list(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let (content, scrollbar) = split_for_scrollbar(area);
    let viewport_rows = content.height as usize;
    let max_width = content.width as usize;
    let total_rows = state.github.prs.len();
    let selected_row = pr_selected_row_index(&state.github);
    let mut lines = Vec::new();

    for viewport_index in 0..viewport_rows {
        let Some(pr) = pr_at_viewport(&state.github, viewport_index) else {
            break;
        };
        let row_index = state.github.prs_scroll_offset + viewport_index;
        lines.push(render_pr_line(
            pr,
            row_index,
            selected_row,
            max_width,
            focused,
            theme,
        ));
    }

    if lines.is_empty() {
        let hint = if state.github.prs_loading {
            "Loading pull requests…"
        } else if let Some(error) = &state.github.prs_error {
            error.as_str()
        } else {
            "No open pull requests · R refresh"
        };
        lines.push(Line::from(Span::styled(
            truncate_line(hint, max_width),
            theme.get(SemanticRole::Muted),
        )));
    }

    frame.render_widget(Paragraph::new(lines), content);
    if let Some(scrollbar_area) = scrollbar {
        render_vertical_scrollbar(
            frame,
            scrollbar_area,
            state.github.prs_scroll_offset,
            total_rows,
            viewport_rows,
            focused,
            theme,
        );
    }
}

fn render_pr_line(
    pr: &PullRequest,
    row_index: usize,
    selected_row: Option<usize>,
    max_width: usize,
    focused: bool,
    theme: &ThemePalette,
) -> Line<'static> {
    let selected = focused && selected_row == Some(row_index);
    let state_style = match pr.state {
        PrState::Open => theme.get(SemanticRole::PrOpen),
        PrState::Draft => theme.get(SemanticRole::PrDraft),
        PrState::Merged => theme.get(SemanticRole::PrMerged),
        PrState::Closed => theme.get(SemanticRole::PrClosed),
    };

    let mut spans = vec![
        Span::styled(format!("#{} ", pr.number), state_style),
        Span::styled(
            truncate_line(&pr.title, max_width.saturating_sub(8)),
            if selected {
                theme.get(SemanticRole::Accent)
            } else {
                theme.get(SemanticRole::Fg)
            },
        ),
    ];

    if pr.state != PrState::Open {
        spans.push(Span::styled(
            format!(" [{}]", pr.state.label()),
            state_style,
        ));
    }

    let mut line = Line::from(spans);
    if selected {
        line = line.style(Style::default().add_modifier(Modifier::BOLD));
    }
    line
}

fn render_prs_list_status_line(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &ThemePalette,
) {
    let status = if state.github.prs_loading {
        "Loading pull requests…"
    } else if state.github.prs_error.is_some() {
        "j/k move · R refresh"
    } else if state.github.prs.is_empty() {
        "R refresh"
    } else {
        "j/k move · Enter view · i/p switch · R refresh"
    };

    frame.render_widget(
        Paragraph::new(truncate_line(status, area.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn pr_detail_title(state: &AppState) -> String {
    let mut title = String::from("PRs");
    if state.github.pr_detail_loading {
        title.push_str(" · loading");
    } else if state.github.pr_detail_error.is_some() {
        title.push_str(" · error");
    } else if let Some(number) = state.github.selected_pr {
        title.push_str(&format!(" · #{number}"));
    }
    title
}

fn render_pr_detail_content(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let (content, scrollbar) = split_for_scrollbar(area);
    let viewport_rows = content.height as usize;
    let max_width = content.width as usize;

    if state.github.selected_pr.is_none() {
        render_issue_detail_message(
            frame,
            content,
            theme,
            &[
                "No pull request selected",
                "",
                "Select a PR in the GH left panel (Alt+4) and press Enter.",
            ],
        );
        return;
    }

    if state.github.pr_detail_loading && state.github.pr_detail.is_none() {
        render_issue_detail_message(frame, content, theme, &["Loading pull request detail…"]);
        return;
    }

    if let Some(error) = &state.github.pr_detail_error {
        render_issue_detail_message(frame, content, theme, &[error.as_str()]);
        return;
    }

    let Some(detail) = &state.github.pr_detail else {
        render_issue_detail_message(
            frame,
            content,
            theme,
            &[
                "Pull request detail not loaded",
                "",
                "Press Enter on a PR in the GH left panel.",
            ],
        );
        return;
    };

    let start = state.github.pr_detail_scroll_offset;
    let end = (start + viewport_rows).min(detail.display_lines.len());
    let fg = theme.get(SemanticRole::Fg);

    for (row, line_index) in (start..end).enumerate() {
        let line_text = &detail.display_lines[line_index];
        let display = truncate_line(line_text, max_width);
        let style = if line_index == 0 {
            Style::default().add_modifier(Modifier::BOLD)
        } else if line_index == 1 {
            pr_state_style(detail.state, theme)
        } else {
            fg
        };
        let line = line_spans_with_selection(
            &display,
            line_index,
            SelectionPane::PrDetail,
            &state.text_selection,
            style,
            theme,
        );

        let row_area = Rect {
            x: content.x,
            y: content.y.saturating_add(row as u16),
            width: content.width,
            height: 1,
        };
        frame.render_widget(Clear, row_area);
        frame.render_widget(Paragraph::new(line), row_area);
    }

    if let Some(scrollbar_area) = scrollbar {
        render_vertical_scrollbar(
            frame,
            scrollbar_area,
            start,
            detail.display_lines.len(),
            viewport_rows,
            focused,
            theme,
        );
    }
}

fn pr_state_style(state: PrState, theme: &ThemePalette) -> Style {
    match state {
        PrState::Open => theme.get(SemanticRole::PrOpen),
        PrState::Draft => theme.get(SemanticRole::PrDraft),
        PrState::Merged => theme.get(SemanticRole::PrMerged),
        PrState::Closed => theme.get(SemanticRole::PrClosed),
    }
}

fn render_pr_detail_status_line(
    frame: &mut Frame<'_>,
    area: Rect,
    _focused: bool,
    state: &AppState,
    theme: &ThemePalette,
) {
    let status = if state.github.pr_detail_loading {
        "Loading detail…"
    } else if state.github.selected_pr.is_none() {
        "Enter on GH left list to view"
    } else if state.github.pr_detail.is_some() {
        "j/k scroll · drag select · Ctrl+C · o browser · R refresh"
    } else {
        "Enter on GH left list · R refresh"
    };

    frame.render_widget(
        Paragraph::new(truncate_line(status, area.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn issue_detail_title(state: &AppState) -> String {
    let mut title = String::from("Issues");
    if state.github.issue_detail_loading {
        title.push_str(" · loading");
    } else if state.github.issue_detail_error.is_some() {
        title.push_str(" · error");
    } else if let Some(number) = state.github.selected_issue {
        title.push_str(&format!(" · #{number}"));
    }
    title
}

fn render_issue_detail_content(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let (content, scrollbar) = split_for_scrollbar(area);
    let viewport_rows = content.height as usize;
    let max_width = content.width as usize;

    if state.github.selected_issue.is_none() {
        render_issue_detail_message(
            frame,
            content,
            theme,
            &[
                "No issue selected",
                "",
                "Select an issue in the GH left panel (Alt+4) and press Enter.",
            ],
        );
        return;
    }

    if state.github.issue_detail_loading && state.github.issue_detail.is_none() {
        render_issue_detail_message(frame, content, theme, &["Loading issue detail…"]);
        return;
    }

    if let Some(error) = &state.github.issue_detail_error {
        render_issue_detail_message(frame, content, theme, &[error.as_str()]);
        return;
    }

    let Some(detail) = &state.github.issue_detail else {
        render_issue_detail_message(
            frame,
            content,
            theme,
            &[
                "Issue detail not loaded",
                "",
                "Press Enter on an issue in the GH left panel.",
            ],
        );
        return;
    };

    let start = state.github.issue_detail_scroll_offset;
    let end = (start + viewport_rows).min(detail.display_lines.len());
    let fg = theme.get(SemanticRole::Fg);

    for (row, line_index) in (start..end).enumerate() {
        let line_text = &detail.display_lines[line_index];
        let display = truncate_line(line_text, max_width);
        let style = if line_index == 0 {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            fg
        };
        let line = line_spans_with_selection(
            &display,
            line_index,
            SelectionPane::IssueDetail,
            &state.text_selection,
            style,
            theme,
        );

        let row_area = Rect {
            x: content.x,
            y: content.y.saturating_add(row as u16),
            width: content.width,
            height: 1,
        };
        frame.render_widget(Clear, row_area);
        frame.render_widget(Paragraph::new(line), row_area);
    }

    if let Some(scrollbar_area) = scrollbar {
        render_vertical_scrollbar(
            frame,
            scrollbar_area,
            start,
            detail.display_lines.len(),
            viewport_rows,
            focused,
            theme,
        );
    }
}

fn render_issue_detail_message(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &ThemePalette,
    lines: &[&str],
) {
    frame.render_widget(
        Paragraph::new(
            lines
                .iter()
                .map(|line| Line::from((*line).to_string()))
                .collect::<Vec<_>>(),
        )
        .style(theme.get(SemanticRole::Fg))
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_issue_detail_status_line(
    frame: &mut Frame<'_>,
    area: Rect,
    _focused: bool,
    state: &AppState,
    theme: &ThemePalette,
) {
    let status = if let Some(message) = &state.github.issue_action_message {
        message.as_str()
    } else if state.github.issue_detail_loading {
        "Loading detail…"
    } else if state.github.selected_issue.is_none() {
        "Enter on GH left list to view"
    } else if state.github.issue_detail.is_some() {
        "j/k scroll · drag select · Ctrl+C · o browser · palette: comment/label · R refresh"
    } else {
        "Enter on GH left list · R refresh"
    };

    frame.render_widget(
        Paragraph::new(truncate_line(status, area.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn render_issue_list(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
) {
    let (content, scrollbar) = split_for_scrollbar(area);
    let viewport_rows = content.height as usize;
    let max_width = content.width as usize;
    let total_rows = state.github.issues.len();
    let selected_row = issue_selected_row_index(&state.github);
    let mut lines = Vec::new();

    for viewport_index in 0..viewport_rows {
        let Some(issue) = issue_at_viewport(&state.github, viewport_index) else {
            break;
        };
        let row_index = state.github.issues_scroll_offset + viewport_index;
        lines.push(render_issue_line(
            issue,
            row_index,
            selected_row,
            max_width,
            focused,
            theme,
        ));
    }

    if lines.is_empty() {
        let hint = if state.github.issues_loading {
            "Loading issues…"
        } else if let Some(error) = &state.github.issues_error {
            error.as_str()
        } else {
            "No open issues · R refresh"
        };
        lines.push(Line::from(Span::styled(
            truncate_line(hint, max_width),
            theme.get(SemanticRole::Muted),
        )));
    }

    frame.render_widget(Paragraph::new(lines), content);
    if let Some(scrollbar_area) = scrollbar {
        render_vertical_scrollbar(
            frame,
            scrollbar_area,
            state.github.issues_scroll_offset,
            total_rows,
            viewport_rows,
            focused,
            theme,
        );
    }
}

fn render_issue_line(
    issue: &Issue,
    row_index: usize,
    selected_row: Option<usize>,
    max_width: usize,
    focused: bool,
    theme: &ThemePalette,
) -> Line<'static> {
    let selected = focused && selected_row == Some(row_index);
    let state_style = match issue.state {
        IssueState::Open => theme.get(SemanticRole::IssueOpen),
        IssueState::Closed => theme.get(SemanticRole::IssueClosed),
    };

    let mut spans = vec![
        Span::styled(format!("#{} ", issue.number), state_style),
        Span::styled(
            truncate_line(&issue.title, max_width.saturating_sub(8)),
            if selected {
                theme.get(SemanticRole::Accent)
            } else {
                theme.get(SemanticRole::Fg)
            },
        ),
    ];

    if issue.state == IssueState::Closed {
        spans.push(Span::styled(
            format!(" [{}]", issue.state.label()),
            theme.get(SemanticRole::IssueClosed),
        ));
    }

    let mut line = Line::from(spans);
    if selected {
        line = line.style(Style::default().add_modifier(Modifier::BOLD));
    }
    line
}

fn render_issues_list_status_line(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &ThemePalette,
) {
    let status = if state.github.issues_loading {
        "Loading issues…"
    } else if state.github.issues_error.is_some() {
        "j/k move · R refresh"
    } else if state.github.issues.is_empty() {
        "R refresh"
    } else {
        "j/k move · Enter view · i/p switch · R refresh"
    };

    frame.render_widget(
        Paragraph::new(truncate_line(status, area.width as usize))
            .style(theme.get(SemanticRole::Muted)),
        area,
    );
}

fn render_github_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    theme: &ThemePalette,
    state: &AppState,
    title: &str,
    lines: Vec<Line<'_>>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let border_style = if focused {
        theme.get(SemanticRole::Accent)
    } else {
        theme.get(SemanticRole::Border)
    };

    let block = Block::default()
        .title(title_with_auth_status(title, state))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(chrome_style(theme));

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.get(SemanticRole::Fg))
            .wrap(Wrap { trim: false }),
        inner,
    );
}

fn title_with_auth_status(title: &str, state: &AppState) -> String {
    if state.github.loading {
        return format!("{title} · checking auth");
    }

    if !state.github.auth_checked {
        return title.to_string();
    }

    if state.github.auth_ok {
        format!("{title} · authenticated")
    } else {
        format!("{title} · auth required")
    }
}

fn github_auth_message(state: &AppState) -> Option<&str> {
    if state.github.auth_ok {
        return None;
    }

    if !state.github.auth_checked && !state.github.loading {
        return None;
    }

    state
        .github
        .error
        .as_deref()
        .filter(|message| !message.is_empty())
}

fn auth_error_lines(message: &str, kind: Option<GitHubAuthErrorKind>) -> Vec<Line<'static>> {
    let heading = match kind {
        Some(GitHubAuthErrorKind::NotInstalled) => "GitHub CLI required",
        Some(GitHubAuthErrorKind::NotAuthenticated) => "GitHub login required",
        Some(GitHubAuthErrorKind::CommandFailed) | None => "GitHub auth check failed",
    };

    let mut lines = vec![Line::from(Span::styled(
        heading,
        Style::default().add_modifier(Modifier::BOLD),
    ))];

    for paragraph in message.split("\n\n") {
        lines.push(Line::from(""));
        for line in paragraph.lines() {
            lines.push(Line::from(line.to_string()));
        }
    }

    lines
}

fn pane_inner(area: Rect) -> Option<Rect> {
    if area.width < 2 || area.height < 2 {
        return None;
    }

    Some(Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    })
}

fn truncate_line(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let char_count = text.chars().count();
    if char_count <= max_width {
        return text.to_string();
    }

    if max_width <= 1 {
        return "…".to_string();
    }

    text.chars()
        .take(max_width.saturating_sub(1))
        .collect::<String>()
        + "…"
}

fn chrome_style(theme: &ThemePalette) -> Style {
    let mut style = Style::default();
    if let Some(bg) = theme.get(SemanticRole::Bg).bg {
        style = style.bg(bg);
    }
    if let Some(fg) = theme.get(SemanticRole::Fg).fg {
        style = style.fg(fg);
    }
    style
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::config::ResolvedConfig;
    use crate::github::{
        GitHubAuthErrorKind, GitHubLeftPane, Issue, IssueDetail, IssueState, PrDetail, PrState,
        PullRequest,
    };
    use crate::layout::compute_layout;
    use crate::navigation::{LeftNavTab, MainTab};
    use crate::state::AppState;
    use crate::theme::capabilities::TerminalCapabilities;
    use crate::theme::loader::load_theme_with_capabilities;

    use super::*;

    fn test_state() -> AppState {
        AppState::from_startup(
            std::path::PathBuf::from("."),
            false,
            ResolvedConfig::default(),
            load_theme_with_capabilities(
                &ResolvedConfig::default().theme,
                TerminalCapabilities::TrueColor,
            )
            .expect("theme"),
            compute_layout(120, 40, 30).expect("layout"),
        )
    }

    #[test]
    fn github_issue_interaction_at_selects_issue_row() {
        let mut state = test_state();
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.issues = vec![
            Issue {
                number: 1,
                title: "First".to_string(),
                state: IssueState::Open,
                labels: Vec::new(),
                assignees: Vec::new(),
            },
            Issue {
                number: 2,
                title: "Second".to_string(),
                state: IssueState::Open,
                labels: Vec::new(),
                assignees: Vec::new(),
            },
        ];
        state.github.left_pane = GitHubLeftPane::Issues;

        let area = Rect::new(0, 0, 60, 12);
        let row = (area.y..area.y.saturating_add(area.height))
            .find(|row| github_issue_interaction_at(&state, area, area.x + 2, *row).is_some())
            .expect("issue row");
        let index = github_issue_interaction_at(&state, area, area.x + 2, row).expect("index");
        assert_eq!(index, 0);
    }

    #[test]
    fn github_pr_interaction_at_selects_pr_row() {
        let mut state = test_state();
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.prs = vec![
            PullRequest {
                number: 59,
                title: "PR list via gh json".to_string(),
                state: PrState::Open,
                author: "octocat".to_string(),
                is_draft: false,
            },
            PullRequest {
                number: 60,
                title: "PR detail view".to_string(),
                state: PrState::Open,
                author: "hubot".to_string(),
                is_draft: false,
            },
        ];
        state.github.left_pane = GitHubLeftPane::Prs;

        let area = Rect::new(0, 0, 60, 12);
        let row = (area.y..area.y.saturating_add(area.height))
            .find(|row| github_pr_interaction_at(&state, area, area.x + 2, *row).is_some())
            .expect("pr row");
        let index = github_pr_interaction_at(&state, area, area.x + 2, row).expect("index");
        assert_eq!(index, 0);
    }

    #[test]
    fn github_left_pane_renders_prs_list_when_selected() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.prs = vec![PullRequest {
            number: 59,
            title: "PR list via gh json".to_string(),
            state: PrState::Open,
            author: "octocat".to_string(),
            is_draft: false,
        }];
        state.github.left_pane = GitHubLeftPane::Prs;

        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_github_left_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("PRs · 1"));
        assert!(content.contains("#59"));
    }

    #[test]
    fn github_left_pane_renders_issues_and_prs_hub() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.issues = vec![Issue {
            number: 55,
            title: "Issue list via gh json".to_string(),
            state: IssueState::Open,
            labels: Vec::new(),
            assignees: Vec::new(),
        }];
        state.github.left_pane = GitHubLeftPane::Issues;

        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_github_left_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("Issues · 1"));
        assert!(content.contains("PRs"));
        assert!(content.contains("#55"));
    }

    #[test]
    fn issues_list_pane_renders_in_left_gh_panel() {
        let mut state = test_state();
        state.navigation.left_tab = LeftNavTab::Gh;
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.issues = vec![Issue {
            number: 55,
            title: "Issue list via gh json".to_string(),
            state: IssueState::Open,
            labels: vec!["epic-e14".to_string()],
            assignees: vec!["octocat".to_string()],
        }];
        state.github.selected_issue = Some(55);
        state.github.issues_loaded_at = Some(SystemTime::now());

        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_github_left_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("#55"));
        assert!(content.contains("Issue list via gh json"));
    }

    #[test]
    fn issue_detail_pane_shows_loaded_body_and_comments() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Issues;
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.selected_issue = Some(56);
        state.github.issue_detail_number = Some(56);
        state.github.issue_detail = Some(IssueDetail {
            number: 56,
            title: "Issue detail view".to_string(),
            state: IssueState::Open,
            author: "pacificnm".to_string(),
            labels: vec!["epic-e14".to_string()],
            assignees: Vec::new(),
            display_lines: vec![
                "#56 Issue detail view".to_string(),
                "State: open · Author: pacificnm".to_string(),
                "Labels: epic-e14".to_string(),
                String::new(),
                "— Body —".to_string(),
                String::new(),
                "Detailed body text".to_string(),
                String::new(),
                "— Comments (1) —".to_string(),
                String::new(),
                "@reviewer · 2026-06-24".to_string(),
                "Ship it".to_string(),
            ],
        });

        let backend = TestBackend::new(100, 16);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_issue_detail_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("#56 Issue detail view"));
        assert!(content.contains("Detailed body text"));
        assert!(content.contains("@reviewer"));
        assert!(content.contains("Ship it"));
    }

    #[test]
    fn pr_detail_pane_shows_loaded_description_commits_and_checks() {
        let mut state = test_state();
        state.navigation.main_tab = MainTab::Prs;
        state.github.auth_checked = true;
        state.github.auth_ok = true;
        state.github.selected_pr = Some(60);
        state.github.pr_detail_number = Some(60);
        state.github.pr_detail = Some(PrDetail {
            number: 60,
            title: "PR detail view".to_string(),
            state: PrState::Open,
            author: "pacificnm".to_string(),
            display_lines: vec![
                "#60 PR detail view".to_string(),
                "State: open · Author: pacificnm".to_string(),
                "Branch: feature → main · +10 -2 · 3 files".to_string(),
                String::new(),
                "— Description —".to_string(),
                String::new(),
                "PR body text".to_string(),
                String::new(),
                "— Commits (1) —".to_string(),
                "· Add loader · @pacificnm · 2026-06-25".to_string(),
                String::new(),
                "— Checks (1) —".to_string(),
                "· ci: SUCCESS".to_string(),
            ],
        });

        let backend = TestBackend::new(100, 18);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_pr_detail_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("#60 PR detail view"));
        assert!(content.contains("PR body text"));
        assert!(content.contains("Add loader"));
        assert!(content.contains("ci: SUCCESS"));
    }

    #[test]
    fn main_pane_shows_install_instructions_when_gh_missing() {
        let mut state = test_state();
        state.github.auth_checked = true;
        state.github.auth_ok = false;
        state.github.error_kind = Some(GitHubAuthErrorKind::NotInstalled);
        state.github.error = Some("GitHub CLI (gh) not found.".to_string());

        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_issue_detail_pane(frame, frame.area(), true, &state.theme, &state);
            })
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("GitHub CLI required"));
    }
}
