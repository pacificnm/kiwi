# kiwi_core Code Analysis

**Date:** 2026-06-27
**Scope:** `crates/kiwi_core/src/` — shared-logic crate consumed by `kiwi` (TUI) and `kiwi_gui` (GUI)
**Method:** Static read-only analysis of all source modules

---

## Executive Summary

`kiwi_core` is a well-structured shared library following an Elm-style `AppEvent → reduce() → Vec<SideEffect>` architecture. The separation of domain logic from UI is sound, and the codebase has good test coverage for most modules. However, the analysis surfaced two correctness bugs that are actively misfiring (ANSI splitting in the terminal emulator, and a blocking I/O syscall on the reducer thread), one latent panic (integer-log of zero), and a cluster of performance problems concentrated in hot render paths. Code quality issues are mostly localized duplication and a god-module reducer that is difficult to navigate.

Fifteen distinct findings are documented below, with severity and priority estimates.

---

## 1. Panic Risks

| # | File:Line | Description | Severity | Recommendation |
|---|-----------|-------------|----------|----------------|
| P1 | `reducer/mod.rs:2287` | `value.ilog10()` called inside `max_lineno_width()` on every `u32` line number from git2. `u32::ilog10(0)` panics in stable Rust; git2 can return line number 0 for binary/synthetic hunks. | **HIGH** | Guard with `value.checked_ilog10().unwrap_or(0) as usize + 1`, or filter out zero before the iterator enters this function. |
| P2 | `agent/manager.rs:154,163` | `active_session()` and `active_session_mut()` both call `.expect("active agent id must exist")`. The invariant is structurally maintained, but any future code path that sets `active_agent` without inserting into `self.agents` will panic with no recovery opportunity. | **MED** | Return `Option<&ManagedAgentSession>` or `Result` from both accessors; let callers handle the absent case gracefully. |
| P3 | `file_tree/state.rs:130` (test) | `state.nodes.get_mut(&root.join("src")).expect("src")` in `apply_fs_invalidation_reloads_expanded_parent` — not production code, but a test panic is opaque to CI. | **LOW** | Use `unwrap_or_else(|| panic!("src node missing after apply_children_loaded"))` with a descriptive message, or `assert!(...)` with a message. |

---

## 2. Error Handling Gaps

### 2.1 `eprintln!` in TUI context (`workspace/persistence.rs`)

Seven `eprintln!` calls are used for all workspace I/O error reporting (lines 31, 51, 65, 74, 94, 121, 123). In a TUI application running under crossterm raw mode, writing to stderr corrupts the terminal display: the raw-mode escape sequences do not render correctly and the user sees garbled output. The calls inside `try_save_from_reduce_view` (line 94) and `try_merge_save_gui` (line 121) are particularly frequent since they fire on every auto-save cycle.

**Recommendation:** Propagate errors from the `save_*` functions through the `SideEffect` system instead (e.g., a `SideEffect::WorkspaceSaveError(String)` that the frontend logs to its own overlay or status bar). The `load_*` path is less critical since it runs before TUI raw mode is active, but replacing `eprintln!` with `tracing::warn!` is lower risk.

### 2.2 Agent output reader errors silently discarded (`agent/io.rs`)

The PTY reader loop in `AgentOutputReader` returns `Ok(())` on all I/O errors after the session exits. Legitimate mid-session read errors (e.g., OS resource exhaustion) are indistinguishable from clean session shutdown and are never surfaced to the user or logged.

**Recommendation:** Distinguish between `ErrorKind::BrokenPipe`/`ConnectionReset` (normal PTY close) and unexpected errors; surface unexpected errors via an `AgentExited` event with a non-zero code.

### 2.3 git2 path-less entries silently skipped (`git/repository.rs`)

`map_git2_status` calls `entry.path()` and early-returns `None` when the path is absent. git2 can return path-less entries for submodule conflicts or binary renames. The entries are silently dropped from the file list, giving the user no indication that the status view is incomplete.

**Recommendation:** Log a warning (via `tracing::warn!`) when a path-less entry is encountered and include a placeholder entry in the UI.

### 2.4 GitHub number type mismatch loses type-system protection

