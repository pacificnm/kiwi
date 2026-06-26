# Navigation Design

## Dual Tab Model

Kiwi uses **two independent tab bars**. This is intentional: users keep file context visible while switching main workspace views.

Example combinations:

| Left Tab | Main Tab | Use Case |
|----------|----------|----------|
| Files | Agent | Implement with agent while browsing tree |
| Git | Diff | Review changes file-by-file |
| GH | Issues | Triage issues |
| Search | Preview | Find and read code |

## Left Navigation Tabs

### Files

- Primary repository browser
- Tree: `▸` collapsed, `▾` expanded
- Git status icon/color per file
- Keys: `j`/`k` move, `l` expand, `h` collapse, `Enter` preview, `e` editor

### Git

- Grouped sections: Modified, Added, Deleted, Untracked (if enabled)
- Select file → main Diff tab on `Enter` / double-click
- `R` refresh

### GH

- Hub sub-navigation: **Issues** | **PRs** (switch with `i` / `p` on left focus)
- **Issues** list (number, title, state, labels) — `j`/`k` navigate; mouse single-click select; double-click or `Enter` opens main **Issues** detail
- **PRs** placeholder until PR list lands (#59); `Enter` opens main **PRs** tab
- Auth errors and loading state shown inline when `gh` is unavailable
- `R` refresh

### Search

- Query input always visible when tab active
- Results replace tree area below input

## Main Workspace Tabs

### Agent

- Full PTY for Cursor Agent (or configured command)
- Agent status reflected in status bar
- With **Main** focus on the Agent tab, keyboard input is forwarded to the agent PTY
- Output renders in the main pane scrollback (including the prompt line before Enter)

### Shell (bottom pane)

- Full PTY for the user shell (default `$SHELL` or `bash`)
- With **Shell** focus, keyboard input is forwarded to the shell PTY
- Output renders in the shell pane scrollback (including the prompt line before Enter)
- Click the shell pane or cycle focus with `Tab` to interact

### Issues

- Detail view for the issue selected in the **GH** left tab
- Shows title, state, labels, assignees; body and comments (future)
- `Enter` on GH left list focuses main and opens detail
- Actions via palette: **Comment on Issue** (palette prompt), **Add Labels to Issue** (multi-select overlay), create branch (future)

### Branches

- Local branch list (current branch first, then alphabetical)
- Current branch marked with `*` and accent color
- `j`/`k` navigate; single-click select; double-click or `Enter` checks out branch
- `R` refresh; checkout errors shown in footer
- Disabled with inline message when not in a git repo

### PRs

- List with state badges (open/draft/merged/closed)
- Detail: description, checks summary
- Create PR via palette workflow

### Diff

- Unified diff for file selected from Git or Diff left tabs
- `n`/`p` next/previous file

### Preview

- Read-only file view
- Triggered explicitly from Files/Search

### Logs

- Kiwi application logs for debugging
- Filter: info/warn/error (future)

## Focus Model

Four focus targets cycle with `Tab`:

```text
Left → Main → Command Palette → Shell → Left
```

`Shift+Tab` reverses.

### Input routing

| Focus | Keys go to |
|-------|------------|
| Left | Tree/list navigation |
| Main + Agent tab | Agent PTY |
| Main + Issues tab | Issue detail (read-only scroll; actions via palette) |
| Main + other tabs | Tab content views (Diff, Preview, PRs, Logs) |
| Palette | Palette input |
| Shell | Shell PTY |

`Tab` / `Shift+Tab` always cycle focus, even when a PTY is active.

## Quick Switching

| Shortcut | Action |
|----------|--------|
| `Alt+1`–`Alt+4` | Left tab |
| `1`–`6` | Main tab (when main or left focused) |
| `Ctrl+P` | Command palette |
| `Ctrl+`` | Focus shell (optional) |

## Mouse Navigation

- Click tab labels to switch tabs **and move focus** (left tabs → Left focus, main tabs → Main focus)
- Click inside the agent pane (Agent tab) to focus Main
- Click inside the shell pane to focus Shell
- **Single click** list/tree row to select
- **Double click** file in Files tab → Preview tab; double click folder → expand
- **Double click** search result → Preview tab (content hits jump to line)
- **Left drag** in Preview, Agent, or Shell to highlight text for copy (`Ctrl+C`)

See [mouse-interaction.md](./mouse-interaction.md) and [keyboard-shortcuts.md](./keyboard-shortcuts.md).

## Related

- [layout.md](./layout.md)
- SPEC-004 Navigation System
