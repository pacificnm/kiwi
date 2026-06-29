# kiwi_gui Crate Analysis

## Overview

`kiwi_gui` is the egui/eframe desktop GUI frontend for Kiwi. It sits atop `kiwi_core` for all
business logic and state, communicating through the same reducer+event system the TUI uses.
The crate owns: window bootstrapping, dock layout (egui_dock), panel rendering, PTY session
management, theme bridging, search debounce, and workspace persistence of the GUI-specific dock
state. It has no Cargo workspace dependencies other than `kiwi_core`, `egui`, `egui_dock`,
`eframe`, `serde`, `serde_json`, and `clap`.

---

## File Inventory

| File | Description |
|---|---|
| `src/main.rs` | Binary entry point; calls bootstrap, builds `GuiRuntime`, runs eframe |
| `src/app.rs` | `KiwiApp` — the `eframe::App` impl; top-level update loop, input routing, workspace save timer |
| `src/bootstrap.rs` | Pre-eframe startup: repo validation, config load, theme load |
| `src/cli.rs` | `Cli` struct (clap); exposes `--config`, `--theme`, positional path |
| `src/runtime.rs` | `GuiRuntime` — owns `AppState`, `EventChannel`, `PtyRuntime`, `SearchRuntime`; wraps dispatcher |
| `src/services.rs` | `execute_gui_effect` / `process_pending_events`; wires side-effects to background tasks |
| `src/navigation_bridge.rs` | Maps `NavigationState` ↔ `KiwiTab`; `sync_dock_from_navigation` |
| `src/theme/bridge.rs` | `GuiTheme`; maps `ThemePalette` semantic roles to `egui::Color32` / `Visuals` |
| `src/chrome/menu_bar.rs` | Top menu bar (File, View, Git, Help); reset-layout modal |
| `src/chrome/status_bar.rs` | Bottom status bar using `compute_status_bar` from core |
| `src/chrome/command_palette.rs` | Centered palette modal; keyboard routing while palette is open |
| `src/chrome/help_modal.rs` | Keyboard shortcuts and About modals |
| `src/dock/mod.rs` | `DockShell` — owns `DockState<KiwiTab>`; `ensure_tab`, `close_tab`, `focused_tab` |
| `src/dock/tab.rs` | `KiwiTab` enum; titles, `factory_tabs`, `all_variants`, `closable` |
| `src/dock/layout.rs` | `initial_dock_state()` — three-region factory layout |
| `src/dock/region.rs` | `DockRegion` enum; `find_leaf_for_region`, `push_tab_to_leaf`, `focus_tab` |
| `src/dock/actions.rs` | `TabActions` — low-level tab open/close/focus helpers |
| `src/dock/viewer.rs` | `KiwiTabViewer` implements `egui_dock::TabViewer`; routes to `render_panel` |
| `src/dock/context.rs` | `PanelContext` — shared inputs for panel render functions |
| `src/dock/persistence.rs` | `snapshot_from_dock` / `restore_dock`; serde_json round-trip for dock layout |
| `src/dock/panels/mod.rs` | `render_panel` dispatch table; re-exports keyboard functions |
| `src/dock/panels/terminal.rs` | Shell PTY panel; chrome header + `render_shell_panel` |
| `src/dock/panels/agent.rs` | Agent PTY panel; chrome header + `render_agent_panel` |
| `src/dock/panels/scrollback.rs` | Shared PTY scrollback renderer; `render_shell_panel`, `render_agent_panel` |
| `src/dock/panels/pty_input.rs` | PTY keyboard/paste/scroll input; `collect_pty_input`, key-to-bytes encoding |
| `src/dock/panels/explorer.rs` | File explorer panel; virtualized tree rows with git status badges |
| `src/dock/panels/git_status.rs` | Git status panel; virtualized file list |
| `src/dock/panels/git_diff.rs` | Unified diff panel; gutter + colorized lines |
| `src/dock/panels/preview.rs` | Read-only file preview; line numbers, truncation |
| `src/dock/panels/search.rs` | Search panel; query input bound directly to `AppState` |
| `src/dock/panels/search_input.rs` | Search keyboard and global-focus-shortcut handlers |
| `src/dock/panels/github_left.rs` | GitHub left tab: Issues/PRs hub + virtualized list |
| `src/dock/panels/github_prs.rs` | PRs detail panel |
| `src/dock/panels/issues_detail.rs` | Issues detail panel |
| `src/dock/panels/github_common.rs` | Shared GitHub helpers: auth gate, row commands, detail lines |
| `src/dock/panels/github_input.rs` | GitHub keyboard and navigation sync handlers |
| `src/dock/panels/github_context_menu.rs` | Right-click context menus for issue/PR list rows |
| `src/dock/panels/ansi.rs` | ANSI SGR → `egui::LayoutJob` parser for PTY scrollback |
| `src/dock/panels/layout.rs` | `render_virtual_rows`; `pty_dimensions_from_ui` |
| `src/dock/panels/placeholder.rs` | Stub panel for unimplemented tabs (Logs, Config, GitLog) |
| `src/pty/mod.rs` | `PtyRuntime` — manages shell and agent PTY sessions |
| `src/pty/agent_runtime.rs` | `AgentRuntime` — per-agent session/reader HashMap |
| `src/pty/resize.rs` | `effective_pty_size` — clamped PTY dimensions helper |

