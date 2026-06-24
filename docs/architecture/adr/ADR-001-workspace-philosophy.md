# ADR-001: Workspace Philosophy

## Status

Accepted

## Context

Modern AI-assisted development often assumes a graphical IDE (Cursor, VS Code). Many developers prefer terminal-centric workflows with editors like Neovim or Helix. Kiwi must serve both audiences by providing IDE-like orchestration—agents, issues, PRs, diffs—without becoming another text editor.

Competing approaches include:

- **Full TUI editors** (e.g., built-in editing) — high complexity, duplicates existing tools
- **Thin terminal wrappers** — insufficient integration for issue-driven workflows
- **Graphical-only IDEs** — exclude terminal-native users

## Decision

Kiwi is an **orchestrator first, editor second**.

Kiwi coordinates:

- AI agent sessions
- Embedded shell
- Git and diff review
- GitHub Issues and Pull Requests
- File navigation, search, and read-only preview
- Launching external editors for actual file modification

Kiwi explicitly does **not** implement a full text editor. File editing is delegated to user-configured external editors (Vim, Neovim, Helix, Nano, Micro, VS Code, Cursor, Zed).

## Consequences

### Positive

- Reduced scope; faster path to MVP
- Respects user editor preferences and muscle memory
- Clear boundary: Kiwi = workspace, editor = editing
- Agent and Git workflows remain centralized

### Negative

- Context switching when editor opens in another terminal pane or GUI window
- No inline edit-in-place for quick fixes without editor launch
- Preview pane is read-only; users must understand the orchestrator model

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Built-in modal editor (vi-like) | Massive scope; poor parity with real editors |
| Embed LSP + code editor in TUI | Complexity rivals full IDE; conflicts with orchestrator role |
| GUI wrapper around web IDE | Violates terminal-native goal |

## Follow-up Work

- Document editor launch UX in [design/workflows.md](../../design/workflows.md)
- Implement SPEC-015 Editor Launcher
- Status bar and UI copy should reinforce "open in editor" rather than "edit here"
- Future: optional split-pane editor integration via plugin (ADR-018), not core
