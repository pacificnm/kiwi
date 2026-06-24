# Roadmap

Planning documents for Kiwi delivery: milestones, backlog, and release strategy.

## Documents

| Document | Description |
|----------|-------------|
| [milestones.md](./milestones.md) | Seven milestones with epics, stories, dependencies |
| [backlog.md](./backlog.md) | Implementation backlog and GitHub issue breakdown |
| [release-plan.md](./release-plan.md) | Versioning, MVP release, and future enhancements |

## MVP Scope

Milestones **1–5** deliver MVP per plan.md success criteria. Milestones 6–7 are post-MVP enhancements.

## Timeline Guidance

Estimates assume one experienced Rust developer full-time:

| Milestone | Duration (estimate) |
|-----------|-------------------|
| M1 Foundation | 2–3 weeks |
| M2 Terminal Services | 2 weeks |
| M3 File Management | 2–3 weeks |
| M4 Git Integration | 2 weeks |
| M5 GitHub Integration | 2–3 weeks |
| M6 Workspace Features | 1–2 weeks |
| M7 Advanced | 4+ weeks |

Total MVP: **~10–13 weeks**.

## Dependency Chain

```text
M1 → M2 → M3 → M4 → M5
              ↘ M6 (after M1, parallel after M3)
M7 after MVP stable
```

## Related

- [plan.md](../../plan.md)
- [specs/](../specs/README.md)