`GitHubState` stores `selected_issue: Option<u64>` and `selected_pr: Option<u64>`, but the GitHub API uses `u32` for issue/PR numbers. This forces constant conversions (`u64::from(number)` when storing, `u32::try_from(number).ok()` when reading) throughout the codebase. The `try_from().ok()` silently returns `None` for a value >4 billion, hiding a type mismatch behind a silent Option unwrap.

**Recommendation:** Change `selected_issue` and `selected_pr` to `Option<u32>` to match the rest of the type system and eliminate the conversion noise.

---

## 3. Logic / Correctness Issues

### 3.1 ANSI escape detection broken in `split_at_visible` (`shell/scrollback.rs:524`) — **HIGH**

```rust
// split_at_visible, line 524
let mut chars = line.char_indices().peekable();
while let Some((idx, ch)) = chars.next() {
    if ch == '\x1b' {
        if chars.next_if_eq(&(0, '[')).is_some() {  // BUG: always false
```

`char_indices()` yields `(byte_index: usize, char)` tuples. `next_if_eq(&(0, '['))` compares the next tuple against `(0usize, '[')`. Since the `[` character following `\x1b` appears at byte offset ≥ 1 in the string (never at byte 0), this predicate is always `false`. The ANSI escape body is never consumed, so the escape characters are counted as visible columns. This causes incorrect column tracking when the terminal emulator performs mid-line overwrites on ANSI-colored output.

Compare with `truncate_ansi_line` on line 565, which correctly uses `line.chars().peekable()` (not `char_indices()`) and checks `chars.next_if_eq(&'[')`.

**Recommendation:** Change `chars` in `split_at_visible` from `line.char_indices().peekable()` to `line.chars().peekable()` and track the byte offset manually (or mirror the approach in `truncate_ansi_line`). Add a unit test with an ANSI-colored string and assert that `split_at_visible(colored_str, 3).0` has the correct visible length.

### 3.2 Blocking filesystem syscall on reducer thread (`file_tree/state.rs:206`)

```rust
pub fn invalidate_children(&mut self, path: &Path) {
    if let Some(child_paths) = self.children.remove(path) {
        for child in child_paths {
            if child.is_dir() {          // BUG: std::path::Path::is_dir() — syscall
                self.invalidate_children(&child);
            }
```

`std::path::Path::is_dir()` is a synchronous `stat(2)` syscall. `invalidate_children` is called from `FileTreeState::apply_fs_invalidation`, which is called from the reducer — the application's single synchronous event-processing thread. On a large repository or slow filesystem (network mounts, FUSE), this blocks the entire event loop.

The information is already available in the tree: `self.nodes` stores a `FileNode` with an `is_dir` field loaded during the original directory scan.

**Recommendation:** Replace `child.is_dir()` with `self.nodes.get(&child).is_some_and(|n| n.is_dir)`. This reads from the in-memory map without touching the filesystem.

### 3.3 Redundant `left_pane` assignment in `reduce_github_open_selected`

Inside the `Issues` match arm of `reduce_github_open_selected`, the code sets `state.github.left_pane = GitHubLeftPane::Issues`. This is a no-op: the arm only executes when `left_pane` is already `Issues`. The assignment is harmless but misleading, suggesting the arm also handles other pane states.

**Recommendation:** Remove the redundant assignment.

### 3.4 `is_wt_new` double-check in `map_git2_status` (`git/repository.rs`)

The function checks `is_wt_new() && !is_index_new()` to classify `Untracked` files, then at the end of the match also checks plain `is_wt_new()` again for `Untracked`. The second branch can never fire: any file matching plain `is_wt_new()` would have been caught by the first branch (whether or not `is_index_new()` is also set).

**Recommendation:** Remove the trailing `is_wt_new()` branch, or combine both cases into `is_wt_new()` if the `!is_index_new()` guard was unintentional.

### 3.5 `reduce_github_issues_loaded` triggers PR detail effects

When issues finish loading, `reduce_github_issues_loaded` checks whether `navigation.main_tab == MainTab::Prs` and, if so, calls `github_pr_detail_effects`. This is semantically confusing: an *issues-loaded* handler should not branch on whether the user is looking at PRs. It also means a stale GitHub response can cause unexpected PR detail fetches.

