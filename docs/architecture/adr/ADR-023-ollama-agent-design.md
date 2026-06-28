# ADR-023: Ollama Agent Design

## Status

Accepted

## Context

Kiwi's agent system is intentionally model-agnostic: any executable can be configured as the agent via `[agent] command`. Users running a local Ollama server with qwen2.5-coder need a first-class agent binary that:

1. Connects to Ollama's HTTP streaming API and renders responses in Kiwi's PTY pane
2. Provides RAG-assisted context from the local repository without requiring external services
3. Ships as a workspace crate (`plugins/kiwi_plugin_ollama`) alongside the Kiwi plugin cdylib

## Decision

### Crate structure: hybrid binary + cdylib

The `plugins/kiwi_plugin_ollama` crate ships two targets from one source tree:

- **`kiwi-ollama` binary** — the agent that Kiwi spawns via PTY. Handles Ollama API communication, conversation context, and RAG.
- **cdylib plugin** — loaded by Kiwi's plugin system; registers palette commands (`ollama.restart`, `ollama.clear_context`). Stubs today; wired to agent stdin when plugin API gains event subscriptions (SPEC-020 Phase 2).

`lib.rs` and `main.rs` are separate crate roots. The shared modules (`ollama.rs`, `context.rs`, `rag.rs`) are declared in `main.rs` only; `lib.rs` imports only `kiwi_plugin_api`. This avoids the limitation that a `cdylib` does not expose a Rust ABI to its companion binary target.

### HTTP client: `ureq` (synchronous)

`ureq 2.x` was chosen over `reqwest` (async) for the agent binary because:

- The REPL is inherently sequential: read prompt → send request → stream response → loop.
- Synchronous streaming (`response.into_reader()` → `BufReader` → line iterator) maps directly onto NDJSON chunk-by-chunk parsing without executor overhead.
- Smaller dependency footprint: no `hyper`, `h2`, or `tokio` runtime in the binary itself.
- TLS disabled (`default-features = false`) since Ollama runs on `http://localhost` by default, reducing compile time further.

`tokio` remains available in the workspace for other crates; it is not pulled into the agent binary.

### Streaming protocol: NDJSON line iteration

Ollama's `/api/chat` endpoint with `stream: true` returns newline-delimited JSON chunks:

```json
{"model":"qwen2.5-coder","message":{"role":"assistant","content":"Hello"},"done":false}
{"model":"qwen2.5-coder","message":{"role":"assistant","content":"!"},"done":false}
{"model":"qwen2.5-coder","done":true,"done_reason":"stop"}
```

Each chunk is deserialized independently. Content tokens are printed to stdout with an explicit `flush()` call so Kiwi's PTY reader receives sub-50ms chunks.

### RAG: Ollama embeddings + in-memory cosine similarity

Rather than adding a vector database dependency (SQLite + sqlite-vec, pgvector, Qdrant), the RAG index is:

- An in-memory `Vec<IndexEntry>` loaded from a JSON cache file at `~/.cache/kiwi-ollama/embeddings.json`
- Staleness detected by comparing file `mtime` to the cached value; only changed files are re-embedded
- Cosine similarity computed in pure Rust (no BLAS) — sufficient for repositories up to ~2,000 files × ~5 chunks each (≤10,000 dot products per query, negligible latency)
- Background thread: indexing runs concurrently with the REPL via `std::sync::mpsc`; the main loop polls the channel non-blocking on each prompt

Embedding model: `nomic-embed-text` (Ollama-hosted). This keeps all inference local with no external API keys required.

### Status keyword protocol

The agent binary emits lines that align with Kiwi's `infer_status_from_text` patterns (see `kiwi_core::agent::status`):

| Output | Kiwi `AgentStatus` |
|---|---|
| `thinking: …` | `Thinking` |
| `running tool: …` | `Executing` |
| `completed: …` | `Success` |
| `error: …` | `Error` |

This requires no changes to Kiwi core: the agent binary self-signals status via stdout.

### Conversation context management

Conversation history is kept in memory as a `Vec<ChatMessage>` capped at 20 turns (40 messages). Oldest messages are pruned when the cap is exceeded. RAG context is injected as a temporary exchange before the conversation history on each turn and is not persisted to the message list.

## Consequences

### Positive

- Zero changes to Kiwi core (`kiwi`, `kiwi_core`) — the agent plugs in via the existing `[agent] command` configuration.
- Fully local inference: no API keys, no external services, no data leaves the machine.
- RAG is incremental after the first run; subsequent starts are fast.
- The cdylib plugin gives Kiwi users palette discoverability without requiring core changes.

### Negative

- Conversation history is in-process memory only; it does not survive agent restarts. A future enhancement could persist history to disk.
- Cosine similarity scales at O(n) with the number of chunks. Repositories with >2,000 files may need a proper vector index in a future release.
- The cdylib palette commands are stubs until SPEC-020 Phase 2 lands. Users must type `/clear` in the agent pane; the palette command cannot trigger it yet.
- TLS disabled by default: HTTPS Ollama endpoints require overriding the `ureq` features in a fork or a future config option.

## Alternatives Considered

| Alternative | Rejection Rationale |
|---|---|
| `ollama run qwen2.5-coder` directly as `[agent] command` | Works but no RAG, no structured status keywords, no conversation management |
| `reqwest` async HTTP client | Async runtime unnecessary for a sequential REPL; larger compile footprint |
| `rusqlite` + sqlite-vec for vector store | Adds a C dependency (libsqlite3); JSON cache sufficient for the target scale |
| Plugin-only cdylib (no binary) | Current plugin API (`extern "C" fn() -> PluginResult`) cannot stream output to Kiwi's PTY pane |
| Extend plugin API for streaming | Premature given SPEC-020 Phase 2 scope; deferred |

## Follow-up Work

- Persist conversation history to disk between sessions (e.g., `~/.cache/kiwi-ollama/history.json`)
- Wire `ollama.clear_context` palette command to the agent's stdin when SPEC-020 Phase 2 lands
- Approximate nearest-neighbour index (e.g., HNSW) for RAG when repository scale exceeds 2,000 files
- TLS support via optional `ureq` feature flag for HTTPS Ollama endpoints
- `OLLAMA_CONTEXT_LENGTH` configuration for models with large context windows (e.g., `qwen2.5-coder:32b`)
