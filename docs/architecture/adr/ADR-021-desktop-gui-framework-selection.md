# ADR-021: Desktop GUI Framework Selection

## Status

Accepted

## Context

Kiwi needs a native desktop GUI for an IDE-like workspace: file explorer, Git panels, GitHub issues, diff viewer, embedded terminals, and AI agent chat. The GUI must run on Linux and macOS (Windows desirable), integrate with Rust async I/O for PTY and subprocesses, and feel responsive for developer-tool density.

ADR-020 establishes `kiwi_gui` as a separate frontend sharing `kiwi_core`. This ADR selects the GUI stack.

## Decision

Use **egui** for immediate-mode widgets, **eframe** as the native application shell, and **egui_dock** for VS Code–style dockable panels.

```toml
# Workspace dependencies (pin at scaffold time)
egui = "0.31"
eframe = { version = "0.31", default-features = false, features = ["default_fonts", "glow", "persistence"] }
egui_dock = "0.16"
```

### Component roles

| Crate | Responsibility |
|-------|----------------|
| [egui](https://github.com/emilk/egui) | Widgets, layouts, input, styling, immediate-mode render loop |
| [eframe](https://docs.rs/eframe/latest/eframe/) | Native window, event loop, persistence hooks, wgpu/glow backend |
| [egui_dock](https://github.com/Adanos020/egui_dock) | Draggable tab strips, split panes, dock trees |

### Reference implementations

- [egui demo app](https://github.com/emilk/egui/tree/master/crates/egui_demo_app) — menus, tables, trees, themes, popups
- [egui_dock examples](https://github.com/Adanos020/egui_dock) — IDE-style layouts

### Rendering backend

Start with **glow** (OpenGL) for broad Linux compatibility. Evaluate **wgpu** if Wayland/multi-monitor issues arise. eframe feature flags allow switching without changing panel code.

### Community crates (evaluated per panel)

| Need | Candidate | When |
|------|-----------|------|
| File tree / open dialog | `rfd`, `egui-file-dialog`, custom tree | M8 panel implementation |
| Syntax-highlighted preview | `syntect` + egui integration | Post-parity enhancement |
| Terminal emulator | `wezterm-term` / custom ANSI surface | Shell and agent panels |
| Tables (issues, git status) | egui `TableBuilder` | GitHub and Git panels |

No additional GUI framework layers (e.g., egui_mate, bevy_ui) in v1.

### PTY in GUI

PTY output renders in dedicated dock tabs (Terminal, Agent). Unlike the TUI ANSI viewport (SPEC-011), the GUI may use a scrollable text buffer with ANSI parsing or embed a terminal widget crate. PTY **I/O and session management** stay in `kiwi_core` (ADR-006); only the surface differs.

### Async integration

- Tokio runtime created at GUI startup (same as TUI)
- Services unchanged: spawn tasks, send `AppEvent` to a channel
- eframe `App::update` drains the channel each frame before drawing dock tabs
- Long operations show egui progress indicators; never block the UI thread on subprocess I/O

## Consequences

### Positive

- egui is widely used for Rust developer tools; active ecosystem
- Immediate mode fits panel-based IDE layout; less state sync than retained-mode toolkits
- egui_dock provides draggable tabs without custom layout engine
- Same language and async stack as the rest of Kiwi
- Online demo and examples accelerate prototyping

### Negative

- Immediate-mode UIs require discipline to avoid per-frame allocation hot spots
- egui default look differs from terminal aesthetic; theme bridge required (SPEC-023)
- egui_dock API may change across minor versions; pin and review changelogs
- No built-in terminal widget; custom PTY surface is non-trivial
- eframe adds GPU/OpenGL dependency; headless CI needs careful test design

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| **iced** | Retained-mode; docking IDE layout harder; smaller widget ecosystem for dense dev UIs |
| **Slint** | Declarative DSL; less flexible for dynamic dock layouts |
| **Tauri + web frontend** | Heavier runtime; duplicates stack; terminal PTY embedding awkward |
| **GTK (gtk4-rs)** | Steeper learning curve; docking requires custom chrome |
| **Qt (cxx-qt)** | C++ dependency; licensing and build complexity |
| **ratatui in a terminal widget** | Nested terminal inside GUI is poor UX; does not leverage GUI affordances |

## Follow-up Work

- SPEC-021: GUI application lifecycle and eframe bootstrap
- SPEC-022: Dock layout engine with `KiwiTab` enum
- SPEC-023: Theme bridge from semantic roles to `egui::Visuals`
- Spike: PTY text surface prototype in a dock tab
- Pin exact crate versions in workspace `Cargo.toml` at scaffold
- Add `kiwi_gui` to CI `cargo build --workspace` with display-less smoke test
