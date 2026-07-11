# Kiwi Tauri + Agent Host v1 Implementation Plan

## Status: P1 Explorer (+ context menu), P2 Monaco editor, P6 Source Control (commit + graph) & agent PTY live; integrated terminal (P4) landed

Two coordinated changes for Kiwi:

1. **Desktop UI** — Replace egui/eframe (`nest-gui`) with Tauri + React + Nest default theme (`cbre-light`), while **preserving the VS Code-style workbench layout** from the current egui shell.
2. **Agent strategy** — Stop expanding custom `nest-agent` tools; explore **mature coding agents** launched via Ollama (`ollama launch … --model qwen3.5`).

Related:

- [nest-tauri v1](../../../../docs/plan/nest-tauri-v1.md) — platform bootstrap
- [Desktop template](../../../../templates/desktop/README.md) — Swift / airtable-sync pattern
- [agent-mcp-v1.md](../agent-mcp-v1.md) — **superseded in spirit** by this plan (keep crates for CLI / fallback)
- [agent-workflow-v2.md](../agent-workflow-v2.md) — native tool context (deprioritized)

---

## Goals

| Goal | Success criteria |
| --- | --- |
| Tauri shell | `kiwi` desktop runs from `ui/` + `src-tauri/` with `cbre-light` theme |
| Layout parity | Activity bar, sidebar, editor, bottom panel, AI panel, status bar match egui spatial model |
| Agent pivot | User can run Claude Code, Codex, or OpenCode against local `qwen3.5` without Kiwi-maintained tool loop |
| Incremental | egui desktop can coexist during migration; CLI unchanged until cutover |

## Non-goals (v1)

- Pixel-perfect egui recreation (React/Tailwind is the target look)
- Replacing Cursor as the primary agent host for Nest monorepo work
- `@nest/ui` npm package (copy template + workbench components locally first)
- Cloud-only models (local `qwen3.5` is the default spike target)

---

## Part A — Workbench layout (keep egui structure)

Kiwi is **not** a ribbon app (Swift, airtable-sync). It is a **Cursor/VS Code workbench**. The template `AppShell` + `Ribbon` are reused only for shared primitives (`Icon`, `Toast`, `StatusBar`, `ConfirmDialog`, theme CSS) — not the ribbon layout.

### Current egui panel stack

