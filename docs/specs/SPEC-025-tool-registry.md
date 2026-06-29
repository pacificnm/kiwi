# SPEC-025: Kiwi Tool Registry & Permission System

## Status
Draft

## Overview

Replace the current hardcoded `KiwiTool::all_schemas()` approach with a centralised Tool Registry, provider adapters, and permission profiles. A tool is defined once; adapters translate it for each model provider; profiles control which agents can use which tools.

## Motivation

The current state has several problems:

- All 8 tools are always sent to Claude in every request — no filtering or access control
- OpenAI and Ollama providers have **zero tool support** — they get only the chat messages
- Adding a new tool requires coordinated changes in `tools.rs` and `tool_executor.rs` with no permission model
- Tool schemas are Claude-specific (`input_schema` format) — there is no abstraction for other providers
- The `KiwiTool` enum grows unboundedly with no way to scope tools to agent type

The goal:

```
One Tool Registry   →  Defined once
Many Agents         →  Different tool subsets per agent
Provider Adapters   →  Claude / OpenAI / Ollama format translation
Permission Profiles →  Declarative per-agent tool access control
```

## Architecture

```
Kiwi Tool Registry (KiwiToolDef)
         ↓
Tool Permission Profiles
         ↓
Agent Config (active_profile)
         ↓
Provider Adapter (Claude / OpenAI / Ollama)
         ↓
Model Provider API
```

### Tool Registry

Each tool is identified by a dotted name (`file.read`, `git.diff`, etc.) and carries a provider-agnostic JSON Schema:

```rust
pub struct KiwiToolDef {
    pub id: &'static str,               // "file.read"
    pub description: &'static str,
    pub input_schema: serde_json::Value, // JSON Schema object
}
```

`ToolRegistry::all()` returns `&'static [KiwiToolDef]` for every registered tool. The executor matches on `id` to dispatch execution.

### Provider Adapters

Each provider adapter converts `&[KiwiToolDef]` into the format that model expects:

| Provider | Wire format |
|----------|-------------|
| **Claude** | `{name, description, input_schema}` — current format, no change needed |
| **OpenAI** | `{type: "function", function: {name, description, parameters}}` where `parameters` = input_schema |
| **Ollama** | Same as OpenAI (models with tool support use OpenAI format) |

Adapters are free functions in `api_client.rs`:

```rust
fn tools_for_claude(tools: &[KiwiToolDef]) -> Vec<ClaudeToolSchema>
fn tools_for_openai(tools: &[KiwiToolDef]) -> Vec<OpenAiToolSchema>
```

**Key principle**: a tool should not care whether the agent is Claude, OpenAI, Ollama, or Gemini. You write `file.read` once; the adapter makes it speak the right language.

### Tool Permission Profiles

A profile is a named set of allowed tool IDs:

```rust
pub struct ToolProfile {
    pub name: &'static str,
    pub allowed: &'static [&'static str],
}
```

Pre-defined profiles:

| Profile | Tools |
|---------|-------|
| `coding` | file.read, file.write, file.list, file.search, file.grep, shell.run, git.status, git.diff, git.commit, cargo.check, cargo.test |
| `code_review` | file.read, file.search, file.grep, git.diff, cargo.check |
| `github` | github.issues, github.prs, git.branch, git.commit, git.status |
| `planner` | project.context, memory.search, file.search, file.grep |
| `all` | Every registered tool (backward-compatible default) |

### Agent Config Integration

`AgentSettings` gains a `tool_profile` field:

```rust
pub struct AgentSettings {
    // ... existing fields ...
    pub tool_profile: String,  // default "all"
}
```

`ProviderSettings` can override at the provider level:

```rust
pub struct ProviderSettings {
    // ... existing fields ...
    pub tool_profile: Option<String>,
}
```

The stream spawner resolves the active profile, filters the registry, and passes the filtered schema list to the provider adapter before building the API request body.

## Tool Inventory

### Existing Tools (rename to dotted namespace)

