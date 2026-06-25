# ADR-013: External Editor Strategy

## Status

Accepted

## Context

Kiwi does not edit files internally (ADR-001). Users must open files in their preferred editor with predictable resolution of which binary to launch and how arguments are passed.

## Decision

Implement an **editor launcher** that resolves the command and launches it using one of two strategies depending on editor type.

### Resolution order

1. `[editor] command` in config (project overrides user)
2. `$VISUAL`
3. `$EDITOR`
4. Fallback: `nano`

### Supported editors (tested targets)

Vim, Neovim, Helix, Nano, Micro, VS Code (`code`), Cursor (`cursor`), Zed (`zed`)

### Terminal vs GUI launch

Kiwi classifies the resolved editor command and chooses a launch path:

| Class | Examples | Behavior |
|-------|----------|----------|
| **Terminal** | `vim`, `nvim`, `nano`, `micro`, `hx` | **Suspend** Kiwi (leave alternate screen, disable raw mode), run editor on the controlling TTY with inherited stdio, **wait** for exit, then **resume** Kiwi |
| **GUI** | `code`, `cursor`, `zed` | **Detached** spawn with null stdio; Kiwi keeps the terminal and redraws immediately |

Classification uses the command basename against known GUI editors. Unknown commands default to **terminal** (foreground on the TTY). Override with config:

```toml
[editor]
command = "my-editor"
terminal = false   # force detached/GUI-style launch
```

### Spawn behavior

```bash
# Terminal editor (Kiwi suspended)
$EDITOR +N <absolute-path>    # vim family when line provided; blocks until exit

# GUI editor (Kiwi running)
$EDITOR <absolute-path>       # detached; Kiwi stays responsive
```

- Working directory: repository root
- Absolute path argument; error if file missing
- Terminal editors block the Kiwi event loop only while the editor session runs (expected)
- GUI editors: spawn and return focus to Kiwi immediately
- Log successful launches to the Logs tab at info level

### Configuration

```toml
[editor]
command = "nvim"
terminal = true              # optional; overrides auto-detection
args = ["+{line}", "{path}"] # optional template; future enhancement
```

## Consequences

### Positive

- Simple, reliable, editor-agnostic
- Honors Unix conventions (VISUAL/EDITOR)
- Terminal editors (nano, vim) work in the same terminal session as Kiwi
- No LSP or buffer management in Kiwi

### Negative

- Kiwi is unresponsive while a terminal editor runs (by design)
- Shell/agent PTY panes do not redraw during editor suspend (they resume on return)
- Line/column jump requires per-editor arg templates (partial in v1)
- No "dirty buffer" tracking in Kiwi

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Built-in $EDITOR picker TUI | Unnecessary; config + env sufficient |
| Always open VS Code | Violates editor-agnostic principle |
| Detached spawn for all editors | Terminal editors cannot attach with null stdio |
| Built-in editor tab in Kiwi | Violates ADR-001 orchestrator scope |
| Always wait for editor close (GUI) | Blocks workflow for windowed editors |

## Follow-up Work

- SPEC-015 Editor Launcher
- Editor-specific arg presets in docs (not code) for common editors
- Optional `e` keybinding in file tree and preview
- Helix `path:line` arg template
- Optional preview reload after terminal editor exits
