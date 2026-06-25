# Issue Resolution Log

Chronological record of reported problems, root causes, and fixes. Prefer GitHub issues for new work; add an entry here when a fix lands so future sessions have context.

Format for new entries:

```markdown
### YYYY-MM-DD — Short title (GitHub #NNN)

- **Symptom:** …
- **Cause:** …
- **Fix:** …
- **Files:** …
- **Verify:** …
```

---

## M2 — Agent and Shell PTY (2026-06)

### Fuzzy filter for command palette (GitHub #28, SPEC-013)

- **Symptom:** #27 shipped a minimal subsequence matcher with flat scoring; ranking was weak (e.g. id/title tie-break only) and there was no performance guard for the SPEC-013 `< 5ms for 100 commands` budget.
- **Cause:** Basic `fuzzy_score` used fixed bonuses without gap penalties or word-boundary weighting; needle whitespace handling and consecutive-match detection had edge-case bugs.
- **Fix:** Replaced palette fuzzy logic with a scored matcher (`FuzzyMatch`, gap penalties, word-boundary and consecutive bonuses, whitespace-insensitive queries). Added reusable `filter_ranked` helper and a perf regression test over 100 synthetic commands.
- **Files:** `crates/kiwi/src/commands/fuzzy.rs`, `crates/kiwi/src/commands/mod.rs`
- **Verify:** `git_ref_ranks_refresh_above_left_git`, `filter_updates_within_spec_budget_for_100_commands`, existing palette fuzzy tests; 184 tests pass.

### Command registry and palette UI (GitHub #27, SPEC-013)

- **Symptom:** Commands pane showed only a static `Ctrl+P` hint; no searchable command list, no palette execution path, agent restart deferred to #27.
- **Cause:** `CommandPaletteState` was a boolean stub; `Ctrl+P` only moved focus without opening a modal palette, registry, or fuzzy filter.
- **Fix:** Added `commands/` registry with static `CommandDef` list, subsequence fuzzy matcher, palette reducer commands (`PaletteOpen`, input/selection/execute), `ui/palette.rs` rendering, global `Ctrl+P` handling, mouse click-to-execute, and in-session command history. Wired agent restart, git refresh, quit, focus, tab navigation, and editor open into the registry.
- **Files:** `crates/kiwi/src/commands/`, `state/domains.rs`, `state/event.rs`, `state/reducer.rs`, `ui/palette.rs`, `ui/render.rs`, `app.rs`, `navigation/keys.rs`
- **Verify:** `fuzzy_find_git_ref_matches_refresh_command`, `palette_close_restores_previous_focus`, `palette_match_at_maps_rows_below_prompt`; 180 tests pass.

### Agent pane repeating prompts (scrollback fidelity)

- **Symptom:** Agent tab stacked duplicate prompts and status lines, as if output were being debounced or appended multiple times.
- **Cause:** `ScrollbackBuffer` naively split PTY bytes on `\n` and committed every segment permanently. Interactive agent TUIs redraw with `\r`, clear-screen (`\x1b[2J`), and cursor-up (`\x1b[1A`); each redraw became a new scrollback line instead of overwriting the screen.
- **Fix:** Rewrote scrollback as a minimal cursor-based PTY screen (history + active screen grid). Handles `\r`, `\n`, `\t`, `\b`, CSI clear/position/cursor movement, alternate screen (`?1049`), and SGR passthrough. Agent and shell share the same buffer and `render_scrollback_pane` path — no separate agent display pipeline.
- **Files:** `crates/kiwi/src/shell/scrollback.rs`, `crates/kiwi/src/ui/scrollback.rs`, `crates/kiwi/src/ui/agent.rs`
- **Verify:** `clear_screen_drops_duplicate_prompts`, `cursor_up_allows_redrawing_previous_line`, `carriage_return_overwrites_current_line`, `split_escape_sequence_across_reads_is_reassembled`; manual agent tab shows one updating prompt.

### PTY pane colors overridden by Kiwi theme

