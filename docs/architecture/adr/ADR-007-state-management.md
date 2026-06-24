# ADR-007: State Management

## Status

Accepted

## Context

Kiwi has many concurrent data sources: file tree, Git status, GitHub lists, search results, PTY output, UI focus, and tab state. Full re-renders on every change cause flicker and lost scroll/selection. State must be predictable and testable.

## Decision

Use a **centralized application state** with **immutable snapshots** and **incremental patches** for list/tree updates.

### State structure

```rust
// Conceptual — not implementation code
AppState {
    config: ResolvedConfig,
    workspace: WorkspaceState,      // tabs, focus, dimensions
    navigation: NavigationState,    // left + main tab indices
    file_tree: FileTreeState,
    git: GitState,
    github: GitHubState,
    search: SearchState,
    agent_pty: PtyState,
    shell_pty: PtyState,
    theme: ResolvedTheme,
    status_bar: StatusBarState,
}
```

### Update model

1. **Events** arrive (user input, service callback, watcher notification)
2. **Reducer** applies event to produce `StatePatch` or new sub-state
3. **Merge** patches into state; preserve `StableId` for list items where possible
4. **Render** reads snapshot only

### Flicker avoidance rules

- Lists keyed by stable IDs (file path, issue number, commit hash)
- On Git refresh: diff old/new file lists; update only changed rows
- Preserve `scroll_offset` and `selected_index` if selected item still exists
- Debounce rapid watcher events (ADR-011) before applying Git patches

### Concurrency

- Main thread: input, render, state reduction
- Tokio tasks: I/O; send `AppEvent` via `mpsc` channel
- No `Arc<Mutex<AppState>>` on hot render path; drain channel then single-threaded update

## Consequences

### Positive

- Testable reducers without terminal
- Predictable focus/scroll preservation
- Clear separation of sync UI state vs async services

### Negative

- Boilerplate for patch types per domain
- Large single state struct (mitigate with sub-modules)
- Must discipline all services to emit events, not mutate state directly

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Elm architecture crate (e.g., tea) | Less idiomatic; team familiarity with custom reducers |
| Global mutex shared state | Race conditions and render flicker |
| Actor model per pane | Over-complex for v1 |
| Relational embedded DB | Absurd for in-memory UI state |

## Follow-up Work

- SPEC-016 State Management
- Define `AppEvent` and `AppCommand` enums in specs
- Unit tests for scroll preservation on Git file list updates
- Optional: `tracing` spans on state transitions for debug
