# ADR-008: File Tree Architecture

## Status

Accepted

## Context

The file browser is the primary navigation surface. Repositories can be large; loading entire trees eagerly is slow. Users expect expand/collapse, ignore rules, preview, and editor launch.

## Decision

Implement a **lazy-loaded file tree** with explicit expansion state and ignore defaults.

### Loading strategy

- On expand directory: read immediate children only (async via tokio `spawn_blocking`)
- Cache children per directory path in memory
- Invalidate cache entry on file watcher event for that path or ancestor

### Default ignore globs

```
.git, node_modules, target, dist, build, .next, .nuxt, .venv
```

Merge with `.gitignore` for display (optional hide ignored files toggle in future).

### Node identity

Each node: `path` (stable ID), `name`, `is_dir`, `children_loaded`, `expanded`, `sort_order`.

### Operations

| Action | Behavior |
|--------|----------|
| Expand | Load children if needed; toggle `expanded` |
| Collapse | Toggle `expanded`; retain cache |
| Select | Update selection; optional preview refresh |
| Open | Launch external editor (ADR-013) |
| Refresh | Invalidate root cache; preserve expansion set |

## Consequences

### Positive

- Fast startup on large repos
- Incremental invalidation via watcher
- Stable paths preserve selection across refreshes

### Negative

- First expand incurs latency (show spinner in node)
- Symlink cycles need detection
- Very deep paths need horizontal scroll or truncation

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Full tree walk at open | Slow on `node_modules`-adjacent repos even with ignores |
| `walkdir` upfront with ignore crate | Better for search; too heavy for tree alone at M3 |
| External `ranger`-style subprocess | Poor integration |

## Follow-up Work

- SPEC-005 File Explorer
- Integrate `ignore` crate in M3+ for consistent ignore semantics with search
- Watcher invalidation rules in ADR-011
- Keyboard: `h` collapse, `l` expand, `Enter` open editor
