# MCP Memory Servers

Kiwi ships two MCP (Model Context Protocol) servers for persistent memory, both written in Rust and backed by PostgreSQL + pgvector.

| Server | Binary | MCP tools |
|---|---|---|
| Project memory | `kiwi-mcp-memory` | `search_project_memory` |
| Context memory | `kiwi-mcp-context` | `save_context_memory`, `search_context_memory`, `list_context_memory`, `get_context_memory` |

Both servers communicate over **stdin/stdout JSON-RPC 2.0** and are configured entirely through environment variables.

---

## Prerequisites

- PostgreSQL with the [pgvector](https://github.com/pgvector/pgvector) extension installed
- An embedding backend: Ollama (default) or OpenAI

## Environment Variables

| Variable | Default | Notes |
|---|---|---|
| `DATABASE_URL` | `postgresql:///kiwi_memory?host=/var/run/postgresql` | PostgreSQL connection string |
| `EMBED_BACKEND` | `ollama` | `ollama` or `openai` |
| `OLLAMA_URL` | `http://localhost:11434` | Ollama API base URL |
| `OLLAMA_EMBED_MODEL` | `nomic-embed-text` | Produces 768-dim vectors |
| `OPENAI_API_KEY` | (required if `openai`) | |
| `OPENAI_EMBED_MODEL` | `text-embedding-3-small` | Produces 1536-dim vectors |

> **Important:** The PostgreSQL `vector(N)` column is fixed at the dimension used when `--setup-db` was first run. Changing embedding backends requires dropping and recreating the table, then re-indexing.

---

## `kiwi-mcp-memory` — Project Memory

Indexes project documentation and exposes semantic search over it.

### Setup

```bash
# Create the project_memory table
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --setup-db

# Index project docs (run from the repo root, or pass --root /path/to/repo)
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --index
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --index --root /path/to/repo
```

### What gets indexed

- Root-level markdown files: `README.md`, `AGENTS.md`, `BUILD_COMMANDS.md`, `KNOWN_ISSUES.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `LICENSE.md`, `plan.md`
- All files under `docs/` and `tools/`

Files are chunked into 1800-character segments with 200-character overlap. Each chunk is hashed (SHA-256) so re-running `--index` skips already-indexed content.

### Running as an MCP server

```bash
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory
```

### Tool: `search_project_memory`

```json
{
  "name": "search_project_memory",
  "arguments": {
    "query": "agent PTY architecture",
    "limit": 8
  }
}
```

Returns the top-`limit` chunks ranked by cosine similarity, with source path and relevance score.

---

## `kiwi-mcp-context` — Agent Context Memory

Saves and retrieves agent session notes across conversations.

### Setup

```bash
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-context --setup-db
```

### Running as an MCP server

```bash
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-context
```

### Tools

#### `save_context_memory`

```json
{
  "name": "save_context_memory",
  "arguments": {
    "content": "Refactored the PTY spawn logic to use async I/O.",
    "title": "PTY async refactor notes",
    "session_key": "main:abc123",
    "tags": ["pty", "async", "refactor"]
  }
}
```

Returns `"Saved context memory entry #<id>"`.

#### `search_context_memory`

```json
{
  "name": "search_context_memory",
  "arguments": {
    "query": "PTY spawn",
    "limit": 5,
    "session_key": "main:abc123"
  }
}
```

`session_key` is optional — omit to search across all sessions.

#### `list_context_memory`

```json
{
  "name": "list_context_memory",
  "arguments": {
    "limit": 20,
    "session_key": "main:abc123"
  }
}
```

Returns recent entries ordered by creation time, with 500-character previews.

#### `get_context_memory`

```json
{
  "name": "get_context_memory",
  "arguments": { "entry_id": 42 }
}
```

Returns full content (up to 10,000 characters) for a single entry.

---

## Wiring into Claude Code / MCP clients

Add to your MCP client configuration (e.g. Claude Code `settings.json`):

```json
{
  "mcpServers": {
    "project-memory": {
      "command": "kiwi-mcp-memory",
      "env": {
        "DATABASE_URL": "postgresql:///kiwi_memory?host=/var/run/postgresql",
        "EMBED_BACKEND": "ollama",
        "OLLAMA_URL": "http://localhost:11434"
      }
    },
    "context-memory": {
      "command": "kiwi-mcp-context",
      "env": {
        "DATABASE_URL": "postgresql:///kiwi_memory?host=/var/run/postgresql",
        "EMBED_BACKEND": "ollama",
        "OLLAMA_URL": "http://localhost:11434"
      }
    }
  }
}
```

---

## Schema reference

### `project_memory`

| Column | Type | Notes |
|---|---|---|
| `id` | bigserial | Primary key |
| `source_path` | text | Relative path from project root |
| `content` | text | Chunk text |
| `content_hash` | text | SHA-256; unique constraint prevents re-indexing |
| `embedding` | vector(N) | N = 768 (Ollama) or 1536 (OpenAI) |
| `created_at` | timestamptz | |

### `agent_context_memory`

| Column | Type | Notes |
|---|---|---|
| `id` | bigserial | Primary key |
| `session_key` | text | Agent session identifier |
| `title` | text | Short descriptive title |
| `content` | text | Full note content |
| `tags` | text[] | Optional categorization tags |
| `embedding` | vector(N) | |
| `created_at` | timestamptz | |
