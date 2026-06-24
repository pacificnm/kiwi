# ADR-012: GitHub Integration

## Status

Accepted

## Context

Issue-driven and AI-driven workflows require GitHub Issues and Pull Requests inside Kiwi. Full GraphQL API integration is powerful but increases auth, rate-limit, and schema maintenance burden for MVP.

## Decision

Use **GitHub CLI (`gh`)** as the initial integration layer for all GitHub operations.

### Operations via `gh`

| Domain | Commands (conceptual) |
|--------|----------------------|
| Issues | `gh issue list`, `view`, `create`, `comment`, `edit` |
| PRs | `gh pr list`, `view`, `create`, `review`, `merge` (view only in v1) |
| Branches | `gh issue develop`, `git` via shell for branch checkout |
| Browser | `gh pr view --web`, `gh issue view --web` |

Parse **JSON output** (`--json` flags) for structured data in TUI lists.

### Configuration

```toml
[github]
command = "gh"
```

Require `gh auth status` success at first GitHub tab access; show setup instructions on failure.

### Future evaluation

**octocrab** (GraphQL/REST) for:

- Reduced subprocess overhead
- Finer-grained caching
- Offline queue (unlikely)

Deferred to post-MVP unless `gh` blockers emerge.

## Consequences

### Positive

- Users already authenticate via `gh auth login`
- CLI stable JSON interfaces
- Faster MVP delivery

### Negative

- Subprocess latency per operation
- Feature set bounded by `gh` capabilities
- Requires `gh` installed and authenticated

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| octocrab from day one | More code, token handling, testing infra |
| Hub (deprecated) | Unmaintained |
| Webview TUI | Not terminal-native |

## Follow-up Work

- SPEC-009 GitHub Service
- Pin minimum `gh` version (e.g., 2.40+) in docs
- Issue list filters: open/closed, assignee, label
- PR colors per ADR-004
- Workflow docs: [workflows.md](../../design/workflows.md)
