# Kiwi Explorer v1 Implementation Plan

## Status: Complete (Phases 0–5)

| Phase | Scope | Status |
| --- | --- | --- |
| 0 | Project root resolution | **Done (v0.1)** |
| 1 | `nest-file` wiring | **Done (v0.1)** |
| 2 | Explorer tree model | **Done (v0.1)** |
| 3 | Explorer UI | **Done (v0.1)** |
| 4 | Editor integration | **Done (v0.1)** |
| 5 | Polish | **Done (v0.1)** |

Enable the **Explorer** sidebar to show the real filesystem tree for the open project
folder, and open files in the editor when clicked.

## Goal

Replace the hard-coded placeholder in `workbench/sidebar/explorer.rs` with a lazy-loaded
directory tree rooted at the user's project folder (e.g. the Nest monorepo at
`/data/projects/nest` or the Kiwi app repo when opened standalone).

## Kiwi paths

| Item | Path |
| --- | --- |
| Desktop app | `apps/kiwi/desktop/` |
| Config | `apps/kiwi/desktop/config.toml` |
| Explorer UI | `crates/kiwi/src/workbench/sidebar/explorer.rs` |
| Editor tabs | `crates/kiwi/src/workbench/editor.rs` |
| Workbench state | `crates/kiwi/src/workbench/state.rs` |
| This plan | `apps/kiwi/docs/explorer-v1.md` |

## Current state

| Component | Today |
| --- | --- |
| `explorer.rs` | Static monospace labels (`▸ src`, `Cargo.toml`, …) |
| `EditorState` | Demo tabs (`src/main.rs`, …); placeholder content |
| `WorkbenchState.project` | Display string only (`"kiwi"`) — not a path |
| `FileModule` / `nest-file` | Not wired in Kiwi |
| Title bar | Shows `Project: kiwi` — no folder picker |

## Architecture

```text
config / CLI / auto-detect
        ↓
  ProjectRoot (resolved PathBuf)
        ↓
  FileModule::scoped(root)  →  FileService
        ↓
  ExplorerState (lazy tree, expand/collapse, ignore rules)
        ↓
  explorer.rs UI  ──click file──→  EditorState (open tab + read content)
```

**Dependency rule:** Explorer logic stays in the Kiwi app crate. Use [`nest-file`](../../../docs/nest-file/README.md)
for all filesystem I/O — do not call `std::fs` directly from UI code.

## Project root resolution

The explorer needs a single canonical **project root** directory. Resolution order
(highest wins):

| Priority | Source | Example |
| --- | --- | --- |
| 1 | CLI flag | `kiwi --project-root /data/projects/nest` |
| 2 | Config file | `[project] root = "../../.."` (relative to config.toml) |
| 3 | Environment | `KIWI_PROJECT_ROOT=/data/projects/nest` |
| 4 | Auto-detect | Walk parents from `std::env::current_dir()` for `.git` |
| 5 | Fallback | Current working directory |

### Suggested config (Nest monorepo dev)

When `./build run` is invoked from `apps/kiwi/desktop/`, the Nest repo root is three
levels up:

```toml
[project]
# Relative to config.toml (apps/kiwi/desktop/config.toml)
root = "../../.."
name = "nest"
```

When Kiwi is opened as a standalone repo, use:

```toml
[project]
root = ".."
name = "kiwi"
```

### Auto-detect rules (v1)

1. Start at process CWD.
2. Walk up until a directory contains `.git` **or** a `Cargo.toml` with `[workspace]`.
3. Use that directory as root.
4. If none found, use CWD and show a non-fatal warning in the explorer header.

## Phases

| Phase | Scope | Deliverable |
| --- | --- | --- |
| 0 | Project root resolution | `[project]` config + title bar shows real name |
| 1 | `nest-file` wiring | `FileModule::scoped(root)` in `modules.rs` |
| 2 | Explorer tree model | `ExplorerState` with lazy children + ignore rules |
| 3 | Explorer UI | Expand/collapse tree in sidebar |
| 4 | Editor integration | Click file → open tab + load content |
| 5 | Polish | Refresh, errors, large-repo perf |

---

## Phase 0 — Project root

**Objective:** Resolve and persist the workspace folder before building the tree.

### 0.1 Config section

```toml
[project]
root = "../../.."
name = "nest"
```

Add `ProjectSection` in `crates/kiwi/src/project/mod.rs`:

