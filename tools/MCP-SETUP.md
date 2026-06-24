# Kiwi MCP Setup

This guide covers local setup for the Kiwi MCP servers and wiring them into
Cursor. The servers are development-time helpers only.

Two MCP servers are provided:

| Server | Script | Purpose |
| --- | --- | --- |
| `kiwi-memory` | `tools/mcp_memory_server.py` | Semantic search over indexed project docs, ADRs, and build notes. |
| `kiwi-context-memory` | `tools/mcp_context_memory_server.py` | Save, search, and retrieve agent session context across Cursor compaction. |

Both servers use the same PostgreSQL database, Python virtual environment, and
OpenAI embedding key.

## Quick Start Checklist

From the repository root:

1. Install PostgreSQL and `pgvector`.
2. Create the `kiwi_memory` database and `project_memory` table.
3. Create `.venv`, install Python dependencies, and create `.env`.
4. Index project docs: `./scripts/index-memory.sh`
5. Create the context table: run `tools/setup_context_memory.sql` as `postgres`.
6. Confirm `.cursor/mcp.json` paths match your machine.
7. Reload Cursor and confirm both servers are connected.
8. Confirm `.cursor/hooks.json` is loaded (Hooks tab in Cursor settings).

## Prerequisites

- PostgreSQL with the `pgvector` extension
- Python 3
- An OpenAI API key
- Cursor with MCP support

Default database URL:

```text
postgresql:///kiwi_memory?host=/var/run/postgresql
```

Override with `DATABASE_URL` in `.env` when using TCP, a remote host, or different
credentials.

## 1. PostgreSQL Setup

On Debian-family systems:

```bash
sudo apt install postgresql postgresql-contrib postgresql-XX-pgvector
```

Replace `XX` with the installed PostgreSQL major version.

Create the database:

```bash
sudo -u postgres createdb kiwi_memory
```

If `sudo -u postgres` is unavailable, switch to the `postgres` system user and
run the same commands from the repository checkout.

Enable `pgvector` and create the project-memory table:

```bash
sudo -u postgres psql kiwi_memory
```

```sql
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS project_memory (
    id bigserial PRIMARY KEY,
    source_path text NOT NULL,
    content text NOT NULL,
    content_hash text NOT NULL UNIQUE,
    embedding vector(1536) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS project_memory_embedding_idx
    ON project_memory
    USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS project_memory_source_path_idx
    ON project_memory (source_path);
```

Grant access to the OS user that will run the MCP servers:

```bash
sudo -u postgres createuser "$USER"
sudo -u postgres psql -d kiwi_memory -c "GRANT ALL PRIVILEGES ON DATABASE kiwi_memory TO \"$USER\";"
sudo -u postgres psql -d kiwi_memory -c "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO \"$USER\";"
sudo -u postgres psql -d kiwi_memory -c "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO \"$USER\";"
```

Skip `createuser` if the role already exists.

## 2. Python Setup

From the repository root:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -r tools/requirements.txt
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL="postgresql:///kiwi_memory?host=/var/run/postgresql"
OPENAI_API_KEY="sk-..."
```

Do not commit `.env`.

The MCP servers load `.env` automatically from the repository root through
`tools/memory_common.py`. You do not need to duplicate secrets in the Cursor
config.

## 3. Index Project Memory

Build the searchable doc index:

```bash
./scripts/index-memory.sh
```

The indexer reads `README.md`, `AGENTS.md`, `BUILD_COMMANDS.md`,
`KNOWN_ISSUES.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`,
`LICENSE.md`, `plan.md`, and Markdown files under `docs/` and `tools/`.
Re-run this after significant documentation changes.

Verify search before wiring Cursor:

```bash
.venv/bin/python tools/search_memory.py "layout engine and navigation"
```

## 4. Set Up Context Memory

The context-memory server stores agent checkpoints in `agent_context_memory`.

Try the helper script first:

```bash
.venv/bin/python tools/setup_context_memory.py
```

If the app user cannot create tables, run the one-time SQL file as `postgres`:

```bash
sudo -u postgres psql kiwi_memory -f tools/setup_context_memory.sql
```

On hosts where `sudo -u postgres` is unavailable:

```bash
su postgres
psql kiwi_memory -f /path/to/kiwi/tools/setup_context_memory.sql
exit
```

Edit `tools/setup_context_memory.sql` and replace `"jaimie"` in the `GRANT`
statements with your OS username before running it on another machine.

Verify context memory:

```bash
PYTHONPATH=tools .venv/bin/python - <<'PY'
from context_memory import save_context, search_context
entry_id = save_context("smoke test", title="setup check", session_key="setup")
print("saved", entry_id)
print("search hits", len(search_context("smoke test", session_key="setup")))
PY
```

## 5. Configure Cursor

Cursor reads MCP server definitions from JSON. This repository ships a
project-local config at `.cursor/mcp.json`. That file is the recommended setup
because it travels with the repo and gives every developer the same server list.

### Project config

Open `.cursor/mcp.json` and confirm the paths match your checkout:

```json
{
  "mcpServers": {
    "kiwi-memory": {
      "command": "/absolute/path/to/kiwi/.venv/bin/python",
      "args": [
        "/absolute/path/to/kiwi/tools/mcp_memory_server.py"
      ],
      "cwd": "/absolute/path/to/kiwi"
    },
    "kiwi-context-memory": {
      "command": "/absolute/path/to/kiwi/.venv/bin/python",
      "args": [
        "/absolute/path/to/kiwi/tools/mcp_context_memory_server.py"
      ],
      "cwd": "/absolute/path/to/kiwi"
    }
  }
}
```

Replace `/absolute/path/to/kiwi` with your local clone path, for example:

```bash
pwd
```

Use the printed path in all three fields for each server.

Notes:

- Use absolute paths for `command`, `args`, and `cwd`.
- Keep secrets out of `mcp.json`. The servers read `DATABASE_URL` and
  `OPENAI_API_KEY` from repo-root `.env`.
- The committed file may contain another developer's path. Update it locally
  after clone.

### Global config (optional)

For a personal setup that applies to every workspace, create
`~/.cursor/mcp.json` with the same structure. Project config in
`.cursor/mcp.json` overrides a global server with the same name.

Cursor does not provide a separate UI for project vs global MCP files. Edit the
JSON directly.

### Reload Cursor

After creating or changing MCP config:

1. Save `.cursor/mcp.json`.
2. Open the command palette: **Ctrl+Shift+P**.
3. Run **Developer: Reload Window**.

A full Cursor restart also works if reload does not pick up the change.

### Verify in Cursor

1. Open **Settings**.
2. Go to **Tools & MCP**.
3. Confirm both servers appear and are connected:
   - `kiwi-memory` — 1 tool
   - `kiwi-context-memory` — 4 tools

If a server shows an error, open its log in the MCP settings panel and compare
the message with the troubleshooting section below.

### Manual server smoke test

These commands should start without import errors:

```bash
.venv/bin/python tools/mcp_memory_server.py
.venv/bin/python tools/mcp_context_memory_server.py
```

Each process waits on stdio. Stop it with **Ctrl+C**. Cursor launches the same
commands in the background when the servers are enabled.

## 5b. Cursor Hooks (memory enforcement)

Project hooks in `.cursor/hooks.json` enforce memory requirements:

| Hook | Script | Behavior |
| --- | --- | --- |
| `sessionStart` | `.cursor/hooks/memory_session_start.sh` | Injects memory-first instructions and recent context memory for the session. |
| `preCompact` | `.cursor/hooks/memory_pre_compact.sh` | Snapshots the conversation transcript into context memory before compaction. |

Hook handlers live in `tools/memory_hooks.py`. Transcript parsing is in
`tools/transcript_snapshot.py`.

Smoke-test the session-start hook:

```bash
echo '{"conversation_id":"test-session","session_id":"test-session"}' \
  | .cursor/hooks/memory_session_start.sh | jq .
