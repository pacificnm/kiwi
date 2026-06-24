# ADR-013: External Editor Strategy

## Status

Accepted

## Context

Kiwi does not edit files internally (ADR-001). Users must open files in their preferred editor with predictable resolution of which binary to launch and how arguments are passed.

## Decision

Implement an **editor launcher** that resolves command and spawns detached child process.

### Resolution order

1. `[editor] command` in config (project overrides user)
2. `$VISUAL`
3. `$EDITOR`
4. Fallback: `nano`

### Supported editors (tested targets)

Vim, Neovim, Helix, Nano, Micro, VS Code (`code`), Cursor (`cursor`), Zed (`zed`)

### Spawn behavior

```bash
# Conceptual
$EDITOR +line:col <absolute-path>   # if editor supports line args (detect or config)
$EDITOR <absolute-path>             # default
```

- Working directory: repository root
- Detach: do not block Kiwi main loop (`std::process::Command` with appropriate flags)
- GUI editors: spawn and return focus to Kiwi immediately
- Terminal editors: may steal terminal — document that user should use GUI editor or terminal multiplexer layout if simultaneous view needed

### Configuration

```toml
[editor]
command = "nvim"
args = ["+{line}", "{path}"]   # optional template; future enhancement
wait = false                    # never block on editor exit in v1
```

## Consequences

### Positive

- Simple, reliable, editor-agnostic
- Honors Unix conventions (VISUAL/EDITOR)
- No LSP or buffer management in Kiwi

### Negative

- Terminal editors block the terminal unless user runs Kiwi in split pane
- Line/column jump requires per-editor arg templates (partial in v1)
- No "dirty buffer" tracking in Kiwi

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Built-in $EDITOR picker TUI | Unnecessary; config + env sufficient |
| Always open VS Code | Violates editor-agnostic principle |
| Wait for editor close before continue | Blocks workflow |

## Follow-up Work

- SPEC-015 Editor Launcher
- Editor-specific arg presets in docs (not code) for common editors
- Optional `e` keybinding in file tree and preview
- Future: detect `$TERMINAL` editor vs GUI via config flag