Source: [`workbench/mod.rs`](../../desktop/crates/kiwi/src/workbench/mod.rs)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ Top menu bar (36px) — File / Edit / View / …                             │
├──┬────────────┬────────────────────────────────────┬───────────────────────┤
│A │ Sidebar    │ Central editor (tabs)              │ AI panel (360px)      │
│c │ (260px)    │                                    │ chat + agent controls │
│t │ Explorer,  │                                    │                       │
│  │ Search,    ├────────────────────────────────────┤                       │
│4 │ SCM, …     │ Bottom panel (~200px)              │                       │
│8 │            │ Terminal · Problems · Logs · …     │                       │
├──┴────────────┴────────────────────────────────────┴───────────────────────┤
│ Status bar (nest-gui StatusBarService)                                   │
└──────────────────────────────────────────────────────────────────────────┘
```

Constants to preserve in CSS:

| Region | Width / height | egui constant |
| --- | --- | --- |
| Activity bar | 48px fixed | `ACTIVITY_BAR_WIDTH` |
| Sidebar | 260px default, 200–400 resizable | `SIDEBAR_WIDTH` |
| AI panel | 360px default, resizable | `AI_PANEL_WIDTH` |
| Menu bar | 36px | `TITLE_BAR_HEIGHT` |
| Bottom panel | 200px default, min 80 | `DEFAULT_HEIGHT` |
| Chat prompt block | ~192px | `AI_PROMPT_SECTION_HEIGHT` |

### React component map

```text
ui/src/workbench/
├── WorkbenchShell.tsx      # grid: menu + body + status
├── ActivityBar.tsx         # Activity enum → icon column
├── Sidebar.tsx             # dispatches by activity
│   ├── ExplorerPanel.tsx
│   ├── SearchPanel.tsx
│   ├── SourceControlPanel.tsx
│   ├── IssuesPanel.tsx
│   ├── TasksPanel.tsx
│   ├── AgentSettingsPanel.tsx   # becomes agent runtime config
│   ├── ToolsPanel.tsx
│   └── ExtensionsPanel.tsx
├── EditorArea.tsx          # tab bar + Monaco (or CodeMirror) views
├── BottomPanel.tsx         # tab bar + terminal / logs / tool activity
├── AiPanel.tsx             # chat transcript + prompt + model/runtime picker
├── MenuBar.tsx
└── StatusBar.tsx           # thin wrapper over shell StatusBar
```

Layout: CSS grid or flex with `react-resizable-panels` (or similar) for sidebar, AI panel, and bottom dock — mirrors egui `SidePanel` / `TopBottomPanel` behavior.

### Editor

egui syntax highlighting and diff tabs are **not** portable. v1 choices:

| Option | Pros | Cons |
| --- | --- | --- |
| **Monaco** (recommended) | VS Code parity, diff, language services | Bundle size |
| CodeMirror 6 | Lighter | More wiring for diff/PR views |

Port order: plain file tabs → diff → GitHub issue/PR read-only views (reuse existing Rust `nest-github` IPC).

### Tauri backend (`src-tauri/`)

Follow airtable-sync / Swift pattern:

| Concern | Approach |
| --- | --- |
| Bootstrap | `TauriApp::new("kiwi")`, `ThemeModule::default()`, logging |
| File I/O | Tauri commands wrapping `nest-file` / existing Kiwi file logic |
| GitHub | Commands wrapping `nest-github` (issues, PRs, merge) |
| Workspace | Project root, recent projects, file watcher events → webview |
| Terminal | `portable-pty` or Tauri plugin; stream to bottom panel |
| Agent | See Part B — subprocess / PTY host, not `nest-agent` loop in v1 |

Keep `desktop/crates/kiwi` Rust crate for **shared logic** (project config, GitHub helpers, file ops) invoked from Tauri commands. Deprecate `workbench/` egui module after cutover.

### Phased UI migration

| Phase | Deliverable |
| --- | --- |
| **P0** | Scaffold `ui/` + `src-tauri/`, theme CSS, empty workbench grid — **done** |
| **P0.1** | `cursor-dark` built-in theme + `ThemeModule::with_active` — **done** |
| **P6 (spike)** | Agent Panel embedded PTY running `ollama launch <runtime> --model <model>` (xterm.js + portable-pty) — **done** |
| **P1** | Activity bar + Explorer (tree, open file → editor tab) — **done** |
| **P2** | Monaco editor + save; syntax highlighting |
| **P3** | Search, Source Control, Issues (port existing sidebar logic) |
| **P4** | Bottom panel: Terminal + Logs |
| **P5** | AI panel shell (prompt UI without agent backend) |
| **P6** | Agent host integration (Part B) |
| **P7** | Remove egui `GuiApp` path; `./build desktop` → Tauri only |

CLI (`kiwi chat`, `kiwi agent`, `kiwi file`) stays on `nest-cli` until agent strategy is decided.

---

## Part B — Agent strategy pivot

### Problem with current approach

Kiwi today runs a **custom agent loop** (`nest-agent` → `nest-ai-ollama` → MCP + native file tools). That required building and maintaining:

- Tool registry, policy, auto-run flags
- Native tools: `search_files`, `search_code`, `read_file`, `write_file`, `cargo_check`, …
- MCP stdio hub, reconnect, server toggles
- Weak-model workarounds ([agent-workflow-v2.md](../agent-workflow-v2.md))

This converges slowly toward Cursor parity. External coding agents already solve tool orchestration, terminal use, and multi-step edits.

### New direction: Ollama agent integrations

[Ollama `launch`](https://ollama.com/blog/launch) (v0.15+) wires local/cloud models into existing agent CLIs **without manual env config**:

```bash
ollama pull qwen3.5
ollama launch claude --model qwen3.5
ollama launch codex --model qwen3.5
ollama launch opencode --model qwen3.5
```

[Qwen 3.5](https://ollama.com/library/qwen3.5) is explicitly listed for these integrations. Highlights:

- Tool calling, 256K context (model-dependent), vision, thinking mode
- Sizes 0.8b–122b; **9b+** realistic for agent work on modest GPU
- Strong agent/coding benchmarks vs prior Qwen generations

**Context length:** Coding agents need **≥ 64k** tokens. Set in Ollama app/CLI (`OLLAMA_CONTEXT_LENGTH=64000`) — see [Ollama context docs](https://github.com/ollama/ollama/blob/main/docs/context-length.mdx).

### Integrations to evaluate

| Integration | Command | Notes |
| --- | --- | --- |
| **OpenCode** | `ollama launch opencode --model qwen3.5` | Open source, terminal-native; good embed candidate |
| **Claude Code** | `ollama launch claude --model qwen3.5` | Polished UX; proprietary CLI |
| **Codex** | `ollama launch codex --model qwen3.5` | OpenAI Codex CLI via Ollama |
| **OpenClaw** | `ollama launch openclaw --model qwen3.5` | Listed on qwen3.5 page |
| **Hermes Agent** | `ollama launch hermes --model qwen3.5` | Nous Research agent |

Config-only (no launch): `ollama launch <tool> --config`

### Host patterns (pick after spike)

```text
┌─────────────────────────────────────────────────────────────┐
│ Kiwi Tauri (workbench)                                      │
│  ┌─────────────────┐  ┌──────────────────────────────────┐ │
│  │ AI panel (UI)   │  │ Bottom: Terminal tab (PTY)        │ │
│  │ mirror / chrome │  │ running `ollama launch opencode`  │ │
│  └────────┬────────┘  └──────────────────────────────────┘ │
│           │ spawn / attach                                    │
│           ▼                                                   │
│   External agent process (Claude / Codex / OpenCode)         │
│           │                                                   │
│           ▼                                                   │
│   Ollama API (qwen3.5) + agent's own tools (shell, edits)    │
└─────────────────────────────────────────────────────────────┘
```

| Pattern | Description | Kiwi work |
| --- | --- | --- |
| **A. Embedded PTY** | Agent runs in bottom terminal; AI panel shows shortcuts/status | Lowest integration cost; v1 default — **implemented** |
| **B. Subprocess + log parse** | Kiwi spawns agent, parses structured output into chat | Medium; depends on agent stdout format |
| **C. Native embed** | Agent exposes SDK/API (if available) | Highest; only if spike proves value |

### Implemented (Pattern A)

The Agent Panel (`ui/src/workbench/AgentPanel.tsx`) embeds an **xterm.js** terminal
wired to a **`portable-pty`** session in `src-tauri`:

| Layer | File | Role |
| --- | --- | --- |
| PTY session | `src-tauri/src/agent.rs` | `AgentPty` managed state; spawns `ollama launch <runtime> --model <model>` in a PTY |
| IPC | `src-tauri/src/commands.rs` | `agent_launch` / `agent_input` / `agent_resize` / `agent_stop` / `agent_status` |
| Events | — | `kiwi://agent-output` (base64 PTY bytes), `kiwi://agent-exit` |
| Terminal | `ui/src/lib/agent.ts` + `AgentPanel.tsx` | xterm.js; runtime picker + model field; Launch/Stop |

