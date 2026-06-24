# ADR-016: Workspace Persistence

## Status

Accepted

## Context

Developers expect Kiwi to remember open tabs, file tree expansion, scroll positions, and panel sizes across sessions. Persistence must be local, privacy-respecting, and per-repository.

## Decision

Persist workspace state to **XDG state directory** keyed by repository identity.

### Storage location

```text
~/.local/state/kiwi/workspaces/<repo-hash>.json
```

`repo-hash` = stable hash of canonical repository root path (SHA256 truncated).

### Persisted fields

| Field | Description |
|-------|-------------|
| `left_nav_tab` | Active left navigation tab |
| `main_tab` | Active main workspace tab |
| `left_width` | Panel width percent or cols |
| `expanded_paths` | Set of expanded directory paths |
| `selected_path` | Last selected file path |
| `open_main_tabs` | Ordered list of open main tabs (future: closable tabs) |
| `scroll_positions` | Map of view ID → offset |
| `command_palette_history` | Recent commands (max 50) |

### Serialization

JSON via `serde_json` for human debuggability. Write atomically (temp file + rename).

### Lifecycle

- Load after config resolve, before first render
- Save on graceful quit (`q` quit command) and debounced every 30s during session
- Corrupt file → log warning, start fresh

### Not persisted

- PTY scrollback and session history
- GitHub auth tokens (handled by `gh`)
- Search query in progress

## Consequences

### Positive

- Resume work quickly across sessions
- Per-repo isolation
- No secrets in state files

### Negative

- Stale paths if files deleted externally
- Disk writes every 30s (negligible)
- JSON schema evolution needs versioning field

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| SQLite per repo | Overkill for key-value workspace state |
| Persist in `.kiwi.toml` | Pollutes git repo |
| No persistence | Poor daily-driver UX |

## Follow-up Work

- SPEC-017 Workspace Persistence
- Add `schema_version` field to state file
- Migration path for renamed fields
- Optional `workspace.persist = false` config
