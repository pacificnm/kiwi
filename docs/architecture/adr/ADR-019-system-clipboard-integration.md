# ADR-019: System Clipboard Integration

## Status

Accepted

## Context

Users expect standard `Ctrl+C` / `Ctrl+V` / `Ctrl+X` shortcuts to copy, paste, and cut between Kiwi panes (preview, search, palette, agent PTY, shell PTY) and the OS clipboard. Terminal TUIs on Linux (X11/Wayland) have additional constraints: the process that last wrote clipboard data must stay alive to serve paste requests from other applications.

Kiwi already routes keyboard input through a single app loop and uses `arboard` as a cross-platform clipboard backend (see workspace `Cargo.toml`).

## Decision

Integrate the **system clipboard** via a session-scoped `ClipboardService` and route copy/paste/cut through the existing reducer and command palette.

### Architecture

| Component | Responsibility |
|-----------|----------------|
| `clipboard/io.rs` | `ClipboardService` — one `arboard::Clipboard` for the app lifetime |
| `clipboard/keys.rs` | Map `Ctrl+C` / `Ctrl+V` / `Ctrl+X` (control only, no shift) |
| `clipboard/target.rs` | Resolve copy source and paste destination from focus + selection |
| `clipboard/paste.rs` | `pty_paste_bytes()` — raw text for single line; bracketed paste for multiline |
| `selection/` | In-app mouse highlight; preferred over pane fallback on copy |
| `commands/registry.rs` | Palette entries: Clipboard Copy / Cut / Paste |

### Copy routing

- **Mouse selection** in Preview, Agent, or Shell (when highlighted) wins over pane fallback.
- **Palette open** — copies palette input, or focused pane before open.
- **Per focus** — preview line, search query/result, file tree path, scrollback viewport, logs.

### Paste routing

| Focus | Destination |
|-------|-------------|
| Shell (running) | PTY write (`pty_paste_bytes`) |
| Agent (running) | PTY write |
| Search (left tab) | Append to query; reschedule debounced search |
| Command palette | Append to palette input |
| Other | Toast: paste not supported |

`crossterm::Event::Paste` is also routed through the same paste reducer.

### Shell and agent PTY exceptions

When the **shell** has focus:

- `Ctrl+C` **without** a shell text selection → PTY interrupt (`0x03`), not clipboard copy.
- `Ctrl+C` / `Ctrl+X` **with** highlighted shell text → clipboard copy/cut.
- `Ctrl+V` → read system clipboard and paste into PTY.

The **agent** pane uses `Ctrl+C` for copy (not interrupt); agent interrupt is not mapped to `Ctrl+C` in the current design.

### Linux clipboard ownership

Do **not** create and drop a new `Clipboard` per copy operation. A short-lived handle triggers `arboard` warnings on stderr (corrupting the alternate-screen TUI) and clipboard managers may never receive the data. Hold `ClipboardService` on `App` for the full session.

### Paste into PTY

- Single-line paste: send bytes directly (avoids bracketed-paste escape noise when the shell has not enabled mode 2004).
- Multiline paste: wrap with `\x1b[200~` … `\x1b[201~` per common terminal convention (SPEC-011).

## Consequences

### Positive

- Familiar desktop shortcuts work across panes.
- Copy/paste between agent, shell, preview, and external apps.
- Palette exposes the same operations for discoverability.
- Session-owned clipboard avoids Linux ownership bugs.

### Negative

- `arboard` may block or fail on headless or minimal Wayland setups; errors surface as toasts.
- Shell `Ctrl+C` semantics depend on whether text is selected (documented in keyboard shortcuts).
- Parallel clipboard access from multiple threads is discouraged by `arboard` on some platforms.

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Internal clipboard only | Cannot exchange with browser, editor, or OS |
| New `Clipboard` per operation | Breaks Linux/Wayland; stderr corrupts TUI |
| `Ctrl+Shift+C/V` only | User expectation is plain `Ctrl+C` / `Ctrl+V` |
| Always bracketed paste | Garbled input when shell lacks bracketed-paste mode |

## Follow-up Work

- Right-click context menu (copy/paste) — deferred
- Optional internal fallback buffer when OS clipboard unavailable
- SPEC-011 acceptance: multiline bracketed paste manual verification
- Design: [keyboard-shortcuts.md](../../design/keyboard-shortcuts.md)
