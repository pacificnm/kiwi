# Kiwi

Terminal-native AI development workspace — orchestrate shells, Git, GitHub,
files, search, diffs, external editors, and AI agents from a single TUI.

Kiwi is **not** a text editor. It is a development workbench that coordinates
the tools you already use while keeping workflows inside the terminal.

## Status

Early development. **Milestone 1 (Foundation)** is in progress: Cargo workspace,
TUI skeleton, configuration, theming, layout, and navigation.

## Quick Start

### Install

**Prerequisites:** Rust (2021 edition) with `cargo`, `rustfmt`, and `clippy`; Git for
repository workflows.

Optional as features land: [GitHub CLI](https://cli.github.com/) (`gh`) for GitHub
integration; [ripgrep](https://github.com/BurntSushi/ripgrep) (`rg`) for content search.

Clone and build from source:

```bash
git clone https://github.com/pacificnm/kiwi.git
cd kiwi
./scripts/build.sh
```

Release build: `./scripts/build.sh --release`

See [BUILD_COMMANDS.md](BUILD_COMMANDS.md) for lint, test, and tooling commands.

### Configuration

Kiwi uses TOML configuration. Precedence (highest first): CLI flags → `.kiwi.toml`
in the repository root → `~/.config/kiwi/config.toml` → built-in defaults.

```bash
mkdir -p ~/.config/kiwi
cp config.example.toml ~/.config/kiwi/config.toml
```

Edit `command` values for your editor, shell, and agent. For team or project
defaults, commit `.kiwi.toml` in the repository root.

Reference: [ADR-005](docs/architecture/adr/ADR-005-configuration-system.md),
[SPEC-018](docs/specs/SPEC-018-configuration-loader.md), and
[config.example.toml](config.example.toml).

### Run

```bash
./scripts/launch.sh
```

Or with Cargo:

```bash
cargo run -p kiwi
```

Application behavior will expand as milestones land. See
[docs/roadmap/milestones.md](docs/roadmap/milestones.md).

## Documentation

| Document | Purpose |
| --- | --- |
| [docs/README.md](docs/README.md) | Documentation index — start here for architecture, specs, and design |
| [plan.md](plan.md) | Project initiation document — vision, workflows, and scope |
| [AGENTS.md](AGENTS.md) | Instructions for AI coding agents |
| [docs/repository-structure.md](docs/repository-structure.md) | Proposed crate and module layout |
| [docs/specs/](docs/specs/) | Behavioral specifications (SPEC-001 …) |
| [docs/architecture/adr/](docs/architecture/adr/) | Architecture decision records |
| [docs/roadmap/milestones.md](docs/roadmap/milestones.md) | Milestone plan and MVP definition |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute code and documentation |
| [SECURITY.md](SECURITY.md) | Vulnerability reporting |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Community standards |

### For implementers

1. Read [docs/architecture/adr/ADR-001-workspace-philosophy.md](docs/architecture/adr/ADR-001-workspace-philosophy.md)
2. Follow milestone order in [docs/roadmap/milestones.md](docs/roadmap/milestones.md)
3. Implement against the relevant SPEC in [docs/specs/](docs/specs/)

### For AI agents

Read [AGENTS.md](AGENTS.md) for documentation precedence, MCP memory usage, and
implementation rules.

## Agent Memory (optional)

Kiwi includes MCP servers for semantic search over project docs and for
persisting agent session context. This is a development-time aid for Cursor and
other MCP clients.

```bash
python3 -m venv .venv
.venv/bin/pip install -r tools/requirements.txt
cp .env.example .env   # set DATABASE_URL and OPENAI_API_KEY
./scripts/index-memory.sh
```

Full setup: [tools/MCP-SETUP.md](tools/MCP-SETUP.md).

## Repository Layout

```text
kiwi/
├── crates/kiwi/          # Main application binary
├── config.example.toml   # Example user/project configuration
├── docs/                 # Architecture, specs, design, roadmap
├── scripts/              # Development scripts (e.g. index-memory.sh)
├── tools/                # MCP memory servers and Python helpers
├── plan.md               # Project initiation document
├── AGENTS.md             # Agent instructions
├── BUILD_COMMANDS.md     # Build and tooling commands
├── CONTRIBUTING.md       # Contribution guide
├── CODE_OF_CONDUCT.md    # Community standards
├── SECURITY.md           # Vulnerability reporting
└── LICENSE.md            # MIT license
```

## Plugins

Kiwi supports native Rust plugins that register commands in the palette. Plugins are
`.so` / `.dylib` files loaded from `~/.config/kiwi/plugins/`.

```bash
# Install a plugin (copies files and registers it)
kiwi plugin install /path/to/my-plugin

# Manage plugins without starting the TUI
kiwi plugin list
kiwi plugin enable  <name>
kiwi plugin disable <name>
kiwi plugin remove  <name>
```

Enable/disable changes take effect on the next restart. The **Plugin Manager** shows
status for all installed plugins:

- **TUI**: press `9` or search the command palette for **"Plugins: Open Manager"**
- **GUI**: **File → Plugins** menu

See [`crates/kiwi_plugin_api/README.md`](crates/kiwi_plugin_api/README.md) for the
full authoring guide and `plugin.toml` reference. A working sample plugin is at
[`plugins/kiwi_plugin_hello/`](plugins/kiwi_plugin_hello/).

> **Security:** plugins run in-process with full user privileges. Only install plugins
> from sources you trust.

## Principles

1. **Orchestrator first, editor second**
2. **Terminal-native workflows**
3. **Incremental, flicker-free UI updates**
4. **Event-driven Git** (watcher + debounce)
5. **Editor-agnostic** external editor launch
6. **GitHub via `gh` CLI**

## License

Kiwi is open source software licensed under the [MIT License](LICENSE.md).

Copyright (c) Pacific NM.

## Community

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before
opening a pull request. Report security issues per [SECURITY.md](SECURITY.md).
