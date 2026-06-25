# SPEC-004: Navigation System

## Purpose

Manage left navigation tabs, main workspace tabs, focus routing, and keyboard/mouse tab switching.

## Scope

### In scope

- Tab state machines
- Focus model
- Keybindings for tab switch
- Orthogonal left vs main selection

### Out of scope

- Closable/dynamic main tabs (future)

## Functional Requirements

### Left navigation tabs (order fixed)

1. Files — file tree
2. Git — git status list
3. Diff — diff file list
4. GH — GitHub issue list (navigate; `Enter` opens main Issues detail)
5. Search — search UI

### Main workspace tabs (order fixed)

1. Agent — agent PTY
2. Issues — GitHub issue detail (list lives in GH left tab)
3. PRs — GitHub PR list/detail
4. Diff — unified diff viewer
5. Preview — read-only file preview
6. Logs — application logs

### Focus targets

`Left`, `Main`, `CommandPalette`, `Shell` — exactly one active.

### Keybindings (defaults)

| Key | Action |
|-----|--------|
| `Alt+1`–`Alt+5` | Left nav tab |
| `1`–`6` (main focused) | Main tab |
| `Tab` | Cycle focus: Left → Main → Palette → Shell |
| `Shift+Tab` | Reverse cycle |
| `Ctrl+P` | Focus command palette |

Mouse: click tab label to activate (SPEC-014).

## Non-Functional Requirements

- Tab switch < 16ms perceived (single frame)
- State preserved per tab when switching away

## Data Structures

```rust
enum LeftNavTab { Files, Git, Diff, Gh, Search }
enum MainTab { Agent, Issues, Prs, Diff, Preview, Logs }
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
- [ ] Tab highlights reflect selection
- [ ] Focus border matches focused pane
- [ ] Mouse click switches tabs
- [ ] Tab state preserved when switching back
