# SPEC-004: Navigation System

## Purpose

Manage left navigation tabs, main workspace tabs, focus routing, and keyboard/mouse tab switching.

## Scope

### In scope

- Tab state machines
- Focus model
- Keybindings for tab switch
- Orthogonal left vs main selection, with contextual pairing when a main tab is selected

### Out of scope

- Closable/dynamic main tabs (future)

## Functional Requirements

### Left navigation tabs (order fixed)

1. Files — file tree
2. Git — git status list (changed files; `Enter` opens main Diff tab)
3. GH — GitHub issue list (navigate; `Enter` opens main Issues detail)
4. Search — search UI

### Main workspace tabs (order fixed)

1. Agent — agent PTY
2. Issues — GitHub issue detail (list lives in GH left tab)
3. Branches — local branch list and checkout
4. PRs — GitHub PR list/detail
5. Diff — unified diff viewer
6. Preview — read-only file preview
7. Logs — application logs

### Focus targets

`Left`, `Main`, `CommandPalette`, `Shell` — exactly one active.

### Keybindings (defaults)

| Key | Action |
|-----|--------|
| `Alt+1`–`Alt+4` | Left nav tab |
| `1`–`7` (main focused) | Main tab |
| `Tab` | Cycle focus: Left → Main → Palette → Shell |
| `Shift+Tab` | Reverse cycle |
| `Ctrl+P` | Focus command palette |

Mouse: click tab label to activate (SPEC-014).

### Main tab → left tab pairing

Selecting a **main** tab (mouse, digit shortcut, or command palette) auto-activates a paired **left** tab when one exists. Selecting a **left** tab alone does not change the main tab.

| Main tab | Paired left tab |
|----------|-----------------|
| Issues | GH |
| PRs | GH |
| Branches | GH |
| Preview | Files |
| Diff | Git |
| Agent, Logs | (none — left tab unchanged) |

When main **Issues** or **PRs** is selected, the GH left sub-pane (`Issues` / `PRs` list mode) syncs to match.

## Non-Functional Requirements

- Tab switch < 16ms perceived (single frame)
- State preserved per tab when switching away

## Data Structures

```rust
enum LeftNavTab { Files, Git, Gh, Search }
enum MainTab { Agent, Issues, Branches, Prs, Diff, Preview, Logs }
enum FocusTarget { Left, Main, CommandPalette, Shell }

struct NavigationState {
    left_tab: LeftNavTab,
    main_tab: MainTab,
    focus: FocusTarget,
}
```

## Events / Commands

```rust
AppCommand::SelectLeftTab(LeftNavTab)
AppCommand::SelectMainTab(MainTab)
AppCommand::SetFocus(FocusTarget)
AppCommand::NextFocus / PreviousFocus
```

## Configuration Options

None in v1; keybindings file future.

## Error Handling

- Invalid tab index ignored
- GH tab without gitHub auth shows inline setup, not panic

## Acceptance Criteria

- [ ] Can view Files tree while Agent tab active in main
- [ ] Selecting Issues/PRs/Preview/Diff/Branches main tab activates paired left tab
- [ ] Selecting Agent or Logs main tab does not force left tab change
- [ ] Tab highlights reflect selection
- [ ] Focus border matches focused pane
- [ ] Mouse click switches tabs
- [ ] Tab state preserved when switching back
