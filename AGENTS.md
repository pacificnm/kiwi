# Agent Instructions

Guidance for AI agents and coding assistants working in the Kiwi repository.

## What Kiwi Is

Kiwi is a Rust-based, terminal-native AI development workspace. It orchestrates
shells, Git, GitHub, file navigation, search, diff review, external editors, and
AI agents from a single TUI. Kiwi coordinates tools; it does not replace editors.

Start with [README.md](README.md) for the project overview and [plan.md](plan.md)
for product vision and workflows.

## Documentation Precedence

When implementation guidance conflicts, follow this order:

1. **Specifications** in [docs/specs/](docs/specs/) — binding behavioral contracts
2. **ADRs** in [docs/architecture/adr/](docs/architecture/adr/) — accepted
   architecture decisions and rationale
3. **Design docs** in [docs/design/](docs/design/) — UX, layout, and interaction
4. **MCP project memory** — semantic search over indexed repository docs
5. **Other docs** — [docs/README.md](docs/README.md), [plan.md](plan.md),
   [BUILD_COMMANDS.md](BUILD_COMMANDS.md), [KNOWN_ISSUES.md](KNOWN_ISSUES.md),
   [CONTRIBUTING.md](CONTRIBUTING.md)
6. **Existing code** — match current conventions when docs are silent

SPECs define *what* to build. ADRs explain *why*. Design docs describe *how it
should feel*. MCP memory is a retrieval aid, not a source of truth.

## Project Memory (MCP)

Before changing code, search project memory for the affected subsystem, milestone,
ADR numbers, and known issues.

Useful queries:

- `current milestone`
- subsystem names such as `layout engine`, `git service`, `command palette`
- `ADR-003 theme` or other ADR numbers
- `SPEC-002 layout` or other SPEC numbers
- build, test, or MCP setup instructions

Re-index docs after substantive documentation changes:

```bash
./scripts/index-memory.sh
```

Setup details: [tools/MCP-SETUP.md](tools/MCP-SETUP.md).

### Context memory

Use the `kiwi-context-memory` MCP server to save and search session notes across
Cursor context compaction. Prefer it for work-in-progress state, not permanent
project documentation.

## Current Focus

**Active milestone:** M1 — Foundation ([docs/roadmap/milestones.md](docs/roadmap/milestones.md))

M1 delivers a runnable TUI skeleton with configuration, theming, layout,
navigation, state management, and a status bar. See
[docs/roadmap/backlog.md](docs/roadmap/backlog.md) for prioritized work items.

**Repository layout:** [docs/repository-structure.md](docs/repository-structure.md)

The workspace currently contains the `kiwi` binary crate under `crates/kiwi/`.
Additional crates (`kiwi_core`, `kiwi_tui`, `kiwi_plugin_api`) are planned per
the repository structure doc.

## Implementation Rules

1. **Match existing patterns** — Read surrounding code before editing. Keep diffs
   focused; do not refactor unrelated code.
2. **Follow SPECs** — Implement against acceptance criteria in the relevant SPEC.
   Cross-reference ADRs for architectural constraints.
3. **Rust workspace** — `unsafe` is forbidden workspace-wide. Run `cargo fmt`,
   `cargo clippy`, and `cargo test` before finishing work.
4. **Incremental UI** — Preserve scroll, selection, and focus across state
   updates; avoid full-screen redraw flicker (see ADR-007 and related SPECs).
5. **Editor-agnostic** — Kiwi launches external editors; do not embed editor
   logic in core services.
6. **GitHub via `gh`** — Use the GitHub CLI for initial integration; do not add
   direct GraphQL clients unless an ADR approves it.

## Build and Verification

See [BUILD_COMMANDS.md](BUILD_COMMANDS.md) for commands. Typical workflow:

```bash
cargo build
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## Where to Look

| Topic | Location |
| --- | --- |
| Doc index | [docs/README.md](docs/README.md) |
| Specs | [docs/specs/README.md](docs/specs/README.md) |
| ADRs | [docs/architecture/adr/README.md](docs/architecture/adr/README.md) |
| Milestones | [docs/roadmap/milestones.md](docs/roadmap/milestones.md) |
| Backlog | [docs/roadmap/backlog.md](docs/roadmap/backlog.md) |
| Crate layout | [docs/repository-structure.md](docs/repository-structure.md) |
| MCP setup | [tools/MCP-SETUP.md](tools/MCP-SETUP.md) |
| Known issues | [KNOWN_ISSUES.md](KNOWN_ISSUES.md) |

## Commit and PR Conventions

Follow [CONTRIBUTING.md](CONTRIBUTING.md). In summary:

- Write commit messages that explain *why*, not just *what*.
- Do not commit secrets (`.env`, API keys).
- Reference SPEC or ADR numbers in PR descriptions when implementing them.
- Keep changes scoped to the requested task.
