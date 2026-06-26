# Build Commands

Common commands for developing Kiwi. Run from the repository root unless noted.

## Rust

```bash
# Build the workspace
cargo build

# Release build
cargo build --release

# Run the application (once a repo path is supported)
cargo run -p kiwi

# Build the desktop GUI (SPEC-021; requires a graphical session to run)
cargo build -p kiwi_gui

# Open GUI for current directory (loads ~/.config/kiwi/config.toml, .kiwi.toml, --theme/--config)
cargo run -p kiwi_gui -- .

# Example with theme override (same merge order as the TUI; SPEC-018)
cargo run -p kiwi_gui -- --theme dracula /path/to/repo

# Run all workspace tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check

# Apply formatting
cargo fmt
```

## Python (MCP memory tools)

```bash
# Create virtual environment (first time)
python3 -m venv .venv
.venv/bin/pip install -r tools/requirements.txt

# Copy and edit environment variables
cp .env.example .env
```

Required `.env` values:

```env
DATABASE_URL="postgresql:///kiwi_memory?host=/var/run/postgresql"
OPENAI_API_KEY="sk-..."
```

## Project memory

```bash
# Index repository documentation into PostgreSQL
./scripts/index-memory.sh

# Command-line semantic search (after indexing)
.venv/bin/python tools/search_memory.py "layout engine"
```

Database and MCP server setup: [tools/MCP-SETUP.md](tools/MCP-SETUP.md).

## Context memory setup

```bash
.venv/bin/python tools/setup_context_memory.py
```

If the database user cannot create tables, run
`sudo -u postgres psql kiwi_memory -f tools/setup_context_memory.sql` once.
