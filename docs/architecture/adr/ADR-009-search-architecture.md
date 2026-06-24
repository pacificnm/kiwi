# ADR-009: Search Architecture

## Status

Accepted

## Context

Users need to find files and content quickly. Search spans left nav (dedicated tab) and may interact with file preview. Future enhancements include ripgrep and tree-sitter symbol search.

## Decision

Phased search architecture:

### Phase 1 (Milestone 3 — MVP search)

- **File name search**: fuzzy match on relative paths; scan with `ignore` rules (future crate) or built-in ignore list
- **Content search**: invoke `rg` (ripgrep) as subprocess if available; fallback to `grep -r` with documented limitations
- Results in left Search panel; selecting opens Preview or editor

### Phase 2 (Milestone 7+)

- Native ripgrep integration via library or optimized walk
- **Symbol search**: tree-sitter parsers for Rust, TypeScript, Python, Go (prioritize by demand)
- Search index cache per repo (optional persistence)

### UI model

Single search input with mode toggle: `Files | Content | Symbols` (Symbols disabled until Phase 2).

Results list uses stable IDs (path + line number for content hits). Incremental result streaming for content search.

## Consequences

### Positive

- MVP delivers value with minimal Rust code
- ripgrep subprocess leverages user's installed tooling
- Clear upgrade path to integrated search

### Negative

- Subprocess search adds latency vs in-process
- Symbol search deferred; document limitation
- Without `rg`, fallback is slower and less feature-rich

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Built-in regex scan only | Too slow on large repos |
| Elasticsearch/index server | Violates terminal-local tool philosophy |
| fzf external only | Poor integration with Kiwi UI |

## Follow-up Work

- SPEC-007 Search System
- Detect `rg` at startup; status bar hint if missing
- Cancel in-flight search on new query (cancellation token)
- Add `search.command` config override for custom search tool
