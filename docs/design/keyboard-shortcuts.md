# Keyboard Shortcuts

Default keybindings for Kiwi. Most shortcuts apply when focus is not in a PTY pane (shell or agent). `Tab` / `Shift+Tab` always cycle focus. `?` opens help overlay (future) listing these bindings.

## Global

| Key | Action |
|-----|--------|
| `Ctrl+P` | Open command palette |
| `Tab` | Cycle focus forward (Left → Main → Palette → Shell) |
| `Shift+Tab` | Cycle focus backward |
| `q` | Quit (when shell/agent PTY is not consuming input) |
| `Ctrl+C` | Quit (when shell/agent PTY is not consuming input); otherwise interrupt PTY |
| `Ctrl+Q` | Quit (always, including from shell/agent) |
| `?` | Help (future) |

When the shell or agent PTY has keyboard focus, `Ctrl+C` once sends an interrupt to the running process. Press `Ctrl+C` twice within 500ms or use `Ctrl+Q` to quit Kiwi.

## Left Navigation Tabs

| Key | Action |
|-----|--------|
| `Alt+1` | Files |
| `Alt+2` | Git |
| `Alt+3` | Diff |
| `Alt+4` | GH |
| `Alt+5` | Search |

## Main Workspace Tabs

| Key | Action |
|-----|--------|
| `1` | Agent |
| `2` | Issues |
| `3` | PRs |
| `4` | Diff |
| `5` | Preview |
| `6` | Logs |

## Agent (main, Agent tab)

| Key | Action |
|-----|--------|
| `Ctrl+Shift+R` | Restart agent |

When the agent process exits, the pane footer shows the exit code and the restart shortcut.

## Files (left, Files tab)

| Key | Action |
|-----|--------|
| `j` / `k` | Down / up |
| `h` / `l` | Collapse / expand directory |
| `Enter` | Preview file in main tab |
| `e` | Open in external editor |
| `r` | Refresh tree |
| `g g` | Go to root (future) |

## Git (left, Git tab)

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection |
| `Enter` | Open diff in main tab |
| `R` | Refresh git status |

## Diff (main tab)

| Key | Action |
|-----|--------|
| `n` / `p` | Next / previous file |
| `s` | Toggle staged/unstaged |
| `j` / `k` | Scroll diff |

## Search (left, Search tab)

| Key | Action |
|-----|--------|
| `/` | Focus search input |
| `Ctrl+M` | Toggle file/content mode |
| `Enter` | Open selection |
| `e` | Open in editor |
| `Esc` | Clear query |

## Command Palette

| Key | Action |
|-----|--------|
| `Ctrl+P` | Open command palette |
| `↑` / `↓` | Move selection (or cycle recent commands when input is empty) |
| `Enter` | Execute selected command |
| `Esc` | Close palette and restore previous focus |

## Shell / Agent (when focused)

| Key | Action |
|-----|--------|
| Most keys | Forwarded to PTY |
| `PgUp` / `PgDn` | Scroll scrollback |
| `Tab` / `Shift+Tab` | Cycle focus (not forwarded to PTY) |
| `Ctrl+Q` | Quit Kiwi |

## GitHub (Issues/PRs main tab)

| Key | Action |
|-----|--------|
| `j` / `k` | List navigation |
| `Enter` | Open detail |
| `c` | Comment (opens palette prompt) |
| `o` | Open in browser |
| `R` | Refresh list |

## Customization

Future: `~/.config/kiwi/keymap.toml`. MVP uses compiled defaults only.

## Related

- [navigation.md](./navigation.md)
- SPEC-004, SPEC-013
