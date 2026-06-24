# Implementation Backlog

Prioritized work items mapped to GitHub issues. Use labels: `milestone-1` … `milestone-7`, `epic-*`, `good-first-issue`.

## Epic Index

| Epic ID | Name | Milestone |
|---------|------|-----------|
| E1 | Project scaffold | M1 |
| E2 | Config & startup | M1 |
| E3 | Theme & layout | M1 |
| E4 | Navigation & state | M1 |
| E5 | Shell PTY | M2 |
| E6 | Agent PTY | M2 |
| E7 | Command palette | M2 |
| E8 | File tree | M3 |
| E9 | Preview & editor | M3 |
| E10 | Search | M3 |
| E11 | Git service | M4 |
| E12 | File watcher | M4 |
| E13 | Diff viewer | M4 |
| E14 | GitHub issues | M5 |
| E15 | GitHub PRs | M5 |
| E16 | Workspace persistence | M6 |
| E17 | Plugins | M7 |
| E18 | Multi-agent | M7 |

---

## Milestone 1: Foundation

### E1: Project scaffold

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #1 | Initialize Cargo binary crate `kiwi` | — |
| #2 | Add core dependencies (ratatui, crossterm, tokio, serde, toml, anyhow) | ADR-002 |
| #3 | CI: fmt, clippy, test on push | — |
| #4 | Add `config.example.toml` and `README` install section | ADR-005 |

### E2: Config & startup

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #5 | Implement CLI argument parsing | SPEC-001 |
| #6 | Implement config loader with merge order | SPEC-018, ADR-005 |
| #7 | Startup lifecycle: terminal init/teardown guard | SPEC-001 |
| #8 | Repository root detection and validation | SPEC-001 |

### E3: Theme & layout

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #9 | Theme engine with semantic roles | SPEC-003, ADR-004 |
| #10 | Bundle kiwi-dark and kiwi-light themes | SPEC-003 |
| #11 | Bundle community themes (dracula, catppuccin, etc.) | SPEC-003 |
| #12 | Layout engine with region rects | SPEC-002, ADR-003 |
| #13 | Render tab bars and pane borders | SPEC-002 |

### E4: Navigation & state

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #14 | Navigation state: dual tabs + focus | SPEC-004 |
| #15 | App state skeleton and event channel | SPEC-016, ADR-007 |
| #16 | Keyboard shortcuts for tabs and focus | design/keyboard-shortcuts |
| #17 | Status bar rendering | SPEC-019 |
| #18 | Basic mouse: tab click | SPEC-014, ADR-015 |

---

## Milestone 2: Terminal Services

### E5: Shell PTY

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #19 | portable-pty shell spawn | SPEC-011, ADR-006 |
| #20 | Shell I/O loop and scrollback buffer | SPEC-011 |
| #21 | Shell focus and input forwarding | SPEC-011 |
| #22 | PTY resize on terminal resize | SPEC-011 |

### E6: Agent PTY

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #23 | Agent lazy spawn on Agent tab | SPEC-010 |
| #24 | Agent I/O and viewport render | SPEC-010 |
| #25 | Agent status heuristics for status bar | SPEC-010, SPEC-019 |
| #26 | Agent restart command | SPEC-010 |

### E7: Command palette

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #27 | Command registry and palette UI | SPEC-013, ADR-014 |
| #28 | Fuzzy filter implementation | SPEC-013 |
| #29 | Initial command set (navigation, quit, focus) | SPEC-013 |

---

## Milestone 3: File Management

### E8: File tree

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #30 | Lazy directory loading | SPEC-005, ADR-008 |
| #31 | Tree widget with expand/collapse | SPEC-005 |
| #32 | Default ignore rules | SPEC-005 |
| #33 | Git status badges on files | SPEC-005, E11 |

### E9: Preview & editor

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #34 | File preview pane with virtualization | SPEC-006 |
| #35 | Editor launcher resolution chain | SPEC-015, ADR-013 |
| #36 | Editor launch from tree, preview, palette | SPEC-015 |
| #37 | Preview reload on file change | SPEC-006 |