---

## Issues Found

### Critical (bugs / data loss / incorrect behavior)

---

**C-1: `SideEffect::CopyToClipboard`, `PasteFromClipboard`, and `PersistUserTheme` are silently dropped**

- File: `crates/kiwi_gui/src/services.rs`, line 318 (`_ => {}` at end of `execute_gui_effect`)
- The outer `SideEffect` match ends with `_ => {}`. Three variants from `kiwi_core::events::SideEffect` reach this arm:
  - `SideEffect::CopyToClipboard(String)` — reducer-dispatched clipboard copy is never executed in the GUI.
  - `SideEffect::PasteFromClipboard` — clipboard paste to PTY is silently lost.
  - `SideEffect::PersistUserTheme { name }` — theme changes are never persisted.
- These effects are handled correctly in the TUI, but the GUI silently drops them.
- Suggested fix: Handle `CopyToClipboard` and `PersistUserTheme` in `execute_gui_effect`. `PasteFromClipboard` requires access to `egui::Context` (see C-2 / A-1 below); add it to `ServiceContext` or route it differently.

---

**C-2: `SideEffect::SaveWorkspace` in `execute_gui_effect` only saves TUI/core state, not the GUI dock layout**

- File: `crates/kiwi_gui/src/services.rs`, lines 107–110
- `execute_gui_effect(SideEffect::SaveWorkspace)` calls only `try_save_from_reduce_view`, which saves the core/TUI workspace file. It does NOT call `try_merge_save_gui`, so the dock layout is never persisted when the reducer triggers a workspace save.
- In contrast, `KiwiApp::save_workspace` (app.rs:65–78) calls both functions.
- Result: any reducer-driven workspace save (e.g., after opening a file) loses the dock layout.
- Suggested fix: Call `try_merge_save_gui` in the `SaveWorkspace` arm, or route all workspace saves through `KiwiApp::save_workspace`.

---

**C-3: `render_command_palette` mutates `AppState` directly during rendering**

- File: `crates/kiwi_gui/src/chrome/command_palette.rs`, lines 97–102
- When the palette input changes, the render function directly writes `state.palette.history_cursor = None` and calls `refresh_matches(&mut view)` on a `ReduceView` it constructs inline, then calls `view.set_dirty()`. This bypasses the reducer and breaks unidirectional data flow.
- Commands that affect `palette.history_cursor` and `palette.matches` should be dispatched via `AppCommand` so the reducer owns the transition. As written, other reducers cannot observe or react to this state change.
- Suggested fix: Dispatch an `AppCommand::PaletteInputChanged` (or similar) command instead of mutating state directly. If that command doesn't exist, add it.

---

**C-4: `search.rs` `TextEdit` widget is bound directly to `&mut ctx.state.search.query`**

