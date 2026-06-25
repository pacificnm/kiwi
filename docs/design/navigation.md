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
- Select file → optional auto-switch to main Diff tab (configurable future)
- `R` refresh

### Diff

- Flat list of changed files only
- Faster navigation when only caring about diffs
- Select → main Diff tab shows content

### GH

- Hub for GitHub: buttons/links to jump main tab to Issues or PRs
- Shows open issue/PR counts
- Auth status indicator

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

- List view default; `Enter` opens detail
- Detail: title, body, labels, comments
- Actions via palette: comment, label, branch

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
| Main + other tabs | Tab placeholder views (future content) |
| Palette | Palette input |
| Shell | Shell PTY |

`Tab` / `Shift+Tab` always cycle focus, even when a PTY is active.

## Quick Switching

| Shortcut | Action |
|----------|--------|
| `Alt+1`–`Alt+5` | Left tab |
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