### E10: Search

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #38 | File fuzzy search | SPEC-007, ADR-009 |
| #39 | Ripgrep content search subprocess | SPEC-007 |
| #40 | Search UI in left Search tab | SPEC-007 |
| #41 | Search debounce and cancel | SPEC-007 |

---

## Milestone 4: Git Integration

### E11: Git service

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #42 | git2 repository open and branch info | SPEC-008, ADR-010 |
| #43 | File status lists with incremental patch | SPEC-008, ADR-007 |
| #44 | Git left panel UI | SPEC-008 |
| #45 | Manual git refresh command | SPEC-008 |

### E12: File watcher

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #46 | notify watcher on repo root | ADR-011 |
| #47 | Debounce and coalesce events | ADR-011 |
| #48 | Path-targeted cache invalidation | ADR-011 |
| #49 | Scroll/selection preservation tests | ADR-007 |

### E13: Diff viewer

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #50 | git2 unified diff generation | SPEC-012 |
| #51 | Diff viewer widget with colors | SPEC-012 |
| #52 | Staged/unstaged toggle | SPEC-012 |
| #53 | Diff file navigation n/p | SPEC-012 |

---

## Milestone 5: GitHub Integration

### E14: GitHub issues

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #54 | gh auth detection and error UI | SPEC-009 |
| #55 | Issue list via gh json | SPEC-009 |
| #56 | Issue detail view | SPEC-009 |
| #57 | Issue comment and label actions | SPEC-009 |
| #58 | Create branch from issue | SPEC-009 |

### E15: GitHub PRs

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #59 | PR list via gh json | SPEC-009 |
| #60 | PR detail view | SPEC-009 |
| #61 | PR create workflow | SPEC-009 |
| #62 | Open in browser command | SPEC-009 |
| #63 | GH left hub tab | design/navigation |

---

## Milestone 6: Workspace Features

### E16: Workspace persistence

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #64 | Workspace snapshot schema | SPEC-017, ADR-016 |
| #65 | Load on startup | SPEC-017 |
| #66 | Save on quit and periodic | SPEC-017 |
| #67 | Palette history persistence | SPEC-017 |

---

## Milestone 7: Advanced

### E17: Plugins

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #68 | kiwi_plugin_api crate | SPEC-020, ADR-018 |
| #69 | Plugin discovery and load | SPEC-020 |
| #70 | Sample plugin | SPEC-020 |

### E18: Multi-agent

| Issue | Title | Spec/ADR |
|-------|-------|----------|
| #71 | AgentManager data model | ADR-017 |
| #72 | Multiple PTY sessions | ADR-017 |
| #73 | Agent tab sub-tabs UI | ADR-017 |

---

## User Story Backlog (Cross-Cutting)

| Story | Milestone | Issues |
|-------|-----------|--------|
| Open repo and see layout | M1 | #1–#18 |
| Run git in embedded shell | M2 | #19–#22 |
| Talk to AI agent | M2 | #23–#26 |
| Browse and open files | M3 | #30–#36 |
| Search codebase | M3 | #38–#41 |
| See live git status | M4 | #42–#49 |
| Review diffs | M4 | #50–#53 |
| Triage GitHub issues | M5 | #54–#58 |
| Create PR | M5 | #59–#63 |
| Resume session | M6 | #64–#67 |

## Technical Debt Tracker

| Item | Priority | Notes |
|------|----------|-------|
| ANSI parser completeness | Medium | PTY fidelity |
| Integration test harness | High | pexpect-style PTY tests |
| Minimum gh version pin | Medium | Document in README |
| Windows support | Low | PTY + paths; post-MVP |

## Definition of Done

- Acceptance criteria in linked SPEC met
- Unit tests for reducers/services where applicable
- No clippy warnings
- Manual test note in PR description
- Docs updated if public behavior changes
