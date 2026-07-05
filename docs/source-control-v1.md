# Kiwi Source Control v1 Implementation Plan

## Status: Phase 1 in progress

| Phase | Scope | Status |
| --- | --- | --- |
| 1 | Git status sidebar (read + stage + commit) | **In progress** |
| 2 | Diff view, discard, open changed file | Planned |
| 3 | Bottom Git output panel + `nest-git` crate | Planned |

## Goal

Replace the Source Control activity sidebar placeholder with a working Git panel
rooted at `[project].root`, similar to VS Code / Cursor.

## Paths

| Item | Path |
| --- | --- |
| Sidebar UI | `crates/kiwi/src/workbench/sidebar/source_control.rs` |
| Git model + CLI | `crates/kiwi/src/workbench/source_control/` |
| Workbench state | `crates/kiwi/src/workbench/state.rs` |
| Bottom Git tab (placeholder) | `crates/kiwi/src/workbench/bottom_panel/git.rs` |

## Phase 1 (current)

- Background `git status --porcelain=1` + `git branch --show-current`
- Branch header + refresh
- Staged / unstaged / untracked groupings
- Stage (+), unstage (−), stage all, commit with message
- Auto-refresh on project load, activity select, file watcher (when SC panel open)
- Non-repo projects show a friendly message

## Workspace (File menu)

- **File → Open Folder…** — native folder picker (`rfd`), switches workspace root
- **File → Open Recent** — persisted in `recent-projects.toml` next to `config.toml`
- Switching folders reloads explorer, terminal cwd, source control, and clears editor tabs

## Phase 2

- Click changed file → open in editor
- Inline diff preview
- Discard changes (`git restore`)

## Phase 3

- Extract `nest-git` module crate (shared with Finch / other Nest apps)
- Wire bottom **Git** tab to command output log
- Ahead/behind remote, push/pull

## Testing

```bash
cd apps/kiwi/desktop
cargo test -p kiwi source_control
RUSTFLAGS="-D warnings" cargo build -p kiwi
```

Manual: open Source Control activity on the Nest repo, verify branch + changed files,
stage a file, commit.

## Related

- [explorer-v1.md](./explorer-v1.md) — project root resolution
- Planned `nest-git` in [architecture.md](../../../docs/architecture.md)