```rust
pub struct ProjectConfig {
    pub root: PathBuf,
    pub name: String,
}

impl ProjectConfig {
    pub fn from_config_service(service: &ConfigService) -> NestResult<Self>;
    pub fn resolve_root(config_path: Option<&Path>, raw: &str) -> PathBuf;
}
```

### 0.2 CLI / env overrides

Extend `GuiApp` / `CliApp` startup (or Kiwi wrapper) with optional `--project-root`.
Read `KIWI_PROJECT_ROOT` when flag and config are absent.

### 0.3 Workbench integration

- Replace `WorkbenchState.project: String` with `ProjectConfig` (or embed root + name).
- Title bar: `Project: nest` using `name`; tooltip shows full `root` path.
- Explorer header: show truncated root path + folder icon.

**Done when:** Title bar and explorer header reflect the resolved Nest repo path after
`./build run`.

---

## Phase 1 — `nest-file` module

**Objective:** Register scoped file I/O for the project root.

### 1.1 Dependencies

Add to `apps/kiwi/desktop/Cargo.toml` (workspace + patch, same pattern as agent MCP):

```toml
nest-file = { workspace = true }
```

### 1.2 Module wiring

```rust
// crates/kiwi/src/modules.rs
pub fn with_gui_modules(app: GuiApp, project_root: PathBuf) -> GuiApp {
    app.module(HttpClientModule::default())
        .module(OllamaModule::new())
        .module(FileModule::scoped(project_root))
}
```

Resolve `project_root` once at startup in `main.rs` before building the app.

### 1.3 Service usage

Explorer and editor obtain `FileService` via `app_ctx.service::<FileService>()?`.

Use:

- `list_dir(path)` — directory entries
- `read_text(path)` — file content for editor
- `metadata(path)` — distinguish files vs directories

Large reads: wrap in `TaskRuntime::spawn_blocking` (nest-file is sync-only).

**Done when:** `cargo test` for project root resolution; manual check that
`FileService` lists the Nest repo root.

---

## Phase 2 — Explorer tree model

**Objective:** In-memory tree with lazy loading and ignore rules.

### 2.1 State types

New module `crates/kiwi/src/workbench/explorer/mod.rs`:

```rust
pub struct ExplorerState {
    pub root: PathBuf,
    pub root_label: String,
    pub tree: TreeNode,
    pub error: Option<String>,
}

pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub kind: NodeKind,
    pub expanded: bool,
    pub children: Vec<TreeNode>,       // empty until expanded
    pub children_loaded: bool,
}

pub enum NodeKind { File, Directory }
```

Store `ExplorerState` on `WorkbenchState` (not in egui widget state).

### 2.2 Lazy loading

- Root node created at startup from `ProjectConfig.root`.
- On expand: if `!children_loaded`, call `FileService::list_dir`, sort entries
  (dirs first, then files, case-insensitive), apply ignore filter, attach children.
- On collapse: keep children in memory (v1) — optional prune in Phase 5.

### 2.3 Ignore rules (v1)

Skip entries where the file name matches:

| Pattern | Reason |
| --- | --- |
| `.git` | VCS metadata |
| `target` | Rust build output |
| `node_modules` | JS deps |
| `.venv` | Python venv |
| `dist`, `build` | Build artifacts |

Config extension (v1.1):

```toml
[project]
ignore = ["target", "node_modules", ".git"]
```

### 2.4 Relative display paths

Show paths relative to project root in the editor tab bar
(e.g. `core/crates/nest-file/src/lib.rs`).

**Done when:** Unit tests for sorting, ignore filter, and lazy child loading with a
temp directory fixture.

---

## Phase 3 — Explorer UI

**Objective:** Replace placeholder with an interactive tree.

### 3.1 Rendering (`sidebar/explorer.rs`)

- Scroll area with `id_salt("kiwi-explorer-tree")`.
- Each row: indent by depth, `▸`/`▾` chevron for directories, file icon for files.
- Use `nest-icon` (`Icon::FOLDER`, `Icon::FOLDER_OPEN`, file glyph or monospace icon).
- Click chevron / folder row → toggle expand (load children on first expand).
- Click file row → select (Phase 4 opens editor).
- Selected file: accent background (reuse `PALETTE` from theme).

### 3.2 Header actions

