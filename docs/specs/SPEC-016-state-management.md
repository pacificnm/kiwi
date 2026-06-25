# SPEC-016: State Management

## Purpose

Define centralized application state, event routing, and incremental update rules per ADR-007.

## Scope

### In scope

- AppState composition
- AppEvent / AppCommand enums
- Reducer pattern
- Channel between async services and main loop

### Out of scope

- Persistent storage format (SPEC-017)
- Individual domain logic (domain SPECs)

## Functional Requirements

1. Single `AppState` owned by main loop.
2. All mutations via `reduce(state, event) -> (State, Vec<SideEffect>)`.
3. Services send `AppEvent` only; never mutate state directly.
4. `AppCommand` originates from user input and palette; may trigger side effects.
5. List updates use stable IDs; preserve scroll/selection per domain rules.
6. `dirty` flag set on any state change requiring redraw.
7. Optional `tracing` span per event in debug builds.

## Non-Functional Requirements

- Reducer for typical input event < 1ms
- Channel depth 1024; drop policy: coalesce duplicate Git refresh events

## Data Structures

```rust
struct AppState {
    config: ResolvedConfig,
    navigation: NavigationState,
    layout: LayoutState,
    file_tree: FileTreeState,
    preview: PreviewState,
    search: SearchState,
    git: GitState,
    diff: DiffState,
    github: GitHubState,
    agent: AgentState,
    shell: ShellState,
    palette: CommandPaletteState,
    theme: ThemePalette,
    status_bar: StatusBarState,
    workspace_meta: WorkspaceMeta,
}

enum AppEvent {
    // Crossterm, services, watcher — union of all domain events
}

enum AppCommand {
    // User intentions
}

enum SideEffect {
    SpawnGitRefresh,
    WritePty { target: PtyTarget, data: Vec<u8> },
    LaunchEditor(PathBuf),
    SaveWorkspace,
    // ...
}
```

## Events / Commands

See domain SPECs for full event lists; this SPEC mandates the pattern.

## Configuration Options

N/A

## Error Handling

- Reducer panics are bugs; use `anyhow` in services instead
- Invalid state transitions ignored with debug log

## Acceptance Criteria

- [x] Unit tests for reducers without terminal
- [x] Git refresh preserves file list selection and scroll where paths remain
- [ ] No Arc<Mutex> on render path
- [ ] Side effects executed after reduce step
- [ ] Event coalescing prevents refresh storm