Defaults: runtime **Claude Code**, model **`qwen3.5:2b`**. Output streams as base64
(safe across UTF-8 chunk splits) → decoded to `Uint8Array` → `term.write`. Keystrokes
(`term.onData`) → `agent_input`. `ResizeObserver` → `fit` + `agent_resize`.

**Prereqs:** Ollama ≥ 0.15 on `PATH`, model pulled on the inference server, context ≥ 64k
for real work. `ollama` is spawned with inherited `PATH`/`HOME`.

**Split runtime (local agent + remote inference):** Codex/Claude/OpenCode run as a
**local** process on the dev machine; model inference runs on **`server.lan`
(192.168.88.10:11434)**. The panel's host field sets `OLLAMA_HOST` on the spawned
process, so `ollama launch` configures the agent to hit the remote Ollama:

```text
dev machine: Kiwi ──spawn──> ollama launch codex --model qwen3.5   (OLLAMA_HOST=http://192.168.88.10:11434)
                                        └─ HTTP ─> server.lan:11434  (GPU, model loaded)
```

Default host `192.168.88.10:11434` (matches the egui Kiwi `[agent]` config). A bare
`host:port` is normalized to `http://host:port`.

**The `ollama` CLI must be installed on the *local* machine** (v0.15+) even though
inference is remote — `ollama launch` is a local command that spawns/configures the
agent and points it at `OLLAMA_HOST`. If it is missing the panel shows
`failed to launch … Is Ollama v0.15+ installed and on PATH?`. The spawn `PATH` is
augmented with `/usr/local/bin`, `/opt/homebrew/bin`, `~/.local/bin`, etc. because
GUI apps often inherit a minimal `PATH`.