**Recommendation:** Move the cross-tab eager-loading logic into the navigation reducer, triggered when `MainTab::Prs` becomes active, rather than piggy-backing on the issues-loaded handler.

---

## 4. Code Quality & Maintainability

### 4.1 Duplicated `scroll_by_lines` implementation

`AgentState::scroll_by_lines` (`state/domains.rs:317–340`) and `ShellState::scroll_by_lines` (`state/domains.rs:412–435`) are byte-for-byte identical. Both delegate to `self.scrollback.scroll_by_lines(delta)` with identical clamping logic.

**Recommendation:** Extract a shared `scroll_by_lines(scrollback: &mut ScrollbackBuffer, delta: i32)` free function, or move the implementation onto `ScrollbackBuffer` directly.

### 4.2 Duplicated `scroll_offset_for_row` function

`git/panel.rs:108–124` and `github/selection.rs:150–166` contain identical implementations of `scroll_offset_for_row(selected_row, scroll_offset, viewport_rows)`. Neither is `pub`, so neither can be imported by the other.

**Recommendation:** Move `scroll_offset_for_row` to a `crate::ui` or `crate::selection` utility module and import it in both files.

### 4.3 Duplicated `coalesce_paths` function

`events/channel.rs:98–104` and `watcher/debounce.rs:50–56` both define a private `coalesce_paths(paths: Vec<PathBuf>) -> Vec<PathBuf>` that deduplicates a path list using a `HashSet`. Neither is accessible from the other module.

**Recommendation:** Move to a shared internal utility module (e.g., `crate::util::paths`).

### 4.4 `github_issue_list_access_effects` is a pointless wrapper

```rust
pub fn github_issue_list_access_effects(state: &mut ReduceView<'_>, force: bool) -> Vec<SideEffect> {
    github_issue_list_effects(state, force)
}
```

This exported function only calls a private function with the same signature. It adds an indirection with no behavior change.

**Recommendation:** Delete it and call `github_issue_list_effects` directly, or make `github_issue_list_effects` the `pub` function.

### 4.5 SRP violation in `github/context_menu.rs`

The file mixes four unrelated concerns:
- Context menu state types (`GhContextMenuState`, `GhContextTarget`, `GhContextMenuAction`)
- Context menu behavior (`move_cursor`, `selected_action`, `menu_width`)
- GUI-specific action arrays (`GUI_ISSUE_LIST_ACTIONS`, `GUI_PR_LIST_ACTIONS`)
- Agent prompt formatting (`format_issue_agent_prompt`, `format_pr_agent_prompt`, `issue_body_excerpt_from_detail`, `truncate_excerpt`)

The agent prompt formatting has no logical connection to context menu state.

**Recommendation:** Move `format_*_agent_prompt`, `issue_body_excerpt_from_detail`, and `truncate_excerpt` to `github/agent_prompts.rs`. Move the GUI constant arrays to a `gui_actions.rs` or keep them in the GUI crate.

### 4.6 Fragile sentinel-string parsing in `issue_body_excerpt_from_detail`

`issue_body_excerpt_from_detail` locates the issue body by searching `detail.display_lines` for the literal string `"— Body —"` and then stops at `"— Comments"`. These are formatting sentinels embedded in the pre-rendered display strings. If the display format changes (e.g., dash style, trailing space), the function silently returns `None`.

**Recommendation:** Store structured data (body text, comments list) separately from the display-formatted strings, and compute the excerpt from the structured data.

### 4.7 Duplicate keyboard shortcut `"8"` in command registry (`commands/registry.rs`)

The shortcut `"8"` is registered for both `"main.settings"` and `"settings.open"`. Both commands navigate to `MainTab::Settings`, making one of them dead code. The duplicate also means the palette will show two entries for the same keystroke.

**Recommendation:** Remove `"settings.open"` or reassign it to a different shortcut.

### 4.8 Missing `#[non_exhaustive]` on public enums

`AppEvent`, `AppCommand`, and `SideEffect` are large public enums exported from `kiwi_core`. Any variant addition is a breaking change for downstream crates matching exhaustively. Currently `kiwi` and `kiwi_gui` use `_` catch-all arms, but this is a soft convention rather than an enforced contract.

**Recommendation:** Annotate all three with `#[non_exhaustive]` to make the contract explicit and prevent future crates from writing exhaustive matches.

