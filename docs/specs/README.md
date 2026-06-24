# Technical Specifications

Specifications define behavioral contracts for Kiwi components. Each SPEC is implementable and testable independently where possible, with explicit acceptance criteria.

## Index

| ID | Title | Milestone | Related ADRs |
|----|-------|-----------|--------------|
| [SPEC-001](./SPEC-001-startup-lifecycle.md) | Startup Lifecycle | M1 | ADR-005, ADR-002 |
| [SPEC-002](./SPEC-002-layout-engine.md) | Layout Engine | M1 | ADR-003 |
| [SPEC-003](./SPEC-003-theme-engine.md) | Theme Engine | M1 | ADR-004 |
| [SPEC-004](./SPEC-004-navigation-system.md) | Navigation System | M1 | ADR-003, ADR-007 |
| [SPEC-005](./SPEC-005-file-explorer.md) | File Explorer | M3 | ADR-008 |
| [SPEC-006](./SPEC-006-file-preview.md) | File Preview | M3 | ADR-001 |
| [SPEC-007](./SPEC-007-search-system.md) | Search System | M3 | ADR-009 |
| [SPEC-008](./SPEC-008-git-service.md) | Git Service | M4 | ADR-010, ADR-011 |
| [SPEC-009](./SPEC-009-github-service.md) | GitHub Service | M5 | ADR-012 |
| [SPEC-010](./SPEC-010-agent-service.md) | Agent Service | M2 | ADR-006, ADR-017 |
| [SPEC-011](./SPEC-011-shell-service.md) | Shell Service | M2 | ADR-006 |
| [SPEC-012](./SPEC-012-diff-viewer.md) | Diff Viewer | M4 | ADR-010 |
| [SPEC-013](./SPEC-013-command-palette.md) | Command Palette | M2 | ADR-014 |
| [SPEC-014](./SPEC-014-mouse-support.md) | Mouse Support | M1–M3 | ADR-015 |
| [SPEC-015](./SPEC-015-editor-launcher.md) | Editor Launcher | M3 | ADR-013 |
| [SPEC-016](./SPEC-016-state-management.md) | State Management | M1 | ADR-007 |
| [SPEC-017](./SPEC-017-workspace-persistence.md) | Workspace Persistence | M6 | ADR-016 |
| [SPEC-018](./SPEC-018-configuration-loader.md) | Configuration Loader | M1 | ADR-005 |
| [SPEC-019](./SPEC-019-status-bar.md) | Status Bar | M1 | ADR-003 |
| [SPEC-020](./SPEC-020-plugin-framework.md) | Plugin Framework | M7 | ADR-018 |

## Specification Template

Each document includes:

- Purpose
- Scope
- Functional Requirements
- Non-Functional Requirements
- Data Structures
- Events / Commands (where applicable)
- Configuration Options (where applicable)
- Error Handling
- Acceptance Criteria

## Dependency Graph (simplified)

```text
SPEC-018 Config → SPEC-001 Startup → SPEC-016 State
                              ↓
         SPEC-002 Layout + SPEC-003 Theme + SPEC-004 Navigation
                              ↓
              SPEC-011 Shell, SPEC-010 Agent, SPEC-013 Palette
                              ↓
         SPEC-005 Files, SPEC-006 Preview, SPEC-007 Search, SPEC-015 Editor
                              ↓
                    SPEC-008 Git, SPEC-012 Diff
                              ↓
                       SPEC-009 GitHub
                              ↓
                  SPEC-017 Persistence, SPEC-020 Plugins
```

## Verification

Acceptance criteria map to integration tests and manual test checklists in [../roadmap/backlog.md](../roadmap/backlog.md).