IPC errors are rendered via `formatIpcError` (extracts `NestError.message` + code)
so failures are readable instead of `[object Object]`. Backend logs launch attempts
and failures under target `kiwi` in `desktop/logs/kiwi`.

**Inline-plugin ACL (required):** `src-tauri/build.rs` must register the inline `kiwi`
plugin via `tauri_build::try_build(Attributes::new().plugin("kiwi", InlinedPlugin::new()
.commands([...]).default_permission(AllowAllCommands)))`, and `capabilities/default.json`
must list `"kiwi:default"`. Without both, Tauri v2 denies every `plugin:kiwi|*` invoke
("plugin not found / not allowed"). Keep the command list in `build.rs` in sync with
`kiwi_plugin`'s `generate_handler!`.

Recommendation: **spike Pattern A** for all three of OpenCode, Claude Code, Codex on `qwen3.5`; compare UX and reliability before investing in chat mirroring.

---

## Implemented (P1 — Explorer + editor tabs)

The Explorer sidebar and file tabs are ported from the egui workbench
(`desktop/crates/kiwi/src/workbench/explorer`). Rust owns scoped, safe file I/O;
React owns all presentation.

### Backend (`src-tauri/src/workspace.rs`)

`Workspace` is managed Tauri state wrapping a `nest_file::FileService` scoped to
the project root (so path traversal / symlink escape is rejected by
`SafePathResolver`). Root resolution precedence: `KIWI_PROJECT_ROOT` env →
`[project].root` in `desktop/config.toml` → nearest `.git` / Cargo-workspace
ancestor → current dir.

| Command | Returns | Notes |
| --- | --- | --- |
| `workspace_info` | `{ root, name }` | Active project root + display name |
| `workspace_list` | `[{ name, relPath, isDir }]` | Dirs first, then case-insensitive name; hides `.git`, `target`, `node_modules`, `.venv`, `dist`, `build` |
| `workspace_read` | `{ relPath, content }` | Rejects directories, files > 2 MiB, and binary (NUL) / non-UTF-8 content |
| `workspace_open` | `{ root, name }` | Re-scopes the `FileService` to a new folder (Open Folder) |

Commands live in the inline `kiwi` plugin — **`build.rs` `KIWI_COMMANDS` must
list them** (ACL), same rule as the agent commands.

### Frontend (`ui/src/workbench`)

- `lib/workspace.ts` — IPC wrappers for the four commands.
- `state.tsx` — `WorkbenchProvider` / `useWorkbench`: holds `workspace`, open
  editor `tabs`, `activePath`, plus `openFile` / `closeTab` / `openWorkspace` /
  `refreshWorkspace`. File reads are lazy and cached per tab.
- `ExplorerPanel.tsx` — lazy tree (loads children on first expand, dirs-first),
  refresh + collapse-all, error rows. Clicking a file opens an editor tab.
- `EditorArea.tsx` — tab bar (close buttons) + read-only `<pre>` view. **Monaco
  replaces the `<pre>` in P2.**
- `WorkbenchShell.tsx` — wraps the grid in `WorkbenchProvider`; **File → Open
  Folder** uses the dialog plugin → `workspace_open`; status bar shows the
  workspace name.

The Agent Panel now **auto-fills its working folder (`cwd`) from the opened
project root** (resolves the P0.1 follow-up), so `ollama launch` runs in the
project by default.

