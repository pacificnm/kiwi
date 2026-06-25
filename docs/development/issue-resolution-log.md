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
