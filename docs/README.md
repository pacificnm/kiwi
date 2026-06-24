# Kiwi Documentation

Kiwi is a Rust-based, terminal-native AI development workspace. It orchestrates AI agents, shells, Git, GitHub, file navigation, search, diff review, and external editors from a single TUI—without replacing the user's editor of choice.

This documentation package expands the [project initiation plan](../plan.md) into actionable artifacts for implementation.

## Document Map

| Area | Purpose | Start Here |
|------|---------|------------|
| [Architecture](./architecture/README.md) | High-level structure and decision records | [ADR index](./architecture/adr/README.md) |
| [Specifications](./specs/README.md) | Detailed behavioral and interface contracts | [SPEC-001 Startup](./specs/SPEC-001-startup-lifecycle.md) |
| [Design](./design/README.md) | UX, layout, interaction, and workflows | [Layout](./design/layout.md) |
| [Roadmap](./roadmap/README.md) | Milestones, backlog, and release planning | [Milestones](./roadmap/milestones.md) |
| [Repository Structure](./repository-structure.md) | Proposed crate and module layout | — |

## Core Principles

1. **Orchestrator first, editor second** — Kiwi coordinates tools; it does not replace editors.
2. **Terminal-native** — Full development workflows stay inside the terminal.
3. **Incremental, flicker-free updates** — Internal state updates preserve scroll, selection, and focus.
4. **Event-driven Git** — File watcher + debounce; no polling for repository status.
5. **Editor-agnostic** — Launch external editors via config, `$VISUAL`, `$EDITOR`, or `nano` fallback.
6. **GitHub via `gh`** — Initial integration uses the GitHub CLI; GraphQL evaluation is deferred.

## Recommended Technology Stack

| Layer | Crates |
|-------|--------|
| TUI | `ratatui`, `crossterm` |
| Async runtime | `tokio` |
| PTY | `portable-pty` |
| File watching | `notify` |
| Serialization | `serde`, `toml` |
| Git | `git2` |
| Errors | `anyhow` |
| Future | `octocrab`, `tree-sitter`, `ignore`, `walkdir` |

## How to Use These Docs

### For implementers

1. Read [ADR-001](./architecture/adr/ADR-001-workspace-philosophy.md) for product philosophy.
2. Follow milestone order in [milestones.md](./roadmap/milestones.md).
3. Implement each component against its SPEC; cross-reference ADRs for rationale.
4. Use [design/](./design/) for UX fidelity and [repository-structure.md](./repository-structure.md) for module boundaries.

### For reviewers

- ADRs capture **why** decisions were made.
- SPECs capture **what** must be built and how to verify it.
- Design docs capture **how it should feel** to use Kiwi.

## MVP Definition

A developer can, without leaving Kiwi:

1. Open a repository
2. Browse files and preview content
3. View GitHub issues
4. Launch an AI agent session
5. Edit files via an external editor
6. Review diffs
7. Create a pull request

See [roadmap/milestones.md](./roadmap/milestones.md) for the milestone breakdown that delivers MVP.

## Document Conventions

- **ADR-NNN** — Architecture Decision Record; status is `Accepted` unless noted.
- **SPEC-NNN** — Technical specification with acceptance criteria.
- Configuration examples use TOML.
- Keyboard shortcuts use `Ctrl+` / `Alt+` notation unless platform-specific.

## Related

- Source plan: [plan.md](../plan.md)
- Issue tracking: GitHub Issues (to be created per [backlog.md](./roadmap/backlog.md))