### What happens to `nest-agent` / MCP / native tools

| Component | v1 decision |
| --- | --- |
| `nest-agent` loop in GUI | **Replace** with external agent host |
| `kiwi agent` CLI | Keep temporarily; mark deprecated; redirect docs to `ollama launch` |
| `nest-mcp` + Python memory servers | **Keep** — Cursor and external agents can share `.cursor/mcp.json` |
| Native file tools (`search_code`, `cargo_check`, …) | **Freeze** — no new tools; optional CLI-only fallback |
| Auto context injection (`context.rs`) | **Revisit** — external agents have their own context; Kiwi may pass workspace root only |
| Agent sidebar MCP toggles | Replace with **runtime picker** (OpenCode / Claude / Codex) + model + Launch button |

Nest crates remain in the monorepo for other hosts and tests; Kiwi stops being the primary driver of agent feature work.

### Spike checklist (manual QA)

Prerequisites: Ollama ≥ 0.15, `qwen3.5` pulled, context ≥ 64k, Kiwi project open in terminal.

1. `ollama launch opencode --model qwen3.5` — edit a file, run `cargo check`
2. `ollama launch claude --model qwen3.5` — same task
3. `ollama launch codex --model qwen3.5` — same task
4. Note: cold-start latency, tool reliability, memory use, whether MCP config is picked up
5. Document winner + embed pattern in `docs/agent/` (update after spike)

### Suggested `[agent]` config (v2)

```toml
[agent]
ollama_host = "192.168.88.10"
ollama_port = 11434
model = "qwen3.5"
runtime = "opencode"   # opencode | claude | codex | openclaw | hermes
context_length = 65536
# Optional: path to opencode.json / mcp — agent-specific
```

---

## Part C — Repo layout (target)

```text
apps/kiwi/
├── desktop/
│   ├── crates/kiwi/          # shared Rust: project, github, file, cli (no egui workbench)
│   └── config.toml
├── src-tauri/                # NEW — Tauri shell
├── ui/                       # NEW — React workbench
├── docs/
│   ├── plan/kiwi-tauri-v1.md # this file
│   └── agent/                # spike results
└── build                     # desktop → Tauri; keep test/cli targets
```

---

## Risks

| Risk | Mitigation |
| --- | --- |
| qwen3.5 too heavy for current GPU | Try `qwen3.5:4b` / cloud; document minimum hardware |
| External agent UX is terminal-only | Pattern A acceptable for v1; polish AI panel later |
| Monaco + large repo perf | Virtualize explorer; lazy tab loading |
| Long egui / Tauri parallel maintenance | Time-box egui; cut over after P4 if agent deferred |
| Agent CLIs not embeddable on Windows | Test all platforms in spike; document constraints |

---

## Implementation order (recommended)

1. **Agent spike** (1–2 days) — validate `qwen3.5` + `ollama launch` before large UI port
2. **P0–P2** — Tauri scaffold + explorer + editor (usable IDE shell)
3. **P3–P4** — SCM, issues, terminal
4. **P6** — Wire agent launch from AI panel / terminal
5. **P7** — Retire egui desktop

---

## Open questions

1. **Primary agent runtime** — OpenCode (open) vs Claude Code (UX) vs Codex?
2. **Chat parity** — Is embedded terminal enough, or must transcript live in right panel?
3. **MCP in external agents** — Does `ollama launch` pass Nest memory servers, or only Ollama tools?
4. **Kiwi repo** — Commit Tauri scaffold on `dev` branch first (like airtable-sync)?

---

## Related files (egui reference)

| Area | Path |
| --- | --- |
| Workbench shell | `desktop/crates/kiwi/src/workbench/mod.rs` |
| Activity bar | `workbench/activity.rs` |
| Sidebar dispatch | `workbench/sidebar/mod.rs` |
| Bottom panel | `workbench/bottom_panel/mod.rs` |
| Agent settings | `agent/mod.rs`, `workbench/sidebar/agent.rs` |
| Chat / agent loop | `chat.rs` |
| Main entry | `desktop/crates/kiwi/src/main.rs` |