### 4.9 `ReduceView<'_>` provides zero field-level encapsulation

`ReduceView<'_>` exposes 23 public mutable reference fields. Any handler function in the 5,321-line reducer module can modify any field at any time. There is no way to audit which handler functions actually need access to which state fields, making it difficult to reason about invariants.

**Recommendation:** This is a larger refactor, but as a first step, group the 23 fields into a smaller number of named domain views (e.g., `github_view`, `shell_view`, `git_view`) and pass only the relevant view to each handler.

---

## 5. Performance

### 5.1 `lines_for_display()` clones entire scrollback buffer on every call (`shell/scrollback.rs:141–155`)

```rust
fn lines_for_display(&self, include_pending: bool) -> Vec<String> {
    let mut lines = self.history.clone();   // up to 10,000 String clones
    lines.extend(self.screen.clone());       // up to 256 String clones
    ...
}
```

`lines_for_display` is the only internal accessor for the combined history+screen view. It is called by `line_count()`, `viewport_lines()`, `recent_text()`, and `cursor_display_position()` — all of which are called on every frame render. This means every frame allocates and copies up to 10,256 heap strings, regardless of whether the scrollback content changed.

**Recommendation:** Add a `dirty` flag to `ScrollbackBuffer`. When `dirty` is false, return a cached `Vec<String>` reference. Alternatively, change `viewport_lines()` to return a slice or iterator view over `history` and `screen` without allocating, avoiding the combine-and-copy entirely.

### 5.2 `visible_rows()` traverses the full tree multiple times per event (`file_tree/state.rs`)

`visible_rows()` walks the entire expanded subtree from the root on every call. It is called by `move_selection`, `ensure_selection`, and `clamp_selection`. Many reducer handlers call `ensure_selection` followed by `move_selection`, which calls `visible_rows()` twice (once inside `ensure_selection` and again inside `move_selection`).

**Recommendation:** Cache `visible_rows()` output in a `Vec<PathBuf>` field on `FileTreeState`, invalidated whenever the tree structure changes (`expand`, `collapse`, `apply_children_loaded`, `apply_fs_invalidation`). Mark it dirty on those operations and recompute lazily.

### 5.3 `build_panel_rows()` rebuilt on every selection call (`git/selection.rs`)

Six git selection functions — `clamp_git_scroll`, `ensure_git_selection`, `git_move_selection`, `git_select_row`, `git_row_at_viewport`, `git_selected_row_index` — all call `build_panel_rows()` independently. A single `GitMoveSelection` event that calls `ensure_git_selection` then `git_move_selection` rebuilds the panel rows twice.

**Recommendation:** Cache the panel rows in `GitState`, invalidated only when `file_entries` or `diff` source changes. Pass the cached value to selection functions rather than rebuilding inside each.

### 5.4 `visible_width()` allocates a `String` on every call (`ansi.rs`)

```rust
pub fn visible_width(s: &str) -> usize {
    strip_ansi(s).chars().count()   // allocates a String
}
```

`visible_width` is called in hot display-render paths (file tree column sizing, diff line width). It allocates a full `String` per call via `strip_ansi`.

**Recommendation:** Implement `visible_width` as a direct ANSI-skipping character iterator that counts without allocating, similar to the existing `truncate_ansi_line` logic.

### 5.5 `DiffState::scroll_horizontal` scans all diff lines on every call

`scroll_horizontal` calls `self.lines.iter().map(|line| line.content.chars().count()).max()` to compute the maximum line width before clamping the horizontal offset. This is O(n lines) per scroll event.

**Recommendation:** Cache `max_line_width: usize` in `DiffState`, updated only when `self.lines` is replaced (on diff load). Clamp against the cached value in `scroll_horizontal`.

---

## 6. Architecture Observations

### 6.1 `reducer/mod.rs` is a 5,321-line god module

All ~100 private handler functions live in a single file alongside the top-level `reduce()` dispatch. Navigation, git, GitHub, shell, agent, search, diff, settings, and workspace persistence are all interleaved. The file size makes it difficult to find handlers, understand the blast radius of changes, and write focused tests.