- **Symptom:** Agent and shell text used Kiwi chrome theme foreground/background instead of the child process ANSI colors.
- **Cause:** Scrollback rendering stripped SGR codes and applied `ThemePalette` styles to PTY lines; full-frame background fill also set theme foreground on every cell.
- **Fix:** Added `ansi.rs` with `pty_base_style()` (`Color::Reset` fg/bg), `ansi_line()` SGR parser (16-color + 256-color fg), and `strip_ansi()` for heuristics only. PTY rows render with terminal-standard colors; Kiwi theme applies to borders, tabs, and status bar only.
- **Files:** `crates/kiwi/src/ansi.rs`, `crates/kiwi/src/ui/scrollback.rs`, `crates/kiwi/src/ui/render.rs`, `crates/kiwi/src/shell/scrollback.rs`
- **Verify:** `ansi_line_preserves_green_text`, `viewport_lines_preserves_ansi_color_codes`, `pty_base_style_resets_terminal_colors`; manual check that tool output colors match a normal terminal.

### Agent pane garbled / odd characters

- **Symptom:** Agent output showed mojibake (`â`, `¢`, etc.) and stray text like `?25h` mixed with real content.
- **Cause:** (1) After the screen-model rewrite, printable bytes were decoded as `byte as char`, breaking multi-byte UTF-8 (arrows, box-drawing, emoji). (2) CSI private-mode sequences (`\x1b[?25h`, `\x1b[?2004h`, etc.) failed to parse because the CSI lexer only accepted digits; failed escapes leaked as visible characters.
- **Fix:** Buffer PTY text in `text_pending` and decode valid UTF-8 before writing; use U+FFFD for invalid sequences. Replaced CSI parser with standard parameter/intermediate byte handling (`0x30–0x3F`, `0x20–0x2F`); private-mode `h`/`l` sequences are consumed and ignored. Short non-CSI escapes (`\x1b(B`, etc.) are consumed without printing.
- **Files:** `crates/kiwi/src/shell/scrollback.rs`
- **Verify:** `utf8_multibyte_characters_decode_correctly`, `utf8_split_across_reads_is_reassembled`, `private_mode_sequences_are_not_printed`; 168 tests pass.

### Agent restart command (GitHub #26, SPEC-010)

