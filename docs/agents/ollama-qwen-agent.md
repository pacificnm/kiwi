# Ollama / qwen2.5-coder Agent

`kiwi-ollama` is a terminal coding agent for Kiwi that connects to a locally running [Ollama](https://ollama.com) server. It streams responses from **qwen2.5-coder** (or any Ollama-hosted model), indexes your repository for RAG-assisted context, and emits live status signals that Kiwi's status bar understands.

The crate lives at `plugins/kiwi_plugin_ollama` and ships both a binary (`kiwi-ollama`) and a Kiwi plugin library (cdylib) in one workspace member.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Configuration](#configuration)
   - [Kiwi config file](#kiwi-config-file)
   - [Environment variables](#environment-variables)
   - [CLI flags](#cli-flags)
4. [RAG — Codebase Context](#rag--codebase-context)
5. [Usage](#usage)
   - [Slash commands](#slash-commands)
   - [Status bar integration](#status-bar-integration)
6. [Plugin palette commands](#plugin-palette-commands)
7. [Troubleshooting](#troubleshooting)
8. [Architecture reference](#architecture-reference)

---

## Prerequisites

| Requirement | Notes |
|---|---|
| [Ollama](https://ollama.com) | Running locally (default `http://localhost:11434`) |
| `qwen2.5-coder` model | Pull with `ollama pull qwen2.5-coder` |
| `nomic-embed-text` model | Required for RAG; pull with `ollama pull nomic-embed-text` |
| Rust toolchain | `cargo` 1.75+ for building from source |

Verify Ollama is running and the models are available:

```bash
ollama list
# NAME                    ID              SIZE    MODIFIED
# qwen2.5-coder:latest    ...             4.7 GB  ...
# nomic-embed-text:latest ...             274 MB  ...
```

---

## Installation

### Build and install the binary

From the Kiwi workspace root:

```bash
# Install kiwi-ollama into ~/.cargo/bin
cargo install --path plugins/kiwi_plugin_ollama
```

Verify the install:

```bash
kiwi-ollama --help
```

### Install the Kiwi plugin (palette commands)

Copy the compiled dynamic library and manifest to Kiwi's plugin directory:

```bash
# Build first (if not already built by cargo install)
cargo build -p kiwi_plugin_ollama

PLUGIN_DIR="${HOME}/.config/kiwi/plugins/ollama"
mkdir -p "${PLUGIN_DIR}"

cp plugins/kiwi_plugin_ollama/plugin.toml "${PLUGIN_DIR}/"

# Linux
cp target/debug/libkiwi_plugin_ollama.so "${PLUGIN_DIR}/"

# macOS
# cp target/debug/libkiwi_plugin_ollama.dylib "${PLUGIN_DIR}/"
```

For release builds replace `debug` with `release` and add `--release` to the `cargo build` command.

---

## Configuration

### Kiwi config file

Tell Kiwi to use `kiwi-ollama` as its agent. Add the following to either:

- `.kiwi.toml` — project-level (repository root), overrides user config
- `~/.config/kiwi/config.toml` — user-level default for all projects

```toml
[agent]
command = "kiwi-ollama"

# Optional: pass CLI flags as args
# args = ["--no-rag"]

# Optional: configure via environment variables
[agent.env]
OLLAMA_URL        = "http://localhost:11434"
OLLAMA_MODEL      = "qwen2.5-coder"
OLLAMA_EMBED_MODEL = "nomic-embed-text"
```

Configuration precedence (highest to lowest):

```
CLI flags (--url, --model) → OLLAMA_* env vars → built-in defaults
```

### Environment variables

| Variable | Default | Description |
|---|---|---|
| `OLLAMA_URL` | `http://localhost:11434` | Base URL of the Ollama server |
| `OLLAMA_MODEL` | `qwen2.5-coder` | Chat model for code generation |
| `OLLAMA_EMBED_MODEL` | `nomic-embed-text` | Embedding model used by RAG |

Set these in `[agent.env]` in your Kiwi config (shown above) or export them in your shell before launching Kiwi.

### CLI flags

When using `args` in the Kiwi config, or running `kiwi-ollama` directly:

| Flag | Env var override | Default | Description |
|---|---|---|---|
| `--url <URL>` | `OLLAMA_URL` | `http://localhost:11434` | Ollama server base URL |
| `--model <MODEL>` | `OLLAMA_MODEL` | `qwen2.5-coder` | Chat model name |
| `--embed-model <MODEL>` | `OLLAMA_EMBED_MODEL` | `nomic-embed-text` | Embedding model for RAG |
| `--repo <PATH>` | — | `.` (current directory) | Repository root to index for RAG |
| `--no-rag` | — | RAG enabled | Skip codebase indexing entirely |

#### Examples

Use a different model for a specific project:

```toml
# .kiwi.toml
[agent]
command = "kiwi-ollama"
args = ["--model", "qwen2.5-coder:14b"]
```

Point to a remote Ollama server:

```toml
[agent]
command = "kiwi-ollama"
[agent.env]
OLLAMA_URL = "http://192.168.1.50:11434"
```

Disable RAG when working in a very large monorepo:

```toml
[agent]
command = "kiwi-ollama"
args = ["--no-rag"]
```

---

## RAG — Codebase Context

RAG (Retrieval-Augmented Generation) indexes your repository's source files and injects the most relevant code snippets as context into each prompt, so the model answers questions about *your* codebase without needing to read every file.

### How it works

1. **Startup**: a background thread walks the repository root (or `--repo <PATH>`) and embeds eligible files using Ollama's `/api/embed` endpoint.
2. **Per prompt**: the user's message is embedded and compared against the index using cosine similarity. The top 5 matching chunks are prepended to the conversation as context.
3. **Cache**: embeddings are saved to `~/.cache/kiwi-ollama/embeddings.json` keyed by file path and mtime. On subsequent runs, only changed or new files are re-embedded — full re-indexing is incremental.

The main REPL is usable immediately on startup; RAG context becomes available once background indexing completes.

### Indexed file types

`.rs` `.ts` `.tsx` `.js` `.jsx` `.py` `.go` `.java` `.c` `.cpp` `.h` `.hpp` `.md` `.toml` `.yaml` `.yml` `.json`

### Skipped directories

`.git` `target` `node_modules` `.venv` `__pycache__` `dist` `build` `.cache` `.mypy_cache`

### Limits

| Parameter | Value |
|---|---|
| Maximum files indexed | 2,000 (warning printed if exceeded) |
| Chunk size | ~1,800 characters |
| Chunk overlap | 200 characters |
| Top-k results per prompt | 5 |
| Cache location | `$XDG_CACHE_HOME/kiwi-ollama/embeddings.json` (falls back to `~/.cache/kiwi-ollama/`) |

### Changing the embedding model

Any model available in Ollama that supports `/api/embed` can be used. `nomic-embed-text` is the recommended default for code because it handles mixed natural-language and code text well and is small enough to run alongside the chat model.

```toml
[agent.env]
OLLAMA_EMBED_MODEL = "mxbai-embed-large"  # alternative
```

---

## Usage

Once `kiwi-ollama` is running as the Kiwi agent (visible in the Agent tab), simply type your coding request and press Enter. The agent maintains a rolling conversation history of up to 20 turns.

```
> explain how the AgentManager handles multiple PTY sessions
thinking: reasoning about your request
running tool: searching codebase
[streams response...]
completed: response ready
```

### Slash commands

These are entered at the prompt in the Agent tab:

| Command | Effect |
|---|---|
| `/clear` | Clears the entire conversation history |
| `/status` | Shows current model, Ollama URL, and RAG index state |
| `/help` | Lists available slash commands and env vars |

### Status bar integration

`kiwi-ollama` emits keywords that Kiwi's status inference engine (`AgentStatus`) picks up from the last 32 lines of scrollback:

| Output line | Kiwi status | When emitted |
|---|---|---|
| `thinking: …` | Thinking | Before calling the model |
| `running tool: searching codebase` | Executing | When RAG retrieves context |
| `completed: response ready` | Success | After a full response streams |
| `error: …` | Error | On connection failure or parse error |

The status label in Kiwi's bottom-right corner updates live as responses stream in.

### Conversation context

The agent keeps the last 20 turns (40 messages) in memory. When this limit is reached the oldest messages are pruned automatically. Use `/clear` to start fresh without restarting the process.

RAG context is injected *per turn* only and is not counted toward the conversation history limit.

---

## Plugin palette commands

When the cdylib plugin is installed (see [Installation](#installation)) and `[plugins] enabled = true` is set in your Kiwi config, the following commands appear in Kiwi's command palette (`Ctrl+P` or `;`):

| Command ID | Palette title | Effect |
|---|---|---|
| `ollama.restart` | Ollama: Restart Agent | Placeholder — use Kiwi's built-in `Agent: Restart` |
| `ollama.clear_context` | Ollama: Clear Context | Placeholder — type `/clear` in the agent prompt |

These are stubs in the current release. They will be wired to the agent's stdin when the plugin API gains event subscription capabilities (SPEC-020 Phase 2).

---

## Troubleshooting

### `error: cannot connect to Ollama at http://localhost:11434`

Ollama is not running or not reachable. Start it with:

```bash
ollama serve
```

Or check with:

```bash
curl http://localhost:11434/api/tags
```

### `warning: RAG disabled: embedding failed (pull nomic-embed-text?)`

The embedding model is not available on the Ollama server. Pull it:

```bash
ollama pull nomic-embed-text
```

If you do not want RAG, use `args = ["--no-rag"]` in your Kiwi config.

### Model not found / HTTP 404 on chat

Ensure the chat model is pulled:

```bash
ollama pull qwen2.5-coder
```

Or point to a model you already have:

```toml
[agent.env]
OLLAMA_MODEL = "llama3.2:3b"   # any model in `ollama list`
```

### RAG indexing is slow on first run

Initial embedding of a large codebase takes time — roughly 5–15 ms per chunk. The REPL is usable immediately and RAG enrichment kicks in once the background thread finishes. On subsequent runs, only changed files are re-embedded.

To skip indexing entirely for the session:

```bash
kiwi-ollama --no-rag
```

### Plugin not loading in Kiwi

Check that `plugin.toml` and the `.so` / `.dylib` file are both present in `~/.config/kiwi/plugins/ollama/`, and that `[plugins] enabled = true` is set in your Kiwi config. Run Kiwi with logging to see plugin load messages.

---

## Architecture reference

- **ADR-023** — Ollama agent design decisions: `docs/architecture/adr/ADR-023-ollama-agent-design.md`
- **SPEC-010** — Agent service contract (PTY spawning, status inference): `docs/specs/SPEC-010-agent-service.md`
- **SPEC-020** — Plugin framework (cdylib loading, palette registration): `docs/specs/SPEC-020-plugin-framework.md`
- **ADR-017** — Multi-agent future design: `docs/architecture/adr/ADR-017-multi-agent-future-design.md`
- **Source** — `plugins/kiwi_plugin_ollama/src/`
