# MCP Memory Servers & Knowledge Base

Kiwi's persistent "brains" consist of three layers, all backed by PostgreSQL + pgvector and served as JSON-RPC 2.0 MCP servers over stdin/stdout.

| Layer | Binary | Purpose | MCP tools |
|---|---|---|---|
| Project memory | `kiwi-mcp-memory` | Project docs (README, AGENTS.md, etc.) | `search_project_memory` |
| Knowledge base | `kiwi-mcp-memory` | External reference manuals (Rust, eGUI, React…) | `search_knowledge_base` |
| Context memory | `kiwi-mcp-context` | Agent session notes | `save/search/list/get_context_memory` |

The project memory and knowledge base share the same `kiwi-mcp-memory` binary and database connection. The context memory runs as a separate process.

```
kiwi-ollama prompt flow (per turn):
  1. search_project_memory   — project docs
  2. search_knowledge_base   — reference manuals
  3. search_context_memory   — prior session notes
  4. local RAG               — live source code (in-memory, no DB)
  → combined context injected into Ollama
  → response streamed + auto-saved to context memory
```

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

> **Important:** The PostgreSQL `vector(N)` column is fixed at the dimension used when `--setup-db` was first run. Changing embedding backends requires dropping and recreating the tables, then re-indexing.

---

## `kiwi-mcp-memory` — Project Memory & Knowledge Base

One binary manages both the `project_memory` table (project docs) and the `knowledge_base` table (reference manuals).

### Initial setup

```bash
# Creates both project_memory and knowledge_base tables in one shot
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --setup-db
```

### Project docs indexing

```bash
# Index from current directory
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --index

# Or specify a path
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --index --root /path/to/repo
```

### What gets indexed (project docs)

- Root-level markdown files: `README.md`, `AGENTS.md`, `BUILD_COMMANDS.md`, `KNOWN_ISSUES.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `LICENSE.md`, `plan.md`
- All files under `docs/` and `tools/`

Files are chunked into 1800-character segments with 200-character overlap. Each chunk is hashed (SHA-256) so re-running `--index` skips already-indexed content.

---

## Knowledge Base

The knowledge base stores external reference documentation in named **collections** — one collection per tech stack or library. `kiwi-ollama` searches it automatically on every prompt alongside project memory.

### Indexing a single collection

```bash
# Rust book (HTML from rustup)
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --index-kb \
  --collection rust-book \
  --source ~/.rustup/toolchains/stable/share/doc/rust/html \
  --extensions html

# eGUI source (Rust docs)
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --index-kb \
  --collection egui \
  --source /path/to/egui/crates/egui/src \
  --extensions "rs,md"

# React (markdown docs)
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --index-kb \
  --collection react \
  --source /path/to/react.dev/src/content \
  --extensions "md,mdx"
```

`--extensions` is a comma-separated list of file extensions to include (no leading dot). Omit it to index all readable UTF-8 files. HTML/HTM files are automatically stripped of tags before chunking.

### Bulk indexing via config file

Create a `knowledge.toml` listing all collections:

```toml
[[collections]]
name = "rust-book"
source = "/home/user/.rustup/toolchains/stable/share/doc/rust/html"
extensions = ["html"]

[[collections]]
name = "egui"
source = "/home/user/egui/crates/egui/src"
extensions = ["rs", "md"]

[[collections]]
name = "eframe"
source = "/home/user/egui/crates/eframe/src"
extensions = ["rs", "md"]

[[collections]]
name = "react"
source = "/home/user/react-docs/src/content"
extensions = ["md", "mdx"]
```

Then index all collections in one command:

```bash
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --index-kb \
  --kb-config /path/to/knowledge.toml
```

### Managing collections

```bash
# List all indexed collections
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory --list-collections

# Re-run --index-kb at any time to add new content; existing chunks are skipped via hash dedup
```

### Running as an MCP server

```bash
DATABASE_URL="postgresql:///kiwi_memory" kiwi-mcp-memory
```

The server exposes both `search_project_memory` and `search_knowledge_base` tools.

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

### Tool: `search_knowledge_base`

```json
{
  "name": "search_knowledge_base",
  "arguments": {
    "query": "egui widget layout",
    "collection": "egui",
    "limit": 8
  }
}
```

Omit `collection` to search across all indexed collections. Returns results prefixed with `collection/path` so the source is always clear.

---

## kiwi-ollama integration

`kiwi-ollama` queries the knowledge base automatically. By default it searches all collections. Use `--kb-collections` to restrict which collections are searched:

```bash
# Search only Rust and eGUI docs
DATABASE_URL="postgresql:///kiwi_memory" kiwi-ollama --kb-collections "rust-book,egui"

# Search all collections (default)
DATABASE_URL="postgresql:///kiwi_memory" kiwi-ollama
```

Use `/status` inside the agent to confirm which collections are active.

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

Add to your MCP client configuration (e.g. Claude Code `settings.json`). Note that `kiwi-mcp-memory` exposes **both** `search_project_memory` and `search_knowledge_base`:

```json
{
  "mcpServers": {
    "kiwi-memory": {
      "command": "kiwi-mcp-memory",
      "env": {
        "DATABASE_URL": "postgresql:///kiwi_memory?host=/var/run/postgresql",
        "EMBED_BACKEND": "ollama",
        "OLLAMA_URL": "http://localhost:11434"
      }
    },
    "kiwi-context": {
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

### `knowledge_base`

| Column | Type | Notes |
|---|---|---|
| `id` | bigserial | Primary key |
| `collection` | text | Collection name (e.g. `rust-book`, `egui`, `react`) |
| `source_path` | text | Relative path from collection source root |
| `content` | text | Chunk text (HTML stripped for `.html`/`.htm` files) |
| `content_hash` | text | SHA-256; unique constraint prevents re-indexing |
| `embedding` | vector(N) | Same dimension as `project_memory` |
| `created_at` | timestamptz | |

Indexes: HNSW on `embedding`, B-tree on `collection`.

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

---

## Switching tech stacks

To switch from (e.g.) a Rust+eGUI project to a React project:

1. Index the React docs as a new collection — existing collections are unaffected:
   ```bash
   kiwi-mcp-memory --index-kb --collection react --source /path/to/react-docs --extensions "md,mdx"
   ```
2. Launch `kiwi-ollama` with the relevant collections:
   ```bash
   kiwi-ollama --kb-collections "react"
   ```

Collections accumulate; nothing is deleted when you add new ones. To remove stale content you can `DELETE FROM knowledge_base WHERE collection = 'old-collection'` directly in PostgreSQL.
