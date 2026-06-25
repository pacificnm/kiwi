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

Both panes share the `ScrollbackBuffer` implementation (10_000 line cap):

- Completed lines end with `\n`
- The **current prompt / in-progress line** has no trailing newline and is shown from the pending buffer when following the tail
- ANSI SGR sequences are stripped for display
- `\r` (carriage return) keeps the segment after the last `\r` (terminal overwrite semantics)
- Tabs expand to spaces; each row is clipped to the pane width

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
- Agent status bar heuristics (#25)
- Agent restart command (#26)
- Bracketed paste forwarding into PTY (terminal enables paste; PTY routing TBD)
- Mouse wheel scroll in PTY panes

## Related

- [keyboard-shortcuts.md](../design/keyboard-shortcuts.md)
- [navigation.md](../design/navigation.md)
- [issue-resolution-log.md](./issue-resolution-log.md)
