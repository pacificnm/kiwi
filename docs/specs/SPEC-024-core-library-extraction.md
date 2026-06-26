# SPEC-024: Core Library Extraction

## Purpose

Define what moves from the `kiwi` binary crate into `kiwi_core` so TUI and GUI frontends share domain logic without UI framework dependencies.

## Scope

### In scope

- `kiwi_core` crate module layout
- Migration order and acceptance per module
- Dependency rules and forbidden imports
- Public API surface for frontends

### Out of scope

- Implementing the migration (tracked in backlog epic E19)
- Plugin API changes (SPEC-020)

## Related Documents

- ADR-020 Dual Frontend Architecture
- SPEC-016 State Management
- [repository-structure.md](../repository-structure.md)

## Functional Requirements

1. **New crate:** `crates/kiwi_core` as workspace library, no binary.
2. **Forbidden in `kiwi_core`:** `ratatui`, `crossterm`, `egui`, `eframe`, `egui_dock`, and any TUI/GUI-specific types.
3. **Migration phases** (order matters):

   | Phase | Modules | Source today |
   |-------|---------|--------------|
   | C1 | `config` | `kiwi::config` |
   | C2 | `theme` (roles, loader, palette types) | `kiwi::theme` |
   | C3 | `events`, `state`, `reducer` | `kiwi::state` |
   | C4 | `workspace` | `kiwi::workspace` |
   | C5 | `file_tree` (logic, not render) | `kiwi::file_tree` |
   | C6 | `git`, `watcher` | `kiwi::git`, `kiwi::watcher` |
   | C7 | `github` | `kiwi::github` |
   | C8 | `search`, `preview`, `diff` | respective modules |
   | C9 | `shell`, `agent` (session + IO) | `kiwi::shell`, `kiwi::agent` |
   | C10 | `editor`, `commands` registry | `kiwi::editor`, `kiwi::commands` |

4. **Remain in `kiwi` after extraction:** `ui/`, `layout/`, `navigation/` (TUI key routing), `terminal/`, `selection/` (TUI hit-testing), `clipboard/` (TUI I/O integration), `app.rs` TUI loop. Domain logic for `navigation`, `editor`, and `commands` lives in `kiwi_core`; `kiwi` keeps thin adapters.
5. **`kiwi` depends on `kiwi_core`:** `kiwi` re-exports nothing publicly; binary-only.
6. **`kiwi_gui` depends on `kiwi_core` only** for domain + state; no dependency on `kiwi`.
7. **Tests:** Domain unit tests move with modules; `kiwi` integration tests remain for TUI.

## Non-Functional Requirements

- Extraction PRs keep `cargo test --workspace` green
- No circular dependencies
- `kiwi_core` public API documented with `//!` module docs

## Data Structures

Target layout (see repository-structure.md):

```text
kiwi_core/src/
├── config/
├── theme/
├── events/
├── state/
├── reducer/
├── commands/
├── clipboard/
├── editor/
├── navigation/
├── workspace/          # C4
├── file_tree/          # C5
├── git/                # C6
├── watcher/            # C6
├── github/             # C7
├── search/             # C8
├── preview/            # C8
├── diff/               # C8
├── shell/              # C9
├── agent/              # C9 (runtime orchestration stays in `kiwi::agent::runtime`)
├── settings/
└── selection/
```

## Migration status

Track completion in extraction PRs. Phases may land out of strict table order when shared
reducer dependencies require an earlier module.

| Phase | Status | Notes |
|-------|--------|-------|
| C1 `config` | Done | `kiwi_core::config`; TUI loader/writer remain in `kiwi` |
| C2 `theme` | Done | Core palette + loader; `kiwi::theme` wraps ratatui `Style` |
| C3 `events`, `state`, `reducer` | Done ([#177](https://github.com/pacificnm/kiwi/issues/177)) | `AppEvent`, `AppCommand`, domain `AppState`, `ReduceView`, and reducer in core; TUI adapter in `kiwi::state::reducer` + `reducer_tui` |
| C4 `workspace` | Done ([#178](https://github.com/pacificnm/kiwi/issues/178)) | Snapshot/persistence in `kiwi_core::workspace`; TUI save adapter in `kiwi` |
| C5 `file_tree` | Done ([#179](https://github.com/pacificnm/kiwi/issues/179)) | Loader, ignore, classify, io in core; TUI re-export shim |
| C6 `git`, `watcher` | Done ([#179](https://github.com/pacificnm/kiwi/issues/179)) | `git2` repository/branch ops + notify watcher in core |
| C7 `github` | Done ([#179](https://github.com/pacificnm/kiwi/issues/179)) | Full `gh` subprocess layer in core |
| C8 `search`, `preview`, `diff` | Done ([#179](https://github.com/pacificnm/kiwi/issues/179)) | Search io/content/file, preview loader, diff generate/io in core |
| C9 `shell`, `agent` | Done ([#179](https://github.com/pacificnm/kiwi/issues/179)) | PTY session + output readers in core; `encode_key` + `AgentRuntime` in TUI |
| C10 `editor`, `commands` | Done ([#179](https://github.com/pacificnm/kiwi/issues/179)) | Launch/resolve/classify in core; TUI target resolution + terminal suspend in `kiwi` |

**C3 acceptance (issue #177):**

- [x] `kiwi_core` publishes `AppEvent`, `AppCommand`, domain `AppState`, and `reduce`
- [x] TUI `kiwi::state::reducer` delegates to core via `ReduceView`; TUI-only paths (resize, clipboard, selection, theme sync) stay in `reducer_tui`
- [x] `kiwi_core` clipboard/editor/commands use `ViewportMetrics` / `ReduceView` (no layout rects)
- [x] `cargo test --workspace` green; `cargo clippy -p kiwi_core -p kiwi -- -D warnings` green

## Dependency Graph (post-extraction)

```text
kiwi      → kiwi_core, ratatui, crossterm
kiwi_gui  → kiwi_core, egui, eframe, egui_dock
kiwi_core → tokio, serde, git2, notify, portable-pty, …
```

## Error Handling

- Migration PR that breaks TUI behavior is rolled back; no partial public API without tests

## Acceptance Criteria

- [x] `kiwi_core` crate exists and publishes stable `AppState`, `AppEvent`, `AppCommand`
- [x] `kiwi` binary behavior unchanged (regression: existing integration tests pass)
- [ ] `kiwi_gui` can import config, state, and spawn git service without `kiwi` dependency
- [x] `cargo clippy -p kiwi_core` passes with no UI crate in dependency tree
- [ ] Each phase C1–C10 complete or explicitly deferred with issue link (see **Migration status**)

## Notes

- Extraction can proceed in parallel with GUI scaffolding once C1–C4 land
- `kiwi_tui` optional crate extracts later from `kiwi::ui` without blocking GUI
