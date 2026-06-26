# SPEC-022: GUI Dock Layout Engine

## Purpose

Define the egui_dock–based layout system for `kiwi_gui`: tab types, default tree, user rearrangement, and persistence.

## Scope

### In scope

- `KiwiTab` enumeration and panel registry
- egui_dock integration and default layout
- Tab show/hide via View menu
- Dock layout serialization with workspace persistence
- Status bar below dock (shared semantics with SPEC-019)

### Out of scope

- Panel-internal widgets (file tree rows, diff hunks, etc.) — reuse domain specs SPEC-005–012
- TUI layout engine (SPEC-002)
- Floating undocked windows

## Related Documents

- ADR-022 GUI Dock Layout Architecture
- ADR-021 Desktop GUI Framework Selection
- SPEC-017 Workspace Persistence
- SPEC-019 Status Bar

## Functional Requirements

1. **Dock manager** wraps `egui_dock::DockState<KiwiTab>` (or equivalent typed tab key).
2. **Default tabs** on first run (no persisted GUI layout):

   | Region | Tabs (top to bottom / left to right) |
   |--------|--------------------------------------|
   | Left | Explorer, Git Status, GitHub Issues |
   | Center | Agent |
   | Bottom | Terminal |

3. **Tab metadata:**

   | KiwiTab | Title | Closable |
   |---------|-------|----------|
   | Explorer | Explorer | Yes |
   | GitStatus | Git Status | Yes |
   | GitDiff | Diff | Yes |
   | GitLog | Git Log | Yes |
   | GitHubIssues | Issues | Yes |
   | GitHubPrs | Pull Requests | Yes |
   | Preview | Preview | Yes |
   | Search | Search | Yes |
   | Terminal | Terminal | Yes |
   | Agent | Agent | Yes |
   | Config | Settings | Yes |
   | Logs | Logs | Yes |

4. **View menu** lists all tab types; checked items reflect open tabs. Toggle opens tab in last known region or default region.
5. **Reset layout** command restores default tree after confirmation.
6. **Drag-and-drop:** Users reorder tabs and split panes per egui_dock behavior; no custom drag logic in v1.
7. **Render order per frame:**
   1. Top menu bar (`egui::TopBottomPanel::top`)
   2. Dock area (`egui::CentralPanel` or full rect minus menu/status)
   3. Bottom status bar (`egui::TopBottomPanel::bottom`, 24px min height)
8. **Persistence:** Serialize dock tree to `gui.dock_layout` in workspace JSON (SPEC-017). Deserialize on startup; on failure use default layout.
9. **Active tab** receives priority for PTY keyboard routing when tab is `Terminal` or `Agent`.
10. **Placeholder panels** render tab title and "Loading…" or empty state until domain state is ready.

## Non-Functional Requirements

- Dock render overhead < 2ms typical (excluding panel content)
- Layout restore identical across restart for same persistence file
- Tab switch latency imperceptible (< 16ms)

## Data Structures

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KiwiTab { /* see ADR-022 */ }

pub struct DockLayoutState {
  pub dock: DockState<KiwiTab>,
  pub open_tabs: HashSet<KiwiTab>,
}

pub struct PanelRegistry {
  panels: HashMap<KiwiTab, Box<dyn DockPanel>>,
}

pub trait DockPanel {
  fn tab(&self) -> KiwiTab;
  fn show(&mut self, ui: &mut egui::Ui, ctx: &PanelContext<'_>);
}
```

## Events / Commands

| Command | Action |
|---------|--------|
| `AppCommand::GuiShowTab(KiwiTab)` | Open or focus tab |
| `AppCommand::GuiCloseTab(KiwiTab)` | Close tab in dock |
| `AppCommand::GuiResetLayout` | Restore default tree |
| `AppEvent::WorkspaceLoaded` | Apply persisted dock |

## Configuration Options

```toml
[gui.dock]
default_center_tab = "Agent"
show_menu_bar = true
tab_bar_height = 28.0
```

## Error Handling

- Unknown tab in persistence → skip tab, log warning
- Empty dock after close → show empty state with "View → Explorer" hint
- egui_dock version mismatch → reset layout, notify user once per session

## Acceptance Criteria

- [ ] Default layout matches ADR-022 diagram on first run
- [ ] User can drag Explorer to bottom and layout persists after restart
- [ ] View menu toggles Git Status tab
- [ ] Reset layout restores factory default
- [ ] Status bar visible and not obscured by dock
- [ ] Closing Terminal tab and reopening restores PTY session if still alive (SPEC-011)
- [ ] All `KiwiTab` variants have a panel implementation (placeholder minimum)

## Panel Implementation Order

| Order | Panel | Depends on |
|-------|-------|------------|
| 1 | Logs | — |
| 2 | Explorer | SPEC-005 |
| 3 | Git Status | SPEC-008 |
| 4 | Preview | SPEC-006 |
| 5 | Diff | SPEC-012 |
| 6 | Terminal | SPEC-011 |
| 7 | Agent | SPEC-010 |
| 8 | GitHub Issues / PRs | SPEC-009 |
| 9 | Search | SPEC-007 |
| 10 | Config | SPEC-018 |