- File: `crates/kiwi_gui/src/dock/panels/search.rs`, lines 49–63
- The `TextEdit::singleline` widget writes to `ctx.state.search.query` on every keystroke. While a follow-up `dispatch(SearchSetQuery(...))` is fired on `response.changed()`, the `query` field is mutated at the widget level before the reducer runs. This means the query string briefly has a value that hasn't gone through the reducer, which is inconsistent with the rest of the architecture where state is only written by reducers.
- More concretely: if another part of the render loop reads `state.search.query` before the command is processed, it sees uncommitted state.
- Suggested fix: Use a local frame-level string buffer for the `TextEdit` binding, compare it to `state.search.query` post-render, and dispatch `SearchSetQuery` only on change without mutating `state.search.query` directly in the render pass.

---

**C-5: `close_window` is called from inside the `CentralPanel` closure; `update()` continues executing afterward**

- File: `crates/kiwi_gui/src/app.rs`, lines 312–347
- Inside the closure passed to `egui::CentralPanel::default().show(ctx, |ui| { ... })`, calling `self.close_window(ctx)` followed by `return` only exits the closure, not the `update()` method. Execution then continues at line 349 — the reset-layout modal, shortcuts modal, about modal, and command palette are all still rendered. `poll_workspace_save()` may also run, causing a second workspace save in the same frame as the first from `close_window`.
- The same issue exists for `handle_pty_input` at line 344: it calls `close_window` but has no `return`, so it falls through to the closing brace of the closure harmlessly — but this is inconsistent with the other three branches that do `return`.
- Suggested fix: Use a boolean `let mut should_close = false;` that is set inside the closure and checked after the `show()` call. Move `close_window` outside the closure.

---

**C-6: `MainTab::Branches` maps to `KiwiTab::GitHubIssues` (wrong tab)**

- File: `crates/kiwi_gui/src/navigation_bridge.rs`, line 99
- `kiwi_tab_for_main(MainTab::Branches, _gh_pane)` returns `Some(KiwiTab::GitHubIssues)`. `KiwiTab::GitLog` exists and is the natural target for a branches/log view. Opening the Issues tab when branches are navigated to is incorrect and confusing.
- Suggested fix: Return `Some(KiwiTab::GitLog)` or `None` if branches are not yet implemented.

---

**C-7: Double shutdown and double workspace save when quitting via Ctrl+Q**

- Files: `crates/kiwi_gui/src/app.rs` lines 258–262 (close_window), and lines 376–379 (on_exit)
- When the user quits via `AppCommand::Quit`, `close_window` is called: it calls `self.runtime.shutdown()` and `self.save_workspace()`. When eframe subsequently closes the window it calls `on_exit`, which also calls `self.runtime.shutdown()` and `self.save_workspace()`. So both are called twice.
- `PtyRuntime::shutdown()` has a `shut_down` guard that prevents double shutdown of PTYs, so it's safe. But `save_workspace` has no guard — it writes the workspace file a second time from `on_exit`. This is harmless in practice (the file is overwritten with the same content) but wasteful and could theoretically lose state if the second save has different data.
- Suggested fix: Add a guard flag in `KiwiApp` (e.g., `workspace_saved_on_exit`) that prevents `on_exit` from re-saving if `close_window` already ran.

---

**C-8: Unmatched `SideEffect` sub-variants silently dropped via `_ => {}`**

- File: `crates/kiwi_gui/src/services.rs`, lines 121, 220, 231, 260, 291, 316
- Five sub-enum arms (`GitEffect`, `ShellEffect`, `AgentEffect`, `FsEffect`, `SearchEffect`) each end with `_ => {}`. Any new variant added to `kiwi_core` for these enums will be silently ignored by the GUI service layer. There is no compile-time exhaustiveness warning because the catch-all suppresses it.
- For example, `ShellEffect` line 231 currently drops any variant other than `Write` and `Resize`. If a `ShellEffect::Signal` variant were added, the GUI would silently ignore it while the TUI handled it.
- Suggested fix: Remove `_ => {}` arms and use explicit `#[allow(unreachable_patterns)]` only for variants genuinely not applicable to the GUI, with a comment explaining why. This makes adding `kiwi_core` variants require an explicit GUI decision.

