# Contributing to Kiwi

Thank you for your interest in Kiwi. This project is open source and welcomes
contributions from the community.

Please read this guide before opening a pull request. For AI-assisted
development, also read [AGENTS.md](AGENTS.md).

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). By
participating, you agree to uphold it.

## Ways to Contribute

- **Code** — Features, bug fixes, tests, and documentation aligned with the
  roadmap
- **Documentation** — SPECs, ADRs, design notes, and user-facing docs
- **Issues** — Bug reports, design discussion, and milestone planning
- **Review** — Thoughtful pull request feedback

Check [docs/roadmap/backlog.md](docs/roadmap/backlog.md) and GitHub issues for
prioritized work. Items labeled `good-first-issue` are a good entry point.

## Getting Started

### Prerequisites

- Rust toolchain (2021 edition)
- `cargo`, `rustfmt`, and `clippy`
- Optional: PostgreSQL + Python 3 for MCP memory tooling during development

### Clone and build

```bash
git clone https://github.com/pacificnm/kiwi.git
cd kiwi
cargo build
cargo test --workspace
```

See [BUILD_COMMANDS.md](BUILD_COMMANDS.md) for lint, formatting, and tooling
commands.

## Development Workflow

### 1. Pick work aligned with the roadmap

Kiwi is milestone-driven. Prefer contributions that advance the current
milestone ([docs/roadmap/milestones.md](docs/roadmap/milestones.md)) or fix
documented defects ([KNOWN_ISSUES.md](KNOWN_ISSUES.md)).

Large features should have a SPEC or ADR before substantial implementation.
Discuss in an issue first when scope is unclear.

### 2. Create a branch

```bash
git checkout -b your-name/short-description
```

### 3. Implement with tests

- Follow the relevant [SPEC](docs/specs/) for behavioral requirements
- Cross-check [ADRs](docs/architecture/adr/) for architectural constraints
- Match existing code style and module boundaries
- Add or update tests for behavior you change
- Keep diffs focused; avoid unrelated refactors

### 4. Verify locally

```bash
cargo fmt
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

### 5. Open a pull request

- Link related issues (`Fixes #123` when applicable)
- Describe **what** changed and **why**
- Reference SPEC or ADR numbers when implementing them
- Note any manual testing performed
- Update documentation when behavior or setup changes

## Documentation Standards

| Change type | Update |
| --- | --- |
| New behavior | Relevant SPEC and/or design doc |
| Architectural choice | New or updated ADR |
| Build or tooling | [BUILD_COMMANDS.md](BUILD_COMMANDS.md) |
| Agent workflow | [AGENTS.md](AGENTS.md) |
| Known bugs | [KNOWN_ISSUES.md](KNOWN_ISSUES.md) |
| Security impact | [SECURITY.md](SECURITY.md) and private report if needed |

After documentation changes, re-index project memory when using MCP tools:

```bash
./scripts/index-memory.sh
```

## Rust Conventions

- Workspace-wide `unsafe` is **forbidden** unless an ADR explicitly approves an
  exception
- Use `anyhow` for application errors per project stack guidance
- Prefer event-driven patterns for Git and UI state (see ADRs)
- Run `cargo fmt` before committing

## Commit Messages

Write clear commit messages in the imperative mood:

```text
Add config loader for user and project TOML

Merge user config from ~/.config/kiwi/config.toml with .kiwi.toml in the
repository root per SPEC-018.
```

Focus on **why** the change matters, not only which files changed.

## Pull Request Review

Maintainers may request changes for:

- Scope creep outside the stated issue or milestone
- Missing tests or documentation
- SPEC/ADR divergence
- Clippy warnings or formatting issues

Reviews aim to be constructive and timely. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Reporting Bugs

1. Search existing issues for duplicates
2. Open a new issue with reproduction steps, expected vs actual behavior, and
   environment details (OS, Rust version)
3. For security issues, follow [SECURITY.md](SECURITY.md) — do **not** file
   public issues

## Reporting Security Issues

See [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contributions will be licensed under the
[MIT License](LICENSE.md), the same license that covers this project.

## Questions

- **Architecture and specs** — [docs/README.md](docs/README.md)
- **Product vision** — [plan.md](plan.md)
- **MCP development setup** — [tools/MCP-SETUP.md](tools/MCP-SETUP.md)

Open a GitHub issue for questions that are not security-sensitive.
