# Keyboard Shortcuts

Default keybindings for Kiwi. Most shortcuts apply when focus is not in a PTY pane (shell or agent). `Tab` / `Shift+Tab` always cycle focus. `?` opens help overlay (future) listing these bindings.

## Global

| Key | Action |
|-----|--------|
| `Ctrl+P` | Open command palette |
| `Tab` | Cycle focus forward (Left → Main → Palette → Shell) |
| `Shift+Tab` | Cycle focus backward |
| `q` | Quit (when shell/agent PTY is not consuming input) |
| `Ctrl+Q` | Quit (always, including from shell/agent) |
| `Ctrl+C` | Copy highlighted text in shell; otherwise **interrupt** running command |
| `Ctrl+X` | Cut highlighted shell text; otherwise forwarded to the shell |
| `Ctrl+V` | Paste into agent PTY, shell, palette input, or search query |
| `?` | Help (future) |

When the shell has keyboard focus, `Ctrl+C` sends an interrupt to the running process (standard terminal behavior). Press `Ctrl+C` twice within 500ms or use `Ctrl+Q` to quit Kiwi from the shell.

## Mouse text selection

| Action | Result |
|--------|--------|
| Left-click + drag | Highlight text in Preview, Agent, or Shell panes |
| `Ctrl+C` | Copy highlighted text (or pane fallback when nothing selected) |
| `Ctrl+V` | Paste into agent, shell, palette, or search |

## GH list context menu (right click)

Right-click an issue or PR row in the **GH** left pane (Issues or PRs hub) to open an action menu. See [mouse-interaction.md](./mouse-interaction.md) for full mouse behavior.

| Action | Result |
|--------|--------|
| Right click | Open context menu on GH issue/PR list row (selects row) |
| Left click (menu open) | Activate highlighted item, or dismiss if outside menu |
| `Esc` | Dismiss menu |
| `j` / `k` or `Up` / `Down` | Move menu highlight |
| `Enter` | Execute highlighted action |

Issue rows: **View**, **Create Branch**, **Comment**, **Add Labels**, **Open in Browser**, **Send To Agent**. PR rows: **View**, **Merge into main** (when mergeable), **Open in Browser**, **Send To Agent**.

## Clipboard

| Key | Action |
|-----|--------|
| `Ctrl+C` | Copy from focused pane |
| `Ctrl+X` | Cut where editable (palette/search query) |
| `Ctrl+V` | Paste into agent, shell, palette, or search |

Copy from preview, search, or logs, then focus the **Agent** tab and press `Ctrl+V` to paste into the agent PTY. Same for the **shell** pane.

Terminal emulator paste (`Event::Paste`) is also routed into the focused pane.

Palette commands: **Clipboard: Copy**, **Clipboard: Cut**, **Clipboard: Paste**.

## Left Navigation Tabs

| Key | Action |
|-----|--------|
| `Alt+1` | Files |
| `Alt+2` | Git |
| `Alt+3` | GH |
| `Alt+4` | Search |

## Main Workspace Tabs

| Key | Action |
|-----|--------|
| `1` | Agent |
| `2` | Issues |
| `3` | Branches |
| `4` | PRs |
| `5` | Diff |
| `6` | Preview |
| `7` | Logs |
| `8` | Settings |
| `9` | Plugins (Plugin Manager) |

## Settings (main, Settings tab)

| Key | Action |
|-----|--------|
| `j` / `k` | Down / up theme list |
| `Enter` | Apply selected theme |

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
| Double-click | Preview file in main tab |
| `e` | Open in external editor |
| `r` | Refresh tree |
| `g g` | Go to root (future) |

## Git (left, Git tab)

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection |
| `Enter` | Open diff in main tab |
| `R` | Refresh git status |

## Branches (main, Branches tab)

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection |
| `Enter` | Checkout selected branch |
| Double-click | Checkout branch |
| `R` | Refresh branch list |

## Diff (main tab)

| Key | Action |
|-----|--------|
| `n` / `p` | Next / previous file |
| `s` | Toggle staged/unstaged |
| `j` / `k` | Scroll diff |

## Search (left, Search tab)

| Key | Action |
|-----|--------|
| `/` (global) | Focus Search tab and input |
| `/` (while focused) | Ignored (use query prefix in input) |
| `Ctrl+M` | Toggle file/content mode |
| `Enter` | Open selection in Preview (content hits jump to line) |
| Double-click | Open selection in Preview (same as Enter) |
| `e` | Open in editor |
| `j` / `k` | Move selection |
| `Esc` | Clear query |

## Preview (main, Preview tab)

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll |
| `PgUp` / `PgDn` | Page scroll |
| `e` | Open in external editor at visible line |

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

## GitHub (GH left pane + Issues/PRs main tab)

Pair **GH** left (`Alt+3`) with **Issues** main (`2`) or **PRs** main (`4`). Use `i` / `p` on the GH left hub to switch between issue and PR lists.

| Key | Action |
|-----|--------|
| `i` / `p` | Switch GH left hub between Issues and PRs lists |
| `j` / `k` | Issue list navigation (GH left, Issues hub) or scroll detail (Issues main, main focus) |
| `PgUp` / `PgDn` | Page scroll issue detail (Issues main, main focus) |
| `Enter` | Open selected issue in Issues main tab |
| Right click | Open context menu on GH issue/PR list row (see **GH list context menu** above) |
| Command palette | **GitHub: Comment on Issue** — prompt for comment text (`Enter` posts) |
| Command palette | **GitHub: Add Labels to Issue** — multi-select overlay (`Space` toggle, `Enter` apply, `Esc` cancel) |
| Command palette | **GitHub: Open in Browser** — opens selected issue or PR via `gh view --web` |
| Command palette | **GitHub: Create Pull Request** — guided title → body → base prompts (`gh pr create`) |
| Command palette | **GitHub: Merge Pull Request** — merge selected open PR via `gh pr merge --merge` |
| `o` | Open selected issue or PR in browser |
| `R` | Refresh list |

## Customization

Future: `~/.config/kiwi/keymap.toml`. MVP uses compiled defaults only.

## Related

- [navigation.md](./navigation.md)
- SPEC-004, SPEC-013, SPEC-014, ADR-019