---

### Performance

---

**P-1: Entire diff and preview `lines` vec cloned every frame**

- Files: `crates/kiwi_gui/src/dock/panels/git_diff.rs`, line 97; `crates/kiwi_gui/src/dock/panels/preview.rs`, line 165
- `let lines = ctx.state.diff.lines.clone();` and `let lines = ctx.state.preview.lines.clone();` clone the entire content vector every render frame to satisfy the closure borrow in `render_virtual_rows`. For a 1000-line file this is ~1000 `String` clones at 60 fps.
- This is avoidable. `render_virtual_rows` takes a `FnMut(&mut Ui, usize)` callback — it only needs to index into the collection, not own it.
- Suggested fix: Change the closure to borrow `&ctx.state.diff.lines` or `&ctx.state.preview.lines` using a shared reference. No clone needed if the closure captures `&[String]` instead.

---

**P-2: `format_gutter` recomputes `max_lineno_digits` O(n) for every row — O(n²) total**

- File: `crates/kiwi_gui/src/dock/panels/git_diff.rs`, lines 228–237
- `format_gutter(old_lineno, new_lineno, lines)` calls `max_lineno_digits(lines.iter()...)` twice per invocation. It is called once per visible diff row inside `render_virtual_rows`. For `n` lines, total cost is O(n²) digit-width computation per frame.
- Suggested fix: Compute `old_width` and `new_width` once before the `render_virtual_rows` call in `render_diff_content`, and pass them as parameters to a closure or helper.

---

**P-3: `truncate_line` and `slice_line` allocate `Vec<char>` per line per frame**

- File: `crates/kiwi_gui/src/dock/panels/git_diff.rs`, lines 254–282
- Both functions collect chars into a `Vec<char>` to do indexed slicing. For a diff with hundreds of visible lines, this is hundreds of `Vec<char>` allocations per frame at 60 fps.
- Suggested fix: Use `str::char_indices` to walk the string without collecting. For `truncate_line`, iterate chars counting up to `max_width` and slice the original `&str` by byte offset. For `slice_line`, similarly skip `offset` chars by byte offset.

---

**P-4: `GuiTheme::apply_to_context` clones `Visuals` and mutates style every frame**

- File: `crates/kiwi_gui/src/theme/bridge.rs`, lines 44–64; called from `app.rs` line 282 every `update()` call
- `ctx.set_visuals(self.visuals.clone())` and `ctx.style_mut(...)` are called every frame. The `Visuals` clone alone involves copying a large struct with multiple sub-structs. egui ignores no-op style calls internally, but the clone still happens.
- Suggested fix: Call `apply_to_context` once during `KiwiApp::new`, and only again when the theme changes (e.g., after a `PersistUserTheme` effect). Store a dirty flag in `GuiTheme`.

---

**P-5: `scroll_row_from_clip` computed twice per frame in `render_virtual_rows`**

- File: `crates/kiwi_gui/src/dock/panels/layout.rs`, lines 64 and 71
- `visible_start` is computed at line 64 (`scroll_row_from_clip`), then `start` is computed by calling the same function again at line 71, adding `.min(max_start)`. The second call can reuse `visible_start`.
- Suggested fix: `let start = visible_start.min(max_start);` at line 71 instead of re-calling `scroll_row_from_clip`.

---

**P-6: `compute_status_bar` called every frame, allocating multiple `String` values**

- File: `crates/kiwi_gui/src/chrome/status_bar.rs`, line 18
- `compute_status_bar(state)` runs every render frame. Inspecting `kiwi_core::status_bar::compute_status_bar`, it builds a `StatusBarSnapshot` with several owned `String` fields (branch, root_name, git_label, etc.). This happens 60× per second even when nothing changes.
- Suggested fix: Cache `StatusBarSnapshot` in `KiwiApp` and only recompute when `state.dirty` is true or relevant state fields change.

