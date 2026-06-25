# PTY Panes (Agent and Shell)

How embedded PTY sessions behave in the current M2 implementation. See SPEC-010 (agent) and SPEC-011 (shell).

## Layout

```text
┌────────────────────────────┬─────────────────────────────────────────────┐
│ Left nav                   │ Main tabs                                   │
├────────────────────────────┼─────────────────────────────────────────────┤
│ Left content               │ Main content (Agent PTY scrollback on Agent)  │
├────────────────────────────┼─────────────────────────────────────────────┤
│ Commands (palette)         │ Shell PTY scrollback                        │
└────────────────────────────┴─────────────────────────────────────────────┘
Status bar
```

## Focus Model

Four focus targets cycle with `Tab` / `Shift+Tab` **always**, even when a PTY is active:

```text
Left → Main → Command Palette → Shell → Left
```

| Focus | Keyboard input |
|-------|----------------|
| Main + **Agent** tab | Agent PTY |
| **Shell** | Shell PTY (`/bin/bash` or `$SHELL`) |
| Left / Palette | Navigation and palette (future input) |

Mouse:

- Click **main tab** → switch tab and focus **Main**
- Click **left tab** → switch tab and focus **Left**
- Click **agent pane** (Agent tab) → focus **Main**
- Click **shell pane** → focus **Shell**

## Scrollback Display

Both panes share the `ScrollbackBuffer` implementation (10_000 line cap). The buffer is a **minimal PTY screen emulator**, not a naive line log — agent and shell use identical code paths.

### Screen model

- **History** — committed rows scrolled off the active screen
- **Screen** — cursor-addressable grid with row/column position
- **Tail follow** — when `follow_tail` is true, the in-progress row at the cursor is included in the viewport even without a trailing `\n`

### Text and encoding

- PTY bytes are accumulated in a UTF-8 pending buffer and decoded before display; split reads across the I/O thread are reassembled
- Invalid UTF-8 sequences render as U+FFFD
- `\r` overwrites from column 0 on the current row; `\t` expands to spaces; `\b` moves the cursor back

### Escape sequences

- **CSI** — clear screen/line (`J`, `K`), cursor position (`H`/`f`), cursor movement (`A`–`D`), SGR color (`m` preserved in line text)
- **Private modes** — `?25h`/`?25l`, `?2004h`, `?1049h`/`?1049l`, etc. are parsed and ignored (not printed)
- **Split reads** — incomplete escapes are held in a pending buffer until the next PTY chunk arrives
- **Non-CSI** — short sequences such as `\x1b(B` (charset) are consumed without rendering

### Rendering and colors

- Each viewport row is clipped to pane width (ANSI-aware visible width)
- **PTY content** uses host terminal colors via `Color::Reset` and an SGR parser (`ansi.rs`) — child ANSI is not remapped to the Kiwi theme (see [themes.md](../design/themes.md))
- **Chrome** (borders, tabs, status bar, hints) uses `ThemePalette`

For symptom → cause → fix notes on scrollback bugs, see [issue-resolution-log.md](./issue-resolution-log.md).

## Quit and Interrupt

| Key | Behavior |
|-----|----------|
| `q` | Quit when focus is not routing keys to a running PTY |
| `Ctrl+C` | Quit when not in PTY; otherwise interrupt the PTY process |
| `Ctrl+C` twice (500ms) | Force quit from shell or agent |
| `Ctrl+Q` | Force quit from anywhere |

Shutdown restores the host terminal before tearing down PTY children. SIGINT/SIGTERM trigger the same clean shutdown path.

## Configuration

```toml
[shell]
command = "bash"   # defaults to $SHELL or bash
args = []

[agent]
command = "agent"
args = []
```

Bash is spawned with `-i` when not already specified so an interactive prompt appears in the PTY.

## Not Yet Implemented (see backlog)

- Agent PTY resize on terminal resize (# follow-up to SPEC-010)
- Bracketed paste forwarding into PTY (terminal enables paste; PTY routing TBD)
- Mouse wheel scroll in PTY panes

Implemented: agent status bar heuristics (#25), agent restart `Ctrl+Shift+R` (#26), command palette registry and UI (#27). Follow-ups: palette command persistence (#29), bracketed paste into PTY.

## Related

- [keyboard-shortcuts.md](../design/keyboard-shortcuts.md)
- [navigation.md](../design/navigation.md)
- [issue-resolution-log.md](./issue-resolution-log.md)
