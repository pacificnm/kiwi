# Keyboard Shortcuts

Default keybindings for Kiwi. All shortcuts work when not captured by focused PTY (shell/agent). `?` opens help overlay (future) listing these bindings.

## Global

| Key | Action |
|-----|--------|
| `Ctrl+P` | Open command palette |
| `Tab` | Cycle focus forward (Left → Main → Palette → Shell) |
| `Shift+Tab` | Cycle focus backward |
| `q` | Quit (confirm if dirty future) |
| `Ctrl+C` | Quit if palette closed; else interrupt PTY |
| `?` | Help (future) |

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
| `↑` / `↓` | Move selection |
| `Enter` | Execute |
| `Esc` | Close |

## Shell / Agent (when focused)

| Key | Action |
|-----|--------|
| All keys | Forwarded to PTY |
| `PgUp` / `PgDn` | Scroll scrollback (when PTY not consuming) |

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