```

Smoke-test pre-compaction with a transcript path:

```bash
echo '{"conversation_id":"test","transcript_path":"/path/to/transcript.jsonl"}' \
  | .cursor/hooks/memory_pre_compact.sh | jq .
```

Restart Cursor after changing hook files. Check the **Hooks** output channel if
a hook does not run.

## 6. MCP Tools

### Project memory (`kiwi-memory`)

| Tool | Arguments | Result |
| --- | --- | --- |
| `search_project_memory` | `query: str`, `limit: int = 8` | Matching doc snippets grouped by source path. |

Use before code changes to find specs, ADRs, known issues, and build notes.

### Context memory (`kiwi-context-memory`)

| Tool | Arguments | Result |
| --- | --- | --- |
| `save_context_memory` | `content: str`, `title: str = ""`, `session_key: str = ""`, `tags: list[str] = []` | Stores one context entry and returns its id. |
| `search_context_memory` | `query: str`, `limit: int = 8`, `session_key: str = ""` | Semantic search over saved context. |
| `list_context_memory` | `limit: int = 20`, `session_key: str = ""` | Recent saved entries, newest first. |
| `get_context_memory` | `entry_id: int` | Full content for one entry. |

Use a stable `session_key` such as a branch name, issue number, or task slug so
related entries stay grouped across compaction.

## Troubleshooting

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| MCP server missing after reload | Wrong config file or invalid JSON | Confirm `.cursor/mcp.json` exists at the repo root and parses as JSON. |
| MCP server fails immediately | Bad Python path or missing venv | Update `command` to your local `.venv/bin/python`. Re-run the Python setup steps. |
| `Missing Python dependency for Kiwi memory MCP` | Dependencies not installed in venv | Run `python -m pip install -r tools/requirements.txt` inside `.venv`. |
| OpenAI authentication errors | Missing or invalid API key | Set `OPENAI_API_KEY` in `.env`. |
| `relation "project_memory" does not exist` | Doc index table not created | Run the PostgreSQL setup SQL in section 1. |
| Empty project-memory search results | Index not built | Run `./scripts/index-memory.sh`. |
| `permission denied for schema public` during context setup | App user cannot create tables | Run `tools/setup_context_memory.sql` as `postgres`. |
| `relation "agent_context_memory" does not exist` | Context table not created | Run `tools/setup_context_memory.sql` as `postgres`. |
| PostgreSQL connection errors | Service down or role mismatch | Confirm PostgreSQL is running and `DATABASE_URL` matches your local setup. |

## Related Files

| File | Purpose |
| --- | --- |
| `.cursor/mcp.json` | Cursor MCP client configuration for this repo. |
| `.env.example` | Template for local secrets and database URL. |
| `tools/requirements.txt` | Python dependencies for memory tooling. |
| `tools/memory_common.py` | Shared env loading and pgvector helpers. |
| `scripts/index-memory.sh` | Index project documentation (wrapper). |
| `tools/index_memory.py` | Build the project doc index. |
| `tools/search_memory.py` | CLI search against project memory. |
| `tools/context_memory.py` | Save/search/list/get helpers for agent context. |
| `tools/setup_context_memory.py` | Context table setup helper. |
| `tools/setup_context_memory.sql` | One-time postgres DDL for context memory. |
| `tools/mcp-memory-setup.md` | Additional reference for memory internals and agent usage. |
