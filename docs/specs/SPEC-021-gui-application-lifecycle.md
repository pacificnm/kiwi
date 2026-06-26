# SPEC-021: GUI Application Lifecycle

## Purpose

Define how the `kiwi_gui` binary starts, runs, and shuts down using eframe, sharing configuration and services with the TUI.

## Scope

### In scope

- Binary entry point and CLI arguments
- eframe window creation and event loop
- Bootstrap: config load, theme resolve, tokio runtime, service spawn
- Graceful shutdown and workspace save
- Minimum window size and error surfaces

### Out of scope

- Individual panel content (SPEC-022)
- Theme visual mapping (SPEC-023)
- TUI lifecycle (SPEC-001)

## Related Documents

- ADR-020 Dual Frontend Architecture
- ADR-021 Desktop GUI Framework Selection
- SPEC-001 Startup Lifecycle
- SPEC-018 Configuration Loader

## Functional Requirements

1. **Binary name:** `kiwi-gui` (workspace crate `kiwi_gui`). Optional future: `kiwi --gui` dispatches from TUI binary.
2. **CLI parity** with TUI where applicable:
   - Positional repository path (default: current directory)
   - `--config` override path
   - `--theme` override name
   - `--version`, `--help`
3. **Startup sequence:**
   1. Parse CLI
   2. Resolve config (SPEC-018 merge order)
   3. Validate repository root
   4. Load workspace persistence (SPEC-017); apply GUI section if present
   5. Resolve theme roles (SPEC-003)
   6. Create tokio runtime handle
   7. Spawn async services (git, github, watcher, shell, agent as configured)
   8. Build `eframe::NativeOptions` (title, size, min size, persistence id)
   9. Run `eframe::run_native` with `KiwiApp` implementing `eframe::App`
4. **Window defaults:**
   - Title: `Kiwi — {repo_name}`
   - Size: 1400×900 or restored from persistence
   - Min size: 800×600
5. **Per-frame loop** (`KiwiApp::update`):
   1. Drain `AppEvent` channel (bounded batch per frame)
   2. Apply reducers to `AppState`
   3. Render menu bar, dock (SPEC-022), status bar, optional command palette
   4. Request repaint if services pending
6. **Shutdown:**
   - On window close or Quit command: save workspace (GUI section), shutdown services, exit 0
   - On fatal error: show native dialog or in-window error, exit non-zero
7. **Headless / CI:** `cargo build -p kiwi_gui` succeeds without display; integration tests use mocked state, not eframe window.

## Non-Functional Requirements

- Cold start to first frame < 2s on typical dev machine
- No `unsafe` (workspace policy)
- Memory: do not retain unbounded PTY scrollback in GUI structures beyond TUI limits
- Single instance per repo optional (defer; document as future)

## Data Structures

```rust
pub struct KiwiApp {
  state: AppState,
  event_rx: mpsc::Receiver<AppEvent>,
  command_tx: CommandSender,
  dock: DockState,
  gui_theme: GuiTheme,
  runtime: tokio::runtime::Handle,
  // services handles for shutdown
}

pub struct GuiBootstrapContext {
  pub config: ResolvedConfig,
  pub repo_root: PathBuf,
  pub persistence: WorkspaceSnapshot,
  pub theme: ResolvedTheme,
}
```

## Events / Commands

| Input | Action |
|-------|--------|
| Window close | `AppCommand::Quit` → save, shutdown |
| `Ctrl+Q` / File → Quit | Same as close |
| `AppEvent::ConfigLoaded` | Initialize state |
| Service events | Standard reducers (SPEC-016) |

## Configuration Options

```toml
[gui]
enabled = true          # informational; binary presence is the switch
default_width = 1400
default_height = 900
min_width = 800
min_height = 600
vsync = true

[gui.font]
size = 14.0             # base egui font scale
```

Existing `[app]` and `[theme]` sections apply unchanged.

## Error Handling

| Condition | Behavior |
|-----------|----------|
| Invalid repo path | Error dialog; exit 1 |
| Config parse error | Message with path; exit 1 |
| eframe init failure | stderr message; exit 1 |
| Service spawn failure | Degraded mode; status bar warning |
| Persistence corrupt | Log warning; default layout |

## Acceptance Criteria

- [ ] `cargo run -p kiwi_gui -- .` opens a window in a graphical session
- [ ] Window title includes repository name
- [ ] Config and theme match TUI for same `config.toml`
- [ ] Quit saves GUI workspace section to XDG state path
- [ ] `cargo build --workspace` includes `kiwi_gui` without errors
- [ ] Services emit events visible in status bar within 5s of open
- [ ] Min window size enforced; content scrolls rather than panics

## Implementation Phases

| Phase | Deliverable |
|-------|-------------|
| G0 | Empty `kiwi_gui` crate, blank window, quit works |
| G1 | Config + theme load, status bar with repo name |
| G2 | Service spawn + event drain loop |
| G3 | Persistence save/load for window geometry |
