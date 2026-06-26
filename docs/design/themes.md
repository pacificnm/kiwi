# Theme and Color Design

## Visual Identity

Kiwi should feel **modern, calm, and information-dense**—closer to a developer tool than a flashy demo TUI. Chrome is subdued; semantic colors carry meaning.

## Built-In Themes

| Theme ID | Mode | Notes |
|----------|------|-------|
| `kiwi-dark` | Dark | Default; balanced contrast |
| `kiwi-light` | Light | For bright environments |
| `dracula` | Dark | Community favorite |
| `catppuccin-mocha` | Dark | Soft pastel dark |
| `catppuccin-latte` | Light | Soft pastel light |
| `gruvbox` | Dark | Warm retro |
| `nord` | Dark | Arctic blue-gray |
| `tokyo-night` | Dark | Blue-purple night |

Each theme defines full semantic role set. Custom themes may extend `kiwi-dark` via `extends = "kiwi-dark"`.

## Semantic Color Strategy

Colors are **semantic**, not per-widget. Widgets request roles; theme resolves to `ratatui::Style`.

### UI Chrome

| Role | Usage |
|------|-------|
| `bg` / `fg` | Default background and text |
| `border` | Pane borders, dividers |
| `accent` | Focus rings, active tab, primary actions |
| `muted` | Secondary text, line numbers, hints |
| `selection` | Selected list/tree row background |

### Git Colors

| Status | Color | Role |
|--------|-------|------|
| Added | Green | `git_added` |
| Modified | Yellow | `git_modified` |
| Deleted | Red | `git_deleted` |
| Untracked | Blue | `git_untracked` |

### GitHub Issues

| State | Color | Role |
|-------|-------|------|
| Open | Cyan | `issue_open` |
| In Progress | Yellow | `issue_in_progress` |
| Closed | Gray | `issue_closed` |

### Pull Requests

| State | Color | Role |
|-------|-------|------|
| Open | Blue | `pr_open` |
| Draft | Yellow | `pr_draft` |
| Merged | Green | `pr_merged` |
| Closed | Gray | `pr_closed` |

### Agent States

| State | Color | Role |
|-------|-------|------|
| Thinking | Purple | `agent_thinking` |
| Executing | Blue | `agent_executing` |
| Success | Green | `agent_success` |
| Error | Red | `agent_error` |
| Warning | Yellow | `agent_warning` |

### File Tree

| Category | Role | Typical extensions / names |
|----------|------|------------------------------|
| Directory | `file_dir` | folders |
| Source code | `file_source` | `.rs`, `.go`, `.c`, `.cpp`, `.java`, … |
| Scripts | `file_script` | `.py`, `.sh`, `.js`, `.ts`, … |
| Markup / docs | `file_markup` | `.md`, `.rst`, `README`, `LICENSE`, … |
| Config | `file_config` | `.toml`, `.yaml`, `.json`, `Dockerfile`, … |
| Data | `file_data` | `.sql`, `.csv`, `.xml`, `.lock`, … |
| Media | `file_media` | `.png`, `.svg`, `.pdf`, fonts, … |
| Other | `file_other` | unmatched files |

Git status colors override file-type colors when both apply.

## Typography

- Use terminal default font; no font switching in v1
- Bold for active tabs and headers
- Italic for comments/metadata (optional per theme)

## PTY Rendering

Shell and agent output use **ANSI colors from child process**; Kiwi does not remap them to theme (preserves tool fidelity). Chrome around PTY still uses theme.

## Configuration

```toml
[theme]
name = "kiwi-dark"
```

Custom:

```toml
[theme]
custom = "~/.config/kiwi/themes/my.toml"
```

Example custom fragment:

```toml
name = "my-kiwi"
extends = "kiwi-dark"

[colors]
accent = "#7aa2f7"
git_modified = "#e0af68"
```

## Accessibility

- Do not rely on color alone: git status shows single-letter badge (`M`, `A`, `D`, `?`)
- PR/issue states include text labels
- Minimum contrast target WCAG AA for chrome text on bg

## Related

- ADR-004 Theme System
- SPEC-003 Theme Engine
