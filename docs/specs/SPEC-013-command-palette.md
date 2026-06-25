# SPEC-013: Command Palette

## Purpose

Provide fuzzy-searchable command execution interface in the bottom-left panel per ADR-014.

## Scope

### In scope

- Command mode (`Ctrl+P`)
- Static command registry
- Fuzzy filter
- Keyboard and mouse execution

### Out of scope

- User-defined macros (future)
- Plugin commands until M7

## Functional Requirements

1. Open palette: `Ctrl+P`; close: `Esc`.
2. Input field with fuzzy match against command title and id.
3. Show top 10 matches with shortcut hints.
4. `Enter` or click executes command.
5. Commands include: navigation, git refresh, github refresh, editor open, agent restart, quit, focus panes.
6. Context-aware commands marked unavailable grayed (wrong panel focused).
7. History: up/down cycles recent commands (from persistence).

## Non-Functional Requirements

- Filter update < 5ms for 100 commands (see `commands::fuzzy` perf test)
- Palette open does not lose shell scrollback state

## Data Structures

```rust
struct CommandDef {
    id: &'static str,
    title: &'static str,
    shortcut: Option<&'static str>,
    context: CommandContext,
    action: AppCommand,
}

struct CommandPaletteState {
    open: bool,
    input: String,
    matches: Vec<usize>,  // indices into registry
    selected: usize,
    history: Vec<String>,
}
```

## Events / Commands

```rust
AppCommand::PaletteOpen
AppCommand::PaletteClose
AppCommand::PaletteSetInput(String)
AppCommand::PaletteExecuteSelected
```

## Configuration Options

None v1.

## Error Handling

- Execute failure: toast in status area or logs tab entry
- Empty input shows recent/hint list

## Acceptance Criteria

- [x] Ctrl+P opens palette
- [x] Fuzzy find "git ref" matches "Git: Refresh Status"
- [x] Enter executes and closes palette
- [x] Esc restores previous focus
- [x] Mouse click executes command