---

### Architecture

---

**A-1: `SideEffect::PasteFromClipboard` cannot be properly implemented in `execute_gui_effect`**

- File: `crates/kiwi_gui/src/services.rs`
- `execute_gui_effect` takes a `&mut ServiceContext` which has no `egui::Context`. Implementing `PasteFromClipboard` requires calling `ctx.input(|i| i.raw.clone())` or `ctx.output_mut(|o| o.copied_text.clone())` to read the system clipboard — but neither is possible without `egui::Context`.
- This is a structural gap: the side-effect execution layer cannot reach the egui context. The PTY paste that `collect_pty_input` handles via `Event::Paste` is a different path and does work. But if the reducer emits `PasteFromClipboard` (e.g., from a command palette "Paste" action), there is no way to fulfil it.
- Suggested fix: Either add `&egui::Context` to `ServiceContext`, or handle clipboard effects in `app.rs` after `execute_gui_effects` returns, by checking which effects were returned by the reducer rather than delegating all dispatch to the service layer.

---

**A-2: `truncate_line` duplicated between `preview.rs` and `git_diff.rs`**

- Files: `crates/kiwi_gui/src/dock/panels/preview.rs`, lines 273–288; `crates/kiwi_gui/src/dock/panels/git_diff.rs`, lines 267–282
- Both files define a private `fn truncate_line(text: &str, max_width: usize) -> String` with identical logic (collect chars into Vec, truncate with `…`). Any bug fixed in one is not fixed in the other.
- Suggested fix: Move to `dock/panels/layout.rs` or a new `dock/panels/text_util.rs` and make pub(super).

---

**A-3: `PtyScrollbackView::footer` is assigned but never used by the renderer**

- File: `crates/kiwi_gui/src/dock/panels/scrollback.rs`, lines 18 and 29
- `PtyScrollbackView` has a `footer: Option<&'a str>` field. Line 29 reads `let _ = pane.footer;` — a deliberate no-op to suppress the unused-field warning. The callers (`render_agent_panel`) separately call `render_pty_footer` with their own footer value after the scrollback returns. The field on the struct is dead weight.
- Suggested fix: Remove `footer` from `PtyScrollbackView`. The `render_pty_footer` call-site pattern is clearer.

---

**A-4: `primary_tab_for_navigation` is production dead code**

- File: `crates/kiwi_gui/src/navigation_bridge.rs`, lines 70–81
- The function is marked `#[allow(dead_code)]` with a comment "navigation helper; covered by unit tests". It is never called in the running application.
- Suggested fix: Either make it used (e.g., expose it for the status bar or for computing dock hints) or delete it and its tests.

---

**A-5: `search_input::navigation_sync_commands` is production dead code**

- File: `crates/kiwi_gui/src/dock/panels/search_input.rs`, lines 10–25
- Marked `#[allow(dead_code)]` with a comment explaining that navigation is handled by `on_tab_button`. The function is never called outside tests.
- Suggested fix: Remove the function (and the `#[allow]` attribute). The tests can be restructured around the `global_search_focus_commands` path that is actually used.

---

**A-6: `GitHubEffect::SpawnRefresh` arm is a confusing intentional no-op**

- File: `crates/kiwi_gui/src/services.rs`, lines 124–127
- The comment says "Reducer refresh path emits SpawnAuthCheck; keep for parity with TUI." This implies the variant is expected to arrive but nothing should happen. However, it looks like an incomplete implementation to a reader. If `SpawnRefresh` always co-arrives with `SpawnAuthCheck`, the `SpawnRefresh` variant itself is unnecessary; if it can arrive alone, it's a bug that nothing happens.
- Suggested fix: Add a detailed comment explaining the exact reducer contract (i.e., that `GitHubRefresh` always produces `SpawnAuthCheck` after `SpawnRefresh`, so `SpawnRefresh` in the GUI is a no-op by design), or remove the dead arm entirely and verify the reducer contract in a test.

---

**A-7: Workspace persistence has two separate paths with different behaviors**

