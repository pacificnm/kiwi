# GUI Implementation Plan

Phased plan for delivering `kiwi_gui` alongside the existing TUI. Prerequisites, milestones, and verification gates.

## Prerequisites

| Gate | Document | Status |
|------|----------|--------|
| Dual frontend decision | ADR-020 | Accepted |
| GUI stack selection | ADR-021 | Accepted |
| Dock architecture | ADR-022 | Accepted |
| Core extraction spec | SPEC-024 | Proposed |

**Hard prerequisite for shared services:** `kiwi_core` phases C1–C4 (config, theme, state, workspace) before GUI wires real data.

**Soft prerequisite:** TUI MVP (M1–M5) stable; GUI can scaffold earlier with placeholders.

## Architecture Summary

```text
Phase C (parallel)          Phase G (GUI)
─────────────────          ─────────────────────────────
kiwi_core extraction  →    kiwi_gui scaffold
  config/theme/state       empty window + theme
  services                 dock + placeholders
                           panel-by-panel wiring
kiwi (TUI) unchanged       feature parity targets
```

## Phase C: Core Extraction (SPEC-024)

| Step | Deliverable | Verification |
|------|-------------|--------------|
| C1 | `kiwi_core` crate, `config` migrated | `cargo test -p kiwi_core` |
| C2 | `theme` in core | Theme fixtures test |
| C3 | `events` + `state` + reducers | Reducer unit tests pass |
| C4 | `workspace` persistence types | Round-trip JSON test |
| C5–C10 | Services incrementally | TUI regression suite green |

**Rule:** Each step is a small PR; TUI behavior unchanged.

## Phase G0: GUI Scaffold

**Goal:** Empty desktop app proves toolchain.

| Task | Spec |
|------|------|
| Add `crates/kiwi_gui` to workspace | SPEC-021 |
| `eframe` window, title, quit | SPEC-021 |
| CI: `cargo build -p kiwi_gui` | ADR-021 |
| Pin egui/eframe/egui_dock versions | ADR-021 |

**Exit criteria:** `cargo run -p kiwi_gui` opens and closes cleanly.

## Phase G1: Bootstrap Parity

| Task | Spec |
|------|------|
| CLI args match TUI | SPEC-021, SPEC-018 |
| Config + theme load | SPEC-021, SPEC-023 |
| Status bar with repo/branch | SPEC-019, SPEC-023 |
| Tokio runtime + event channel | ADR-020 |

**Exit criteria:** Window shows correct repo name and theme colors.

## Phase G2: Dock Shell

| Task | Spec |
|------|------|
| `KiwiTab` enum + egui_dock | SPEC-022, ADR-022 |
| Default layout | [gui-layout.md](./gui-layout.md) |
| View menu show/hide tabs | SPEC-022 |
| Placeholder panels | SPEC-022 |
| Menu bar | SPEC-022 |

**Exit criteria:** User can drag tabs; layout resets; placeholders visible.

## Phase G3: Persistence

| Task | Spec |
|------|------|
| `gui` section in workspace JSON | SPEC-017, ADR-016 |
| Window geometry save | SPEC-021 |
| Dock tree serialize/deserialize | SPEC-022 |

**Exit criteria:** Restart restores tab arrangement.

## Phase G4: Panel Wiring (incremental)

Implement in order from SPEC-022 panel table:

1. **Logs** — validate event drain loop
2. **Explorer** — file tree from core
3. **Git Status** + **Diff** — git service
4. **Preview** + editor launch
5. **Terminal** + **Agent** — PTY surface spike
6. **GitHub** issues/PRs
7. **Search**
8. **Config** editor (read-only v1)

Each panel: read `AppState`, send `AppCommand`, preserve scroll/selection per ADR-007.

## Phase G5: Polish and Parity

| Task | Notes |
|------|-------|
| Command palette modal | SPEC-013 adapted |
| Keyboard shortcuts | [gui-keyboard-shortcuts.md](./gui-keyboard-shortcuts.md) |
| System clipboard | ADR-019 GUI path |
| Performance pass | Profile diff + search panels |
| Documentation | README install section for GUI |

## Milestone Mapping

Tracked as **M8: Desktop GUI** in [milestones.md](../roadmap/milestones.md).

| M8 sub-goal | Phases |
|-------------|--------|
| M8.1 Runnable GUI binary | G0–G1 |
| M8.2 Dock shell + persistence | G2–G3 |
| M8.3 Core panels | G4 (Explorer, Git, Terminal) |
| M8.4 Full MVP parity | G4–G5 |
| M8.5 Release-ready | CI, packaging, docs |

## Risk Register

| Risk | Mitigation |
|------|------------|
| Core extraction breaks TUI | Incremental PRs; full test suite each merge |
| PTY widget complexity | Spike early in G4; fallback to simpler scrollback |
| egui_dock API churn | Pin version; adapter module |
| Headless CI | Build-only job; unit tests without window |
| Two UIs diverge | Shared specs for domain behavior; GUI-only specs for chrome |

## Dependency Versions (initial pin target)

Verify latest compatible set at scaffold time:

```toml
egui = "0.31"
eframe = { version = "0.31", default-features = false, features = ["default_fonts", "glow", "persistence"] }
egui_dock = "0.16"
```

## What We Are Not Doing (v1)

- Replacing or removing TUI
- WASM / browser build
- Floating undocked OS windows per panel
- Embedded editor (still external)
- Feature parity with every TUI mouse affordance on day one

## Related

- [gui-layout.md](./gui-layout.md)
- [../architecture/adr/ADR-020-dual-frontend-architecture.md](../architecture/adr/ADR-020-dual-frontend-architecture.md)
- [../roadmap/backlog.md](../roadmap/backlog.md) epic E19–E22