- **Symptom:** Crashed or exited agent could not be recovered without quitting Kiwi.
- **Cause:** No `AgentRestart` command, exit polling, or restart UX.
- **Fix:** `AppCommand::AgentRestart` + `SideEffect::RestartAgent`; poll child exit in main loop; footer with exit code and `Ctrl+Shift+R` hint; keyboard shortcut on Agent tab (palette wiring deferred to #27).
- **Files:** `app.rs`, `state/event.rs`, `state/reducer.rs`, `state/domains.rs`, `agent/session.rs`, `ui/agent.rs`, `ui/scrollback.rs`
- **Verify:** `agent_restart_emits_side_effect_on_agent_tab`, `agent_restart_shortcut_dispatches_on_agent_tab`; 168 tests pass.

### Agent status heuristics for status bar (GitHub #25, SPEC-010 / SPEC-019)

- **Symptom:** Status bar only showed generic "Agent Running" / "Agent Idle" regardless of agent output.
- **Cause:** No `AgentStatus` field or output parsing; status bar keyed off `running` only.
- **Fix:** Added `agent/status.rs` with keyword heuristics over recent scrollback; `AgentState.status` updated on output and exit; status bar uses semantic theme roles (`agent_thinking`, etc.).
- **Files:** `crates/kiwi/src/agent/status.rs`, `state/domains.rs`, `state/reducer.rs`, `ui/status_bar.rs`, `shell/scrollback.rs`
- **Verify:** `agent_output_updates_status_from_heuristics` and status bar unit tests; `cargo test` (155 tests).

### Agent I/O and viewport render (GitHub #24, SPEC-010)

- **Symptom:** Agent tab showed placeholder text; no agent output in the main pane.
- **Cause:** Lazy spawn (#23) created the PTY but no background reader or scrollback state existed for the agent.
- **Fix:** Added `AgentOutputReader`, `AppEvent::AgentOutput`, agent scrollback fields, and `ui/agent.rs` rendering in `main_content` when `MainTab::Agent`.
- **Files:** `crates/kiwi/src/agent/io.rs`, `state/domains.rs`, `state/reducer.rs`, `ui/agent.rs`, `ui/scrollback.rs`, `app.rs`
- **Verify:** Agent tab shows live output; `cargo test` reducer/render tests for agent scrollback.

### Agent keyboard input (SPEC-010 req. 3 — same branch as #24)

- **Symptom:** Agent output visible but typing had no effect on the Agent tab.
- **Cause:** Only shell focus forwarded keys; Main + Agent tab had no `AgentWrite` path.
- **Fix:** `handle_agent_key` when Main focus + Agent tab + agent running; `AppCommand::AgentWrite` / `AgentScroll` / `WriteAgent` side effect.
- **Files:** `app.rs`, `state/event.rs`, `state/reducer.rs`, `agent/session.rs`
- **Verify:** `agent_focus_forwards_keys_instead_of_quitting` test; manual type in agent pane.

### Quit hang / frozen TUI frame (follow-up to #21, PR #95)

- **Symptom:** `Ctrl+C` or quit left the alternate-screen frame on screen; host terminal unusable.
- **Cause:** SIGINT could bypass the app event loop; PTY reader join blocked shutdown; child reap delayed restore.
- **Fix:** `signal-hook` handlers; restore terminal before killing PTY children; abandon reader threads without joining; non-blocking child shutdown; stdout flush on restore.
- **Files:** `shutdown.rs`, `app.rs`, `terminal.rs`, `shell/io.rs`, `agent/io.rs`, `shell/session.rs`, `agent/session.rs`
- **Verify:** `cargo run` then `q`, `Ctrl+C`, and `Ctrl+Q`; terminal echo restored.

### Tab key not cycling focus

- **Symptom:** `Tab` did nothing while on the Agent tab with a running agent.
- **Cause:** All keys including `Tab` were forwarded to the agent PTY when agent input was active.
- **Fix:** Handle `Tab` / `Shift+Tab` before PTY handlers to always dispatch `NextFocus` / `PreviousFocus`.
- **Files:** `app.rs`, `docs/design/keyboard-shortcuts.md`
- **Verify:** `tab_cycles_focus_when_agent_input_is_active` test.

### Tab click left focus stuck / could not quit cleanly

- **Symptom:** After clicking a main tab, keystrokes still went to shell; `q` did not quit.
- **Cause:** Tab selection did not move focus; shell focus kept capturing input.
- **Fix:** Mouse tab clicks dispatch both `Select*Tab` and `SetFocus` (Main or Left); added `Ctrl+Q` global quit.
- **Files:** `ui/mouse.rs`, `app.rs`
- **Verify:** `mouse_click_on_main_tab_returns_focus_to_main`, `ctrl_q_quits_from_shell_focus` tests.

### Shell pane blank at startup

- **Symptom:** Shell looked empty though bash was running.
- **Cause:** Bash prompt has no trailing `\n`; scrollback only rendered completed lines, so the prompt stayed in the internal partial buffer.
- **Fix:** `viewport_lines(..., include_pending: true)` when following tail; bash spawned with `-i`; `TERM` set for PTY children.
- **Files:** `shell/scrollback.rs`, `ui/scrollback.rs`, `shell/session.rs`
- **Verify:** `viewport_lines_includes_pending_prompt_without_newline`, `draw_frame_renders_shell_prompt_without_trailing_newline` tests.

### Shell output bleeding into Commands pane

- **Symptom:** Shell text appeared in the left **Commands** palette area.
- **Cause:** Ratatui drew long/unprocessed PTY lines past the shell inner rect; `\t` and `\r` in PTY output widened or confused row layout.
- **Fix:** Normalize lines (`\r`, `\t`, ANSI); render one clipped row per terminal row with `Clear` per row; clear pane before border draw.
- **Files:** `shell/scrollback.rs`, `ui/scrollback.rs`
- **Verify:** `draw_frame_keeps_shell_output_inside_shell_pane` test.

---

## Reporting New Issues

1. File a [GitHub issue](https://github.com/pacificnm/kiwi/issues) for backlog tracking when appropriate.
2. Add **Active** entry to [KNOWN_ISSUES.md](../../KNOWN_ISSUES.md) if the bug remains on `main`.
3. When fixed, move summary here and remove or update the active entry.