**Recommendation:** Refactor into a `reducer/` directory of submodules, one per domain (e.g., `reducer/github.rs`, `reducer/git.rs`, `reducer/shell.rs`). The top-level `reducer/mod.rs` becomes a thin dispatcher that delegates to each submodule. This is a large mechanical change but has no behavior impact and is the most impactful quality improvement available.

### 6.2 `TerminalResize` event is a no-op in the core reducer

`AppEvent::TerminalResize { width, height }` is defined and dispatched by frontends but the reducer does not act on it. Shell and agent resize operations are instead triggered by `ResizeShell` side effects emitted from the frontend. The event variant occupies space in the public API without doing anything.

**Recommendation:** Either remove `TerminalResize` from `AppEvent` and document that terminal resize is handled exclusively via `SideEffect::ResizeShell`, or add a handler that updates a `terminal_size` field on `AppState` so layout-dependent reducer logic can use it.

### 6.3 Workspace snapshot uses string-typed tab names with silent fallback

`WorkspaceSnapshot` serializes `main_tab` and `left_nav_tab` as free-form JSON strings. On deserialization, unknown or misspelled tab names silently fall back to defaults. This means a workspace file written by a future version of kiwi (or by a typo) will silently load with the wrong tab selected, giving no indication that the persisted state was ignored.

**Recommendation:** Use `#[serde(deny_unknown_fields)]` or `serde(default)` with an explicit error logged when a tab name is unrecognized, so users know their workspace state was not restored.

### 6.4 `SideEffect`, `AppEvent`, and `AppCommand` have unbounded API surface

All three enums have 30+ variants and no `#[non_exhaustive]` annotation (see Quality §4.8). They are the primary public API of `kiwi_core`. As the application grows, this will continue to accumulate variants. The `SideEffect` enum in particular mixes IO operations of very different scopes (filesystem, network, clipboard, PTY) without any grouping.

**Recommendation:** Consider grouping `SideEffect` variants into sub-enums (e.g., `SideEffect::GitHub(GitHubEffect)`) to contain scope and reduce the size of exhaustive match arms in frontends. Apply `#[non_exhaustive]` immediately.

---

## 7. Priority Recommendations

| Rank | Finding | Location | Effort | Impact |
|------|---------|----------|--------|--------|
| 1 | Fix ANSI escape detection bug in `split_at_visible` — currently always skips ANSI handling, causing wrong column tracking in mid-line terminal overwrites | `shell/scrollback.rs:524` | Low (2–5 lines) | High — correctness bug visible in any ANSI-colored shell/agent output |
| 2 | Guard `ilog10(0)` panic in `max_lineno_width` | `reducer/mod.rs:2287` | Low (1 line) | High — silent crash on any diff containing a line number 0 |
| 3 | Replace `eprintln!` with structured error propagation in workspace persistence | `workspace/persistence.rs` | Low–Med | High — currently corrupts TUI display on every auto-save error |
| 4 | Fix blocking `child.is_dir()` syscall in `invalidate_children` | `file_tree/state.rs:206` | Low (1 line) | Med — blocks event loop on slow or network-mounted filesystems |
| 5 | Cache `lines_for_display()` in `ScrollbackBuffer` | `shell/scrollback.rs:141` | Med | High — eliminates O(10K) string clones per frame render for all shell/agent sessions |
| 6 | Cache `visible_rows()` in `FileTreeState` | `file_tree/state.rs` | Med | Med — eliminates redundant full-tree traversals per event |
| 7 | Fix `u64`/`u32` type mismatch in `GitHubState` | `state/domains.rs` | Low | Med — removes silent truncation risk and eliminates widespread conversion noise |
| 8 | Extract duplicated `scroll_offset_for_row` and `coalesce_paths` to shared utilities | `git/panel.rs`, `github/selection.rs`, `events/channel.rs`, `watcher/debounce.rs` | Low | Low — maintainability, no behavior change |
| 9 | Add `#[non_exhaustive]` to `AppEvent`, `AppCommand`, `SideEffect` | `events/mod.rs` | Low (3 annotations) | Med — prevents downstream crates from writing exhaustive matches on a fast-moving enum |
| 10 | Split `reducer/mod.rs` into per-domain submodules | `reducer/mod.rs` | High (mechanical) | High — the single biggest maintainability barrier in the codebase |
