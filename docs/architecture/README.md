# Architecture

Kiwi is a single-process Rust application with an event-driven core, async service tasks for I/O-bound work (PTY, file watching, Git, GitHub CLI), and **two presentation frontends**: a ratatui TUI (`kiwi`) and an egui/eframe desktop GUI (`kiwi-gui`). See ADR-020.

## System Overview

### TUI process (`kiwi`)

```text
┌─────────────────────────────────────────────────────────────────┐
│                         Kiwi Process (TUI)                       │
├─────────────────────────────────────────────────────────────────┤
│  App (main loop)                                                 │
│    ├── Event Bus / Message Router                                │
│    ├── State Store (immutable snapshots + patches)               │
│    └── Layout Engine → ratatui Widget Tree                       │
├─────────────────────────────────────────────────────────────────┤
│  Services (tokio tasks)                                          │
│    ├── Config Loader                                             │
│    ├── Theme Engine                                              │
│    ├── File Tree + Preview                                       │
│    ├── Search                                                    │
│    ├── Git Service + Watcher                                     │
│    ├── GitHub Service (gh subprocess)                            │
│    ├── Agent PTY                                                   │
│    ├── Shell PTY                                                   │
│    └── Editor Launcher                                           │
├─────────────────────────────────────────────────────────────────┤
│  External                                                        │
│    ├── User editor (nvim, code, etc.)                            │
│    ├── User shell (bash, zsh, fish)                              │
│    ├── Cursor Agent (or configured agent command)                │
│    └── gh CLI                                                    │
└─────────────────────────────────────────────────────────────────┘
```

### Desktop GUI process (`kiwi-gui`)

```text
┌─────────────────────────────────────────────────────────────────┐
│                      Kiwi Process (GUI)                          │
├─────────────────────────────────────────────────────────────────┤
│  eframe App::update (main thread)                                │
│    ├── Drain AppEvent channel                                    │
│    ├── Reducers → AppState                                       │
│    ├── Menu bar + egui_dock (KiwiTab panels)                     │
│    └── Status bar                                                │
├─────────────────────────────────────────────────────────────────┤
│  kiwi_core services (same as TUI)                                │
└─────────────────────────────────────────────────────────────────┘
```

Target crate split: `kiwi_core` holds shared services and state; `kiwi` and `kiwi_gui` are thin frontends (SPEC-024).

## Architectural Layers

| Layer | Responsibility | Key ADRs |
|-------|----------------|----------|
| Presentation (TUI) | ratatui layout, crossterm input, TUI mouse/clipboard | ADR-002, ADR-003, ADR-015, ADR-019 |
| Presentation (GUI) | eframe shell, egui_dock, GUI panels, theme bridge | ADR-020, ADR-021, ADR-022 |
| Application | Navigation, command palette, state | ADR-007, ADR-014, ADR-016 |
| Domain services | Git, GitHub, files, search, agents | ADR-008–ADR-012, ADR-017 |
| Infrastructure | Config, PTY, file watcher, plugins | ADR-005, ADR-006, ADR-011, ADR-018 |
| Integration | External editors | ADR-013 |

## Data Flow

1. **Input** — `crossterm` events (keyboard, mouse, resize) enter the main loop.
2. **Dispatch** — Events translate to `AppCommand` messages; focused pane determines routing.
3. **Mutation** — Reducers update domain state; services emit async results as events.
4. **Render** — State snapshot drives widget tree; theme palette resolves styles.
5. **Side effects** — PTY I/O, `gh` calls, editor spawns, and file watcher callbacks run on tokio.

## Cross-Cutting Concerns

### Flicker avoidance

All list/tree views maintain stable item IDs. Updates apply patches (add/remove/change) rather than full rebuilds. Scroll offset and selection index are preserved when the underlying item still exists. See ADR-007 and ADR-011.

### Configuration

Resolved once at startup with hot-reload deferred to post-MVP. Precedence: CLI → `.kiwi.toml` → `~/.config/kiwi/config.toml` → defaults. See ADR-005.

### Persistence

Workspace state (open tabs, scroll positions, expanded tree nodes) serializes to `~/.local/state/kiwi/` per repository. See ADR-016.

## Decision Records

All architecture decisions are documented in [adr/](./adr/README.md).

## Specifications

Behavioral contracts live in [../specs/](../specs/README.md). Each SPEC maps to one or more ADRs and roadmap milestones.

## Open Architecture Questions

| Question | Status | Owner |
|----------|--------|-------|
| Single vs multi-threaded render thread | Deferred; start with main-thread render | M1 |
| `gh` JSON schema versioning | Pin to minimum `gh` version in docs | M5 |
| Plugin sandbox model | Documented in ADR-018; implementation in M7 | M7 |
| `kiwi_core` extraction timeline | SPEC-024 phases C1–C10; blocks GUI service wiring | M8 |
| GUI PTY surface crate | Spike in M8.3; wezterm-term vs custom ANSI | M8 |
