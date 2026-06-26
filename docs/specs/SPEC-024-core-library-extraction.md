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

4. **Remain in `kiwi` after extraction:** `ui/`, `layout/`, `navigation/`, `terminal/`, `selection/`, `clipboard/` (TUI integration), `app.rs` TUI loop.
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
├── workspace/
├── file_tree/
├── git/
├── github/
├── search/
├── preview/
├── diff/
├── shell/
├── agent/
├── editor/
└── commands/
```

## Dependency Graph (post-extraction)

```text
kiwi      → kiwi_core, ratatui, crossterm
kiwi_gui  → kiwi_core, egui, eframe, egui_dock
kiwi_core → tokio, serde, git2, notify, portable-pty, …
```

## Error Handling

- Migration PR that breaks TUI behavior is rolled back; no partial public API without tests

## Acceptance Criteria

- [ ] `kiwi_core` crate exists and publishes stable `AppState`, `AppEvent`, `AppCommand`
- [ ] `kiwi` binary behavior unchanged (regression: existing integration tests pass)
- [ ] `kiwi_gui` can import config, state, and spawn git service without `kiwi` dependency
- [ ] `cargo clippy -p kiwi_core` passes with no UI crate in dependency tree
- [ ] Each phase C1–C10 complete or explicitly deferred with issue link

## Notes

- Extraction can proceed in parallel with GUI scaffolding once C1–C4 land
- `kiwi_tui` optional crate extracts later from `kiwi::ui` without blocking GUI