| Control | Action |
| --- | --- |
| Refresh | Reload expanded nodes (clear `children_loaded`, re-list) |
| Collapse all | Reset expansion state |

### 3.3 Error display

If `list_dir` fails (permissions, deleted folder), show inline error on the node and
set `ExplorerState.error` for the panel header.

**Done when:** Explorer shows the real Nest repo tree; expanding `core/crates` lists
crate folders.

---

## Phase 4 — Editor integration

**Objective:** Opening a file from the explorer loads it in the center editor.

### 4.1 Open file flow

```rust
pub fn open_file(editor: &mut EditorState, project_root: &Path, path: &Path, content: String);
```

- Tab label: path relative to project root.
- If tab already open for that path, focus existing tab.
- Store full path in `EditorState` (extend tab type — see below).

### 4.2 Editor state change

Replace `tabs: Vec<String>` with:

```rust
pub struct EditorTab {
    pub rel_path: String,
    pub abs_path: PathBuf,
    pub content: String,
    pub dirty: bool,
}
```

Render `content` in a monospace `TextEdit` or read-only `Label` (v1 read-only is fine;
editable buffer is v1.1).

### 4.3 Async read

On file click:

1. Show tab immediately with `"Loading…"`.
2. `spawn_blocking` → `FileService::read_text`.
3. On completion, replace content; on error, show message in tab.

**Done when:** Clicking `apps/kiwi/desktop/crates/kiwi/src/main.rs` opens the real file
content in the editor.

---

## Phase 5 — Polish

| Item | Priority |
| --- | --- |
| `notify` / file watcher → refresh changed nodes | Medium |
| Configurable `[project].ignore` | Medium |
| Open folder dialog (native or text field) | Low |
| Multi-root workspaces | Low |
| Symlink following policy (align with `nest-file` scoped mode) | Low |
| Source Control sidebar reads same root | Future |

---

## Testing strategy

| Layer | Approach |
| --- | --- |
| `ProjectConfig` | Unit tests: relative path resolution, env override, auto-detect with temp dirs |
| `ExplorerState` | Unit tests: ignore filter, sort order, lazy load |
| `nest-file` | Existing crate tests; Kiwi uses scoped fixture root |
| Kiwi GUI | Manual QA checklist (below) |
| Integration | Optional `cargo test -p kiwi` with temp project tree |

### Manual QA checklist

1. Set `[project] root = "../../.."` in `config.toml`.
2. `./build run` — Explorer shows Nest repo top-level (`apps`, `core`, `docs`, …).
3. Expand `core/crates/nest-file` — see `src/`, `Cargo.toml`.
4. Click `src/lib.rs` — editor tab opens with real source.
5. `target/` and `.git/` are hidden.
6. Refresh reloads after creating a file on disk.
7. Standalone Kiwi repo layout works with `root = ".."`.

---

## File touch list (implementation)

| File | Change |
| --- | --- |
| `config.toml` | Add `[project]` section |
| `Cargo.toml` (workspace + kiwi crate) | Add `nest-file` dep + patch |
| `src/project/mod.rs` | **New** — root resolution |
| `src/modules.rs` | Register `FileModule::scoped` |
| `src/main.rs` | Resolve project root at startup |
| `src/workbench/state.rs` | `ExplorerState`, `ProjectConfig` |
| `src/workbench/explorer/mod.rs` | **New** — tree model |
| `src/workbench/sidebar/explorer.rs` | Real tree UI |
| `src/workbench/sidebar/mod.rs` | Pass explorer state |
| `src/workbench/editor.rs` | Tabs with paths + content |
| `src/workbench/mod.rs` | Wire open-file from explorer |

---

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Large repos (`target` not ignored) | Default ignore list; lazy load only expanded dirs |
| Sync I/O blocks UI | `spawn_blocking` for list/read; show loading state |
| Wrong root when CWD varies | Config `[project].root` + auto-detect; document in README |
| Path traversal | Always use scoped `FileService` — never raw absolute paths outside root |
| Kiwi git vs nest gitignore | Kiwi lives in `apps/kiwi/` inside nest; plan documents both layouts |

---

## Related

- [nest-file](../../../docs/nest-file/README.md)
- [nest-file v1 plan](../../../docs/plan/nest-file-v1.md)
- [nest-gui workbench](../../../docs/nest-gui/README.md)
- [Kiwi agent MCP plan](./agent-mcp-v1.md)
