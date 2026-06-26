# SPEC-023: GUI Theme Bridge

## Purpose

Map Kiwi semantic theme roles (ADR-004, SPEC-003) to egui `Visuals` and panel styling so the GUI feels cohesive with the TUI and supports the same bundled/user themes.

## Scope

### In scope

- `ResolvedTheme` → `egui::Visuals` conversion
- Semantic role usage in GUI panels (Git colors, issue states, agent status)
- Light/dark mode selection from theme name
- Font scale from config

### Out of scope

- ANSI truecolor terminal resolution (TUI only)
- PTY child process theming
- Custom egui widget skins beyond `Visuals` and `Style`

## Related Documents

- ADR-004 Theme System
- SPEC-003 Theme Engine
- ADR-021 Desktop GUI Framework Selection

## Functional Requirements

1. **`GuiTheme` struct** holds:
   - `egui::Visuals` (widgets, panels, selection)
   - `SemanticColors` map (role name → `egui::Color32`) for domain panels
   - `base_font_size: f32`
2. **Conversion** from `ResolvedTheme` at startup and on theme change command:
   - `bg` → `visuals.panel_fill`, `window_fill`
   - `fg` → `visuals.text_color()`
   - `border` → window stroke
   - `accent` → `visuals.selection.bg_fill`, hyperlinks
   - `muted` → weak text, disabled widgets
   - `selection` → selected list rows
   - Git roles → explorer labels, diff gutter, status badges
   - Issue/PR roles → GitHub panel labels
   - Agent roles → agent status indicator, log prefixes
3. **Dark/light detection:** Theme name suffix or `ResolvedTheme.is_dark` flag selects `Visuals::dark()` or `light()` as base, then overrides from roles.
4. **Apply each frame:** `ctx.set_visuals(gui_theme.visuals.clone())` in `KiwiApp::update` (or on change only).
5. **User themes:** TOML files in `~/.config/kiwi/themes/` work identically to TUI; missing roles inherit from bundled base theme (same as SPEC-003).
6. **Config:**

   ```toml
   [gui.font]
   size = 14.0

   [theme]
   name = "catppuccin-mocha"   # shared with TUI
   ```

7. **High-DPI:** Respect eframe pixel scaling; font size multiplied by `ctx.pixels_per_point()`.

## Non-Functional Requirements

- Theme build < 1ms
- Colors meet WCAG AA contrast for primary text on `bg` where feasible
- No per-widget hard-coded hex in panel code; use `GuiTheme` accessors

## Data Structures

```rust
pub struct GuiTheme {
  pub visuals: egui::Visuals,
  pub semantic: SemanticColorMap,
  pub font_size: f32,
}

pub struct SemanticColorMap {
  pub git_added: Color32,
  pub git_modified: Color32,
  // ... all roles from ADR-004
}

impl GuiTheme {
  pub fn from_resolved(theme: &ResolvedTheme, gui_config: &GuiConfig) -> Self;
  pub fn role(&self, role: ThemeRole) -> Color32;
}
```

## Events / Commands

| Event / Command | Action |
|-----------------|--------|
| Startup | Build `GuiTheme` from resolved config |
| `AppCommand::SetTheme(name)` | Reload theme, rebuild `GuiTheme` |
| `AppEvent::ThemeLoaded` | Same |

## Error Handling

- Missing role in custom theme → inherit from base; log debug once
- Invalid theme name → fallback `kiwi-dark`; status bar warning

## Acceptance Criteria

- [ ] `kiwi-dark` and `kiwi-light` render readable panels
- [ ] Git modified file uses `git_modified` color in Explorer
- [ ] Issue open/closed badges use issue roles in GitHub panel
- [ ] Changing `theme.name` in config and restarting GUI applies new palette
- [ ] No `Color32::from_rgb` literals in panel code except in theme bridge module
- [ ] Font size config changes base UI scale

## Testing

- Unit tests: role → `Color32` for each bundled theme TOML fixture
- Snapshot optional: render placeholder panel to texture (manual QA checklist)