| Current name | New id | Executor change |
|---|---|---|
| `read_file` | `file.read` | Rename only |
| `write_file` | `file.write` | Rename only |
| `list_directory` | `file.list` | Rename only |
| `search_files` | `file.search` | Rename only |
| `search_content` | `file.grep` | Rename only |
| `run_bash` | `shell.run` | Rename only |
| `git_status` | `git.status` | Rename only |
| `git_diff` | `git.diff` | Rename only |

### New Tools to Implement

| id | Description | Implementation |
|---|---|---|
| `git.commit` | Stage all changes and commit with a message | `git add -A && git commit -m "{message}"` |
| `git.branch` | List branches, or create/checkout a branch | `git branch` / `git checkout -b` |
| `cargo.check` | Run `cargo check` in the repo root | subprocess, captures stdout+stderr |
| `cargo.test` | Run `cargo test` with optional filter | subprocess, captures output |
| `github.issues` | List open GitHub issues | `gh issue list --json` |
| `github.prs` | List open pull requests | `gh pr list --json` |
| `memory.search` | Search the kiwi-memory knowledge base | kiwi-memory MCP client call |
| `project.context` | Repo overview: structure, active branch, recent commits | composite (git log + file.list) |

## Implementation Phases

### Phase 1 — Tool Registry & Rename

**Goal**: Central registry; dotted IDs; no behavioural change for Claude.

- Introduce `KiwiToolDef` struct with `id`, `description`, `input_schema`
- Rename `KiwiTool` enum variants to match dotted IDs (`ReadFile` → `FileRead`, etc.)
- Replace `all_schemas()` with `ToolRegistry::all()` returning `&[KiwiToolDef]`
- Update `from_tool_use()` to match on dotted IDs (`"file.read"` etc.)
- Update `execute_tool()` dispatch to use new variants
- Update the one test that hard-codes schema count

### Phase 2 — Provider Adapters

**Goal**: OpenAI gets tools; Claude adapter is a thin wrapper; format translation is isolated.

- `tools_for_claude(tools) -> Vec<ClaudeToolSchema>`
- `tools_for_openai(tools) -> Vec<OpenAiToolSchema>`
- Wire OpenAI SSE parser to handle `tool_calls` delta (same pattern as Claude)
- Wire Ollama tool support (model-dependent; behind a feature flag or config)

### Phase 3 — Tool Permission Profiles

**Goal**: Each agent type uses only the tools it needs.

- Define `ToolProfile` and pre-defined profiles in `tools.rs`
- Filter `ToolRegistry::all()` by active profile before passing to adapter
- Add `tool_profile: String` to `AgentSettings` (default `"all"`)
- Add `tool_profile: Option<String>` to `ProviderSettings`
- Wire profile resolution in `services.rs` `spawn_claude_stream_effect`

### Phase 4 — New Tools

**Goal**: Expand the tool surface beyond current 8.

- `git.commit` — stage-all + commit
- `git.branch` — list / create / checkout
- `cargo.check` — check without running tests
- `cargo.test` — run tests, optional filter
- `github.issues` — list issues via `gh`
- `github.prs` — list PRs via `gh`
- `memory.search` — delegate to kiwi-memory MCP
- `project.context` — composite context summary

## Files Affected

```
crates/kiwi_core/src/agent/
  tools.rs          — KiwiToolDef, ToolRegistry, ToolProfile
  tool_executor.rs  — dispatch on dotted IDs; new tool impls
  api_client.rs     — tools_for_claude(), tools_for_openai()
  mod.rs            — updated exports
crates/kiwi_core/src/config/types.rs   — AgentSettings.tool_profile, ProviderSettings.tool_profile
crates/kiwi_gui/src/services.rs        — profile resolution; pass filtered tools to stream
```

## Non-Goals

- Tool sandboxing / capability enforcement at runtime (tools are trusted; permission profiles are advisory for the model, not a security boundary)
- Dynamic tool registration at runtime (all tools are compiled in)
- Cursor-specific tool adapters (deferred until Cursor Cloud Agents integration is designed)
