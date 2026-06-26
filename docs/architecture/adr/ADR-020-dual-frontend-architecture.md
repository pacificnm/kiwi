# ADR-020: Dual Frontend Architecture

## Status

Accepted

## Context

Kiwi ships today as a terminal-native application built on ratatui and crossterm (ADR-002). Users want an IDE-like desktop interface with draggable panels, mouse-first navigation, and richer widgets—without replacing the existing TUI.

The codebase currently lives in a single `kiwi` binary crate. Domain logic (Git, GitHub, file tree, search, PTY services, state reducers) is interleaved with ratatui rendering. A second frontend requires clear crate boundaries so both UIs share services and state contracts while keeping presentation code separate.

## Decision

Adopt a **dual frontend architecture**: one shared core library and two presentation crates.

```text
                    ┌─────────────────┐
                    │   kiwi_core     │
                    │ config, events, │
                    │ state, services │
                    └────────┬────────┘
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
        ┌──────────┐  ┌──────────┐  ┌──────────────┐
        │  kiwi    │  │ kiwi_tui │  │  kiwi_gui    │
        │ (binary) │  │ (future) │  │  (binary)    │
        │ ratatui  │  │ widgets  │  │  egui/eframe │
        └──────────┘  └──────────┘  └──────────────┘
```

### Crate responsibilities

| Crate | Role | Depends on |
|-------|------|------------|
| `kiwi_core` | Config, events, commands, reducers, domain state, async services, theme roles, workspace persistence types | tokio, serde, domain crates (git2, etc.) — **no UI frameworks** |
| `kiwi` | TUI binary: crossterm event loop, ratatui render, TUI-specific input and layout (SPEC-002) | `kiwi_core`, ratatui, crossterm |
| `kiwi_gui` | GUI binary: eframe application, egui_dock layout, GUI panels (SPEC-021–023) | `kiwi_core`, egui, eframe, egui_dock |
| `kiwi_tui` | Optional future extraction of ratatui widgets from `kiwi` | `kiwi_core`, ratatui |

### Entry points

- **Default:** `kiwi` — unchanged terminal behavior.
- **GUI:** `kiwi-gui` binary (preferred) or `kiwi --gui` flag that dispatches to the GUI bootstrap. Both frontends share CLI flags for repository root and config paths (SPEC-001, SPEC-018).

### Extraction rules

Move into `kiwi_core` when logic is:

- Used by both frontends, or clearly domain/service logic with no terminal assumptions
- Expressible without `ratatui::`, `crossterm::`, `egui::`, or `eframe::` types

**Keep in `kiwi` (TUI):**

- Layout engine rects and ratatui widget trees (SPEC-002)
- Crossterm keyboard/mouse translation (SPEC-004, SPEC-014)
- TUI hit tests, scrollbars, ANSI PTY viewport rendering
- Terminal init/teardown (raw mode, alternate screen)

**Keep in `kiwi_gui`:**

- egui_dock tree and `KiwiTab` panel widgets (ADR-022)
- egui theme bridge (SPEC-023)
- Window menus, native file dialogs, GUI shortcuts
- egui-based PTY surface (separate from ratatui ANSI viewport)

### Shared contracts

Both frontends use the same:

- `AppEvent` / `AppCommand` enums (SPEC-016)
- `AppState` snapshot and reducers (ADR-007)
- Config loader and theme role resolution (ADR-004, ADR-005)
- Service spawn pattern: tokio tasks → `mpsc` → main loop drain (ADR-006, ADR-010–012)
- Workspace persistence schema with frontend-specific extensions (ADR-016)

### Concurrency model (GUI)

The GUI uses the same event-driven pattern as the TUI:

1. eframe `update()` runs on the main thread each frame
2. Drain pending `AppEvent` messages from services before rendering panels
3. Reducers mutate `AppState` on the main thread
4. Panel widgets read immutable snapshots; no `Arc<Mutex<AppState>>` on the render path

Tokio runtime: embed via `eframe` async integration or a dedicated runtime handle created at startup; PTY and subprocess I/O remain async tasks.

## Consequences

### Positive

- TUI remains the default; no breaking change for terminal users
- GUI can evolve independently (docking, menus, native dialogs)
- Core services tested once; both frontends benefit from fixes
- Clear path to extract `kiwi_tui` later without blocking GUI work

### Negative

- Upfront cost to scaffold `kiwi_core` and migrate modules from `kiwi`
- Two presentation codebases to maintain until feature parity is defined
- Workspace persistence must version or namespace TUI vs GUI layout state
- CI builds and tests both binaries; headless GUI tests need `--no-default-features` or mock eframe

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Replace TUI with egui only | Breaks terminal-native users and SSH workflows |
| Single binary with runtime UI switch | Couples ratatui and egui in one dependency tree; harder to ship minimal TUI-only builds |
| Web UI (Tauri/wasm) | Out of scope; Kiwi targets native desktop + terminal first |
| Shared `kiwi` crate with `#[cfg(feature = "gui")]` | Becomes unmaintainable as widget code diverges |

## Follow-up Work

- Scaffold `kiwi_core` and migrate config, events, state, services incrementally
- Add `kiwi_gui` crate with empty eframe window (SPEC-021)
- Update [repository-structure.md](../../repository-structure.md) dependency graph
- Add M8 milestone and backlog epics in [milestones.md](../../roadmap/milestones.md)
- Define GUI persistence extension in SPEC-017 amendment or SPEC-024
- Document extraction order in [gui-implementation-plan.md](../../design/gui-implementation-plan.md)
