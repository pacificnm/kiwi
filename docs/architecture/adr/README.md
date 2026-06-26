# Architecture Decision Records

Architecture Decision Records (ADRs) document significant technical and product decisions for Kiwi. They are immutable once accepted; superseded decisions receive a new ADR that references the prior record.

## Index

| ID | Title | Status |
|----|-------|--------|
| [ADR-001](./ADR-001-workspace-philosophy.md) | Workspace Philosophy | Accepted |
| [ADR-002](./ADR-002-tui-framework-selection.md) | TUI Framework Selection | Accepted |
| [ADR-003](./ADR-003-layout-architecture.md) | Layout Architecture | Accepted |
| [ADR-004](./ADR-004-theme-system.md) | Theme System | Accepted |
| [ADR-005](./ADR-005-configuration-system.md) | Configuration System | Accepted |
| [ADR-006](./ADR-006-pty-architecture.md) | PTY Architecture | Accepted |
| [ADR-007](./ADR-007-state-management.md) | State Management | Accepted |
| [ADR-008](./ADR-008-file-tree-architecture.md) | File Tree Architecture | Accepted |
| [ADR-009](./ADR-009-search-architecture.md) | Search Architecture | Accepted |
| [ADR-010](./ADR-010-git-integration.md) | Git Integration | Accepted |
| [ADR-011](./ADR-011-file-watcher-architecture.md) | File Watcher Architecture | Accepted |
| [ADR-012](./ADR-012-github-integration.md) | GitHub Integration | Accepted |
| [ADR-013](./ADR-013-external-editor-strategy.md) | External Editor Strategy | Accepted |
| [ADR-014](./ADR-014-command-palette-architecture.md) | Command Palette Architecture | Accepted |
| [ADR-015](./ADR-015-mouse-interaction.md) | Mouse Interaction | Accepted |
| [ADR-016](./ADR-016-workspace-persistence.md) | Workspace Persistence | Accepted |
| [ADR-017](./ADR-017-multi-agent-future-design.md) | Multi-Agent Future Design | Accepted |
| [ADR-018](./ADR-018-plugin-architecture.md) | Plugin Architecture | Accepted |
| [ADR-019](./ADR-019-system-clipboard-integration.md) | System Clipboard Integration | Accepted |
| [ADR-020](./ADR-020-dual-frontend-architecture.md) | Dual Frontend Architecture | Accepted |
| [ADR-021](./ADR-021-desktop-gui-framework-selection.md) | Desktop GUI Framework Selection | Accepted |
| [ADR-022](./ADR-022-gui-dock-layout-architecture.md) | GUI Dock Layout Architecture | Accepted |

## ADR Template

Each ADR follows this structure:

- **Title** — Short descriptive name
- **Status** — Proposed | Accepted | Deprecated | Superseded
- **Context** — Forces and constraints
- **Decision** — What we chose
- **Consequences** — Positive and negative outcomes
- **Alternatives Considered** — Options rejected and why
- **Follow-up Work** — Implementation tasks and open items

## Reading Order for New Contributors

1. ADR-001 — Product philosophy
2. ADR-002, ADR-003 — UI foundation
3. ADR-005, ADR-007 — Config and state
4. ADR-006 — PTY for shell and agent
5. Domain ADRs (008–016, 019) as needed for your milestone
6. ADR-020–022 — Desktop GUI (post-MVP)