- Files: `crates/kiwi_gui/src/app.rs` lines 65–79 (`save_workspace`); `crates/kiwi_gui/src/services.rs` lines 107–110 (`SideEffect::SaveWorkspace`)
- `KiwiApp::save_workspace` saves both TUI state (`try_save_from_reduce_view`) and GUI dock layout (`try_merge_save_gui`). The service-layer `SideEffect::SaveWorkspace` handler only calls `try_save_from_reduce_view`. The 30-second polling timer goes through `save_workspace` (full save); reducer-triggered saves go through the service layer (partial save). This asymmetry is a latent data-loss bug (dock layout not persisted on reducer-triggered saves) and is confusing to maintain.
- Suggested fix: Either route all workspace saves through `KiwiApp::save_workspace` (by emitting a signal from the service layer that the app polls), or make the service layer handler also call `try_merge_save_gui` — which requires adding the dock snapshot to `ServiceContext`.

---

**A-8: Logs, Config, and GitLog tabs are permanently stub panels**

- File: `crates/kiwi_gui/src/dock/panels/placeholder.rs`, lines 25–28
- `KiwiTab::Logs`, `KiwiTab::Config`, and `KiwiTab::GitLog` all show "Panel content arrives in a later milestone." These tabs appear in the View menu and can be opened by the user. The factory layout does not include them, but they are in `all_variants()` and appear as menu items with checkboxes.
- Suggested fix: Either implement these panels, or hide them from the View menu until implemented. Showing unchecked menu items for unimplemented features creates a confusing user experience.

---

### Code Quality

---

**Q-1: `detect_github_key_action` modifier filter passes R with any modifier through**

- File: `crates/kiwi_gui/src/dock/panels/github_input.rs`, lines 99–128
- Line 104: `if input.modifiers.any() && !input.key_pressed(Key::R) { return None; }`. This condition exempts the R key from the modifier guard — so Ctrl+R, Shift+R, and Alt+R all trigger `GitHubRefresh`. Only Cmd+Enter is explicitly handled before this. The likely intent is: "block modifier combos except the ones explicitly listed above", meaning Cmd+Enter and plain R. As written, any R combination passes through to the `Refresh` branch.
- Suggested fix: `if input.modifiers.any() { return None; }` (after the Cmd+Enter check). If Ctrl+R as a refresh shortcut is desired, add an explicit check for it before the guard.

---

**Q-2: Empty `if` body with explanatory comment in `update()`**

- File: `crates/kiwi_gui/src/app.rs`, lines 366–368
- `if self.runtime.poll_search_debounce() { /* comment */ }` — the body is intentionally empty because the dispatch happens inside `poll_search_debounce`. This is not idiomatic Rust and will confuse readers who expect an if-body to do something.
- Suggested fix: Replace with `let _ = self.runtime.poll_search_debounce();` with a comment, or refactor `poll_search_debounce` to not also dispatch internally (separate concerns).

---

**Q-3: `encode_egui_key(Key::F5)` is dead code — F5 is intercepted before PTY input**

- Files: `crates/kiwi_gui/src/dock/panels/pty_input.rs`, line 280; `crates/kiwi_gui/src/app.rs`, lines 206–219
- `handle_input_shortcuts` intercepts F5 globally (refreshes git/GitHub/search). `collect_pty_input` runs after this. So even when Terminal has keyboard focus, the F5 global interceptor fires first and the PTY never receives `encode_egui_key(Key::F5)` → `b"\x1b[15~"`.
- This is not a crash, but `\x1b[15~` is dead code in practice. Terminals that need F5 (e.g., `htop`) will not work.
- Suggested fix: Document the priority decision explicitly and consider whether F5 should be forwarded to the PTY when a PTY panel has explicit keyboard focus, or add a separate refresh shortcut that doesn't conflict with F5.

---

**Q-4: `github_common::github_status_label` has convoluted tab-matching logic**

