# ADR-011: File Watcher Architecture

## Status

Accepted

## Context

Git status, file tree cache, and preview content must update when files change on disk. Polling is explicitly rejected. Kiwi must debounce rapid saves (e.g., formatter on save) and avoid invalidating entire UI state.

## Decision

Use the **notify** crate with **debounced, coalesced events** and **path-targeted invalidation**.

### Watcher scope

- Watch repository root recursively
- Respect `.gitignore` for watcher noise reduction where notify supports it; always ignore `.git/` internal churn from index writes selectively

### Debounce pipeline

```text
notify event → raw queue → debounce (300ms default) → coalesce by path → AppEvent::FsChanged { paths }
```

### Invalidation map

| Changed path | Invalidate |
|--------------|------------|
| Any tracked/untracked file | Git status (debounced) |
| Directory | File tree cache for that dir |
| Open preview file | Preview buffer reload |
| `.git/HEAD`, `.git/index` | Git branch/status |

### State preservation

After invalidation, reducers must:

- Keep scroll offset if still valid
- Keep selection if path still exists
- Keep focus pane unchanged

### Configuration

```toml
[git]
watch = true   # master switch for git-related refresh
```

Future: `[watcher] debounce_ms = 300` (implemented; default 300ms)

## Consequences

### Positive

- Low CPU vs polling
- Coalescing prevents formatter save storms
- Targeted invalidation minimizes flicker

### Negative

- notify behavior varies by OS (inotify, FSEvents, ReadDirectoryChangesW)
- Network filesystems may not support reliable watching
- Debounce adds slight delay to status updates (acceptable)

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Polling `git status` | Explicitly forbidden |
| watchman | Extra daemon dependency |
| Manual refresh only | Poor UX |

## Follow-up Work

- Integrate with SPEC-008 Git Service and SPEC-005 File Explorer
- Log watcher errors at `warn` level; fall back to manual refresh only on total failure
- Test coalescing with 50 rapid touch events

## Acceptance criteria (debounce and coalesce)

- [x] Watcher debounces notify events for `[watcher].debounce_ms` (default 300ms)
- [x] Repeated path changes within the debounce window coalesce to one path set
- [x] New events during the window reschedule the debounce deadline
- [x] Event channel merges duplicate `FsChanged` paths before reducer dispatch
- [x] 50 rapid path updates coalesce to a single `FsChanged` batch in tests

## Acceptance criteria (repo root watcher)

- [x] `notify` recursive watch on repository root at startup
- [x] `.git/` internal paths ignored except `HEAD` and `index`
- [x] Notify callback errors logged to stderr; spawn failure shows one-time in-app warning
- [x] `AppEvent::FsChanged` emitted after debounce for workspace file changes

## Acceptance criteria (path-targeted invalidation)

- [x] File changes invalidate the parent directory cache when loaded in the tree
- [x] Directory changes invalidate that directory's cache and its parent listing
- [x] Expanded invalidated directories reload without clearing selection
- [x] `.git/` internal paths do not invalidate the file tree
