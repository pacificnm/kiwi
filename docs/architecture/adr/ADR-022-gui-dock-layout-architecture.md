# ADR-022: GUI Dock Layout Architecture

## Status

Accepted

## Context

The TUI uses a fixed hierarchical layout with orthogonal left-nav and main tabs (ADR-003). A desktop GUI benefits from user-rearrangeable panels—explorer beside diff, terminal below chat, etc.—similar to VS Code.

ADR-021 selects egui_dock. This ADR defines how docking maps to Kiwi’s functional areas without coupling domain logic to dock internals.

## Decision

Implement the GUI shell as an **egui_dock tree** where each leaf tab is a **`KiwiTab`** variant. Domain state lives in `kiwi_core`; dock structure lives in `kiwi_gui`.

### Default layout (first run)

```text
┌──────────────────────────────────────────────────────────────────┐
│ Menu: File  View  Git  Help                          [─ □ ×]      │
├──────────────┬───────────────────────────────────────────────────┤
│ Explorer     │ Agent                                             │
│ Git Status   │                                                   │
│ GitHub       │                                                   │
│              │                                                   │
├──────────────┴───────────────────────────────────────────────────┤
│ Terminal                                                         │
└──────────────────────────────────────────────────────────────────┘
│ Status: branch · repo · agent · theme                            │
└──────────────────────────────────────────────────────────────────┘
```

Suggested default dock tree (conceptual):

- **Left stack:** Explorer, Git Status, GitHub Issues (narrow, ~20% width)
- **Center:** Agent (or Preview) as primary workspace
- **Bottom:** Terminal (~25% height)

Users may drag tabs to new edges, split horizontally/vertically, and close/reopen tabs via View menu.

### Tab model

```rust
/// GUI-only; not used by TUI navigation (SPEC-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KiwiTab {
  Explorer,
  GitStatus,
  GitDiff,
  GitLog,
  GitHubIssues,
  GitHubPrs,
  Preview,
  Search,
  Terminal,
  Agent,
  AiChat,   // alias or evolution of Agent; one PTY tab type at launch
  Config,
  Logs,
}
```

Multiple instances of the same tab kind are **not** supported in v1 (e.g., one Terminal tab). Reopening a closed tab restores the singleton.

### Tab ↔ core state mapping

| KiwiTab | Core state slice | Service commands |
|---------|------------------|------------------|
| Explorer | `file_tree` | `FileTreeCommand` |
| GitStatus | `git.status` | `GitCommand` |
| GitDiff | `git.diff` | `GitCommand` |
| GitHubIssues | `github.issues` | `GitHubCommand` |
| Terminal | `shell_pty` | `ShellCommand` |
| Agent / AiChat | `agent_pty` | `AgentCommand` |
| Preview | `preview` | `PreviewCommand` |
| Search | `search` | `SearchCommand` |

Panel widgets implement a common trait:

```rust
/// Conceptual — lives in kiwi_gui
pub trait DockPanel {
  fn tab_id(&self) -> KiwiTab;
  fn title(&self) -> &str;
  fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext);
}

pub struct PanelContext<'a> {
  pub state: &'a AppState,
  pub command_tx: &'a CommandSender,
  pub theme: &'a GuiTheme,
}
```

Panels send `AppCommand` messages; they do not mutate `AppState` directly.

### Dock state persistence

Extend workspace persistence (ADR-016) with a GUI-specific section:

```json
{
  "version": 2,
  "tui": { "...": "existing TUI fields" },
  "gui": {
    "dock_layout": "<egui_dock serialized tree>",
    "open_tabs": ["Explorer", "Agent", "Terminal"],
    "window": { "width": 1400, "height": 900, "maximized": false }
  }
}
```

- TUI and GUI layouts are independent; same repo hash file may hold both
- Corrupt `gui` section → revert to default layout; keep TUI section intact
- Save on window close and debounced every 30s (same policy as ADR-016)

### Focus model

- **Focused tab:** egui_dock active tab receives keyboard input
- **Global shortcuts:** Command palette (`Ctrl+Shift+P` or `Ctrl+K`), quit, tab switch via View menu
- **PTY focus:** When Terminal or Agent tab is focused, keyboard events route to PTY unless a egui widget (e.g., search box) has focus

TUI focus regions (`left`, `main`, `shell`, `command_palette`) do not map 1:1; GUI uses dock focus + optional command palette overlay (SPEC-013 adapted for GUI).

### Menu bar

Top-level menus (v1):

- **File:** Open repository, Quit
- **View:** Show/hide tabs, reset layout, toggle command palette
- **Git:** Refresh status, open diff tab
- **Help:** Keyboard shortcuts, about

Command palette remains the power-user entry point (ADR-014); GUI renders it as a centered modal (`egui::Window`).

## Consequences

### Positive

- Users customize layout without Kiwi shipping infinite preset layouts
- Clear separation: `KiwiTab` + `DockPanel` vs `kiwi_core` reducers
- Default layout gives sensible first-run experience
- Aligns with VS Code mental model for target users

### Negative

- Dock serialization format tied to egui_dock version; migration logic needed on upgrades
- Two navigation paradigms (TUI tabs vs GUI docks) require separate design docs
- PTY focus routing more complex with dock splits
- Some TUI combinations (GH left + Issues main) become single GitHub panel or split user choice

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Mirror TUI fixed layout in GUI | Wastes desktop affordances; users expect draggable IDE chrome |
| Custom split-pane layout | Reinvents egui_dock; higher maintenance |
| One tab bar only (no dock tree) | Insufficient for Terminal + Agent + Diff simultaneously |
| Floating windows per panel | Focus and persistence complexity; defer to post-v1 |

## Follow-up Work

- SPEC-022: Dock layout engine requirements and acceptance criteria
- [gui-layout.md](../../design/gui-layout.md): visual design and default proportions
- Implement `DockPanel` registry and placeholder panels
- Extend SPEC-017 schema for `gui.dock_layout`
- Map TUI keyboard shortcuts to GUI equivalents in [gui-keyboard-shortcuts.md](../../design/gui-keyboard-shortcuts.md)