- File: `crates/kiwi_gui/src/dock/panels/github_common.rs`, lines 48–55
- The pattern `KiwiTab::GitHubIssues | KiwiTab::Issues if state.github.issues_loading && matches!(tab, KiwiTab::GitHubIssues)` matches both tabs in the outer arm but then re-checks `matches!(tab, KiwiTab::GitHubIssues)` in the guard. The outer match is wider than the guard, making the intent hard to read.
- Suggested fix: Split into two separate match arms — one for `KiwiTab::GitHubIssues` and one for `KiwiTab::Issues` — each with their own condition.

---

**Q-5: `GitRefreshRequested` dispatch in `runtime.rs:build` ignores the quit return value**

- File: `crates/kiwi_gui/src/runtime.rs`, line 97
- `runtime.dispatch(AppEvent::GitRefreshRequested);` discards the `bool` (quit signal) return value. While it is unlikely that a git refresh triggers quit at startup, the pattern is inconsistent with call sites elsewhere that use `let _ =` to explicitly acknowledge the discard.
- Suggested fix: `let _ = runtime.dispatch(AppEvent::GitRefreshRequested);`

---

**Q-6: `on_tab_button` in `viewer.rs` ignores the quit signal from dispatch**

- File: `crates/kiwi_gui/src/dock/viewer.rs`, line 29
- `let _ = (self.ctx.dispatch)(command);` — the dispatch function signature returns `bool` (quit signal) but all tab-button clicks discard it. If a tab-switch command triggered a quit, it would be silently ignored.
- Suggested fix: Consider checking the return value and propagating it, or document why tab-switch commands cannot cause a quit.

---

**Q-7: `KiwiTab::closable` is a const fn that always returns `true`**

- File: `crates/kiwi_gui/src/dock/tab.rs`, lines 46–48
- Every tab is closable with no distinction. The test "every_tab_is_closable_in_v1" confirms this is intentional for v1, but the function adds complexity with no value in its current form. The comment "in v1" suggests this will change.
- Suggested fix: Leave as-is but document which tabs will become non-closable in a future version, or remove the `closable()` abstraction and pass `true` directly to egui_dock until differentiation is needed.

---

## Summary

Priority order for addressing issues:

**Fix immediately (data loss / silent breakage):**
1. C-1: `CopyToClipboard`, `PasteFromClipboard`, `PersistUserTheme` silently dropped in `services.rs`
2. C-2: `SideEffect::SaveWorkspace` in service layer misses dock layout persistence
3. C-3: `render_command_palette` directly mutates `AppState` during render
4. C-4: Search `TextEdit` binds directly to `AppState.search.query`
5. C-6: `MainTab::Branches` navigates to wrong tab (`GitHubIssues` instead of `GitLog`)
6. C-8: Silent `_ => {}` catch-alls in all sub-enum arms of `execute_gui_effect`

**Fix soon (bugs that manifest under realistic use):**
7. C-5: `close_window` inside `CentralPanel` closure — update continues after quit signal
8. C-7: Double workspace save on Ctrl+Q quit path
9. A-7: Asymmetric workspace persistence (30-sec timer saves dock; reducer-save does not)

**Fix for performance (noticeable at scale):**
10. P-1: Clone of entire `diff.lines` and `preview.lines` every frame
11. P-2: O(n²) gutter-width computation in `render_diff_content`
12. P-4: `apply_to_context` clones `Visuals` every frame
13. P-3: `Vec<char>` allocations per line in `truncate_line`/`slice_line`

**Fix for architecture (maintenance debt):**
14. A-1: `PasteFromClipboard` needs `egui::Context` access in service layer
15. A-2: `truncate_line` duplication between `preview.rs` and `git_diff.rs`
16. A-3: Dead `footer` field in `PtyScrollbackView`
17. A-4, A-5: Dead production code (`primary_tab_for_navigation`, `navigation_sync_commands`)
18. A-8: Stub tabs visible in View menu

**Fix for code quality (low priority):**
19. Q-1: R key with any modifier triggers GitHub refresh
20. Q-3: F5 PTY encoding dead code
21. Q-2, Q-4–Q-7: Minor clarity and consistency issues
