# Development Notes

Operational documentation for implementers: resolved defects, work-in-progress notes, and links to design/spec contracts.

## Documents

| Document | Purpose |
|----------|---------|
| [issue-resolution-log.md](./issue-resolution-log.md) | Symptom → cause → fix history for shipped and in-flight work |
| [pty-panes.md](./pty-panes.md) | How agent and shell PTY panes behave (focus, input, scrollback) |

## When to Update

Add an entry to **issue-resolution-log.md** when:

1. A GitHub issue or user report is fixed on a branch or merged to `main`
2. Behavior intentionally diverged from an older doc (cross-link the design doc update)
3. A workaround exists for an unfixed edge case (also add to `KNOWN_ISSUES.md` **Active**)

After substantive doc edits, re-index project memory:

```bash
./scripts/index-memory.sh
```

## Related

- [KNOWN_ISSUES.md](../../KNOWN_ISSUES.md) — active defects and environment gaps
- [backlog.md](../roadmap/backlog.md) — planned GitHub issues
- [keyboard-shortcuts.md](../design/keyboard-shortcuts.md) — default bindings
- [mouse-interaction.md](../design/mouse-interaction.md) — clicks, double-click preview, selection
- ADR-019 — system clipboard
- [navigation.md](../design/navigation.md) — focus and tab model
