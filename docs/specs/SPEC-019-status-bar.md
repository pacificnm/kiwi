# SPEC-019: Status Bar

## Purpose

Display persistent contextual information in a single full-width bottom status row.

## Scope

### In scope

- Fixed format status line
- Dynamic segments from git, agent, github context

### Out of scope

- Clickable status items (future)
- Progress bars

## Functional Requirements

Format:

```text
Kiwi | <repo> | <branch> | <agent> | <git> | <issue>
```

Example:

```text
Kiwi | cityartwalks | feature/42 | Agent Running | 3 Modified | #42
```

Segments:

| Segment | Source | Fallback |
|---------|--------|----------|
| repo | directory basename | — |
| branch | `GitState.branch` | `no git` |
| agent | `AgentState.status` | `Agent Idle` |
| git | modified count summary | `Clean` |
| issue | linked issue from branch or selection | empty if none |

Use theme muted for separators, accent for warnings/errors.

Truncate middle segments on narrow terminals; never wrap.

## Non-Functional Requirements

- Update within same frame as state change
- No flicker on segment value change (in-place update)

## Data Structures

```rust
struct StatusBarState {
    repo_name: String,
    branch: String,
    agent_label: String,
    git_label: String,
    issue_label: Option<String>,
}
```

## Events / Commands

Derived from `AppState` on each render; no separate reducer except helper `compute_status_bar(state)`.

## Configuration Options

```toml
[status_bar]
show_issue = true
```

## Error Handling

- Missing data shows sensible defaults; never blank bar

## Acceptance Criteria

- [ ] All segments visible at 120 cols
- [ ] Truncation works at 80 cols
- [ ] Modified count updates on file save
- [ ] Agent state changes reflect immediately
- [ ] Issue number shows when issue selected
