# SPEC-009: GitHub Service

## Purpose

Integrate GitHub Issues and Pull Requests via `gh` CLI for list, view, and common workflow actions.

## Scope

### In scope

- Issue list/view/search/comment/label/assign/create branch
- PR list/view/create/merge/review/open browser
- JSON parsing from `gh`

### Out of scope

- Merge method picker (squash/rebase vs merge commit) in v1
- GitHub Enterprise custom hosts (document `GH_HOST` env passthrough)

## Functional Requirements

### Issues

1. List open/closed issues with number, title, labels, assignees.
2. View issue detail: body, comments thread (paginated).
3. Search/filter by text, label, assignee.
4. Actions: comment (prompt in palette), assign self, add label (multi-select TUI).
5. Create branch from issue: `gh issue develop <n>` + checkout feedback.

**UI layout:** Issue list renders in the **GH** left navigation pane; issue detail renders in the main **Issues** tab (orthogonal tabs, same pattern as Git left + Diff main). List navigation (`j`/`k`, `Enter`) requires left focus on the GH tab.

### Pull Requests

1. List PRs: number, title, state (open/draft/merged/closed), author.
2. View PR: description, commits summary, checks status if available.
3. Create PR: guided prompts → `gh pr create`.
4. Merge PR: `gh pr merge <n> --merge` on selected open, non-draft PR via command palette (`github.pr.merge`) or PR context menu (**Merge into main**).
5. Review: comment via `gh pr review --comment`.
6. Open in browser: `gh pr view --web` / `gh issue view --web`.

Merge failures (branch protection, failing checks, conflicts) surface `gh` stderr in a toast/status message. Successful merge refreshes the PR list.

### Auth

On first access, run `gh auth status`; if fail, show setup instructions with link to `gh auth login`.

## Non-Functional Requirements

- List load < 2s on typical connection (network bound)
- Cache issue/PR list for 60s; invalidate on user refresh
- Async subprocess; never block render loop

## Data Structures

```rust
struct Issue {
    number: u32,
    title: String,
    state: IssueState,
    labels: Vec<String>,
    assignees: Vec<String>,
    body: String,
}

struct PullRequest {
    number: u32,
    title: String,
    state: PrState,
    author: String,
    is_draft: bool,
}

struct GitHubState {
    issues: Vec<Issue>,
    prs: Vec<PullRequest>,
    selected_issue: Option<u32>,
    selected_pr: Option<u32>,
    auth_ok: bool,
    error: Option<String>,
}
```

## Events / Commands

```rust
AppCommand::GitHubRefreshIssues
AppCommand::GitHubRefreshPrs
AppCommand::GitHubComment { issue: u32, body: String }
AppCommand::GitHubCreateBranch(u32)
AppCommand::GitHubCreatePr { title, body, base }
AppCommand::GitHubMergePr(u32)
AppCommand::GitHubOpenBrowser { target }
```

## Configuration Options

```toml
[github]
command = "gh"
```

## Error Handling

| Error | Behavior |
|-------|----------|
| gh not installed | Inline panel with install docs |
| Not authenticated | Auth setup panel |
| API rate limit | Show message; backoff |
| Not a github remote | Warn when detecting origin |
| Merge blocked | Toast with `gh` error message |

## Acceptance Criteria

- [ ] Issue list loads for authenticated repo
- [ ] Issue detail shows body
- [ ] PR list shows open PRs with state colors
- [ ] Create branch from issue works
- [ ] Merge PR command and context menu merge selected open PR via `gh`
- [ ] Merge unavailable for draft/merged/closed PRs
- [ ] Browser open command works
- [ ] Colors match theme PR/issue roles
