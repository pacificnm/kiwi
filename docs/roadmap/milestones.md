# Milestones

Seven milestones from project initiation through advanced features. MVP = Milestones 1–5.

## Overview

| # | Name | Goal | Depends On |
|---|------|------|------------|
| M1 | Foundation | Runnable TUI skeleton with config, theme, layout | — |
| M2 | Terminal Services | Shell, agent PTY, command palette | M1 |
| M3 | File Management | Tree, preview, editor, search | M1, M2 |
| M4 | Git Integration | Status, watcher, diff | M1, M3 |
| M5 | GitHub Integration | Issues, PRs, branch workflows | M1, M4 |
| M6 | Workspace Features | Persistence, sessions, theme packs | M1, M3 |
| M7 | Advanced Features | Multi-agent, plugins, performance | MVP |

---

## Milestone 1: Foundation

**Goal:** Kiwi boots, loads config, renders themed layout with navigable empty panes.

### Epics

1. **Project scaffold** — Cargo workspace, CI, lint
2. **Configuration** — SPEC-018
3. **Theme system** — SPEC-003, bundled themes
4. **Layout engine** — SPEC-002
5. **Navigation shell** — SPEC-004, tab bars, focus
6. **State management** — SPEC-016 skeleton
7. **Status bar** — SPEC-019 static segments

### User Stories

- As a developer, I run `kiwi .` and see the workspace layout with correct tabs.
- As a developer, I configure theme and left width in `config.toml`.
- As a developer, I switch tabs and focus panes with keyboard.
- As a developer, I quit cleanly and terminal is restored.

### Deliverables

- Binary `kiwi` with main loop
- Empty pane placeholders
- Mouse click tab switching (SPEC-014 basic)

### Acceptance

All SPEC-001, SPEC-002, SPEC-003, SPEC-004, SPEC-016, SPEC-018, SPEC-019 acceptance criteria for static behavior.

---

## Milestone 2: Terminal Services

**Goal:** Embedded shell and agent PTY; command palette executes app commands.

### Epics

1. **Shell PTY** — SPEC-011
2. **Agent PTY** — SPEC-010
3. **Command palette** — SPEC-013
4. **PTY rendering** — ANSI viewport in layout

### User Stories

- As a developer, I run shell commands in the bottom pane without leaving Kiwi.
- As a developer, I interact with Cursor Agent in the Agent tab.
- As a developer, I invoke commands via `Ctrl+P`.

### Deliverables

- Working bash/zsh shell
- Agent spawn on Agent tab
- ~20 palette commands

### Acceptance

SPEC-010, SPEC-011, SPEC-013 criteria met.

---

## Milestone 3: File Management

**Goal:** Navigate repository, preview files, search, open external editor.

### Epics

1. **File explorer** — SPEC-005
2. **File preview** — SPEC-006
3. **Editor launcher** — SPEC-015
4. **Search** — SPEC-007

### User Stories

- As a developer, I browse a lazy-loaded file tree with ignores.
- As a developer, I preview files read-only in the Preview tab.
- As a developer, I open files in my configured editor with `e`.
- As a developer, I fuzzy-find files and search content with ripgrep.

### Deliverables

- Full Files left tab
- Search tab with file + content modes
- Editor resolution chain

### Acceptance

SPEC-005, SPEC-006, SPEC-007, SPEC-015 criteria met.

---

## Milestone 4: Git Integration

**Goal:** Live git status via watcher; diff review without polling.

### Epics

1. **Git service** — SPEC-008
2. **File watcher** — ADR-011 integration
3. **Diff viewer** — SPEC-012
4. **Git panels** — Left Git and Diff tabs

### User Stories

- As a developer, I see branch and modified files update when I save.
- As a developer, I review unified diffs with semantic colors.
- As a developer, my list selection does not jump on refresh.

### Deliverables

- git2-backed status
- notify debounce pipeline
- Main and left diff navigation

### Acceptance

SPEC-008, SPEC-012 criteria; no polling in codebase.

---

## Milestone 5: GitHub Integration

**Goal:** Issues and PRs inside Kiwi via `gh`.

### Epics

1. **GitHub service** — SPEC-009
2. **Issues UI** — list + detail
3. **PRs UI** — list + detail + create
4. **Workflow commands** — branch from issue, open browser

### User Stories

- As a developer, I list and view GitHub issues for the repo.
- As a developer, I comment and create a branch from an issue.
- As a developer, I create a PR after committing.
- As a developer, I see auth errors with setup guidance.

### Deliverables

- Issues and PRs main tabs
- GH left hub tab
- Palette GitHub commands

### Acceptance

SPEC-009 criteria; issue-driven workflow in [workflows.md](../design/workflows.md) achievable.

**MVP complete at end of M5.**

---

## Milestone 6: Workspace Features

**Goal:** Session continuity and theme distribution.

### Epics

1. **Workspace persistence** — SPEC-017
2. **Saved sessions** — restore tabs, expansion, scroll
3. **Theme packs** — document custom theme sharing

### User Stories

- As a developer, I reopen Kiwi and resume where I left off.
- As a developer, I share a custom theme TOML with my team.

### Deliverables

- XDG state persistence
- Example theme pack in `assets/themes/`

### Acceptance

SPEC-017 criteria.

---

## Milestone 7: Advanced Features

**Goal:** Multi-agent, plugins, performance hardening.

### Epics

1. **Multi-agent** — ADR-017 implementation
2. **Plugin framework** — SPEC-020
3. **Performance** — diff virtualization, search optimization
4. **Symbol search** — tree-sitter Phase 2

### User Stories

- As a developer, I run two agents on different issues.
- As a developer, I install a plugin that adds palette commands.
- As a developer, I search symbols in Rust/TypeScript files.

### Deliverables

- `kiwi_plugin_api` crate
- AgentManager with multiple PTYs
- Profiling report and optimizations

### Acceptance

SPEC-020 Phase 1; multi-agent manual test checklist.

---

## MVP Definition

**Minimum Viable Product** = Milestones 1–5.

A developer can:

1. Open a repository
2. Browse files and preview
3. View GitHub issues
4. Launch an AI agent
5. Edit via external editor
6. Review diffs
7. Create a pull request

All without leaving Kiwi.

## Future Enhancements (Post-M7)

- Config hot reload
- `octocrab` native GitHub API
- Hunk stage/unstage UI
- LSP preview / syntax highlighting
- jj (Jujutsu) support via plugin
- Team sync for workspace state
