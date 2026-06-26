# ADR-003: Layout Architecture

## Status

Accepted

## Context

Kiwi’s workspace has a fixed conceptual layout: left navigation, main workspace tabs, bottom command palette + shell, and a global status bar. The layout must resize gracefully, preserve focus across panes, and support configurable left panel width.

## Decision

Implement a **hierarchical layout engine** with named regions and percentage/fixed constraints:

```text
┌────────────────────────────┬─────────────────────────────────────────────┐
│ Left Nav Tabs (fixed h)  │ Main Tabs (fixed h)                         │
├────────────────────────────┼─────────────────────────────────────────────┤
│ Left Content (scroll)    │ Main Content (scroll / PTY)                 │
│                            │                                             │
├────────────────────────────┼─────────────────────────────────────────────┤
│ Command Palette (var h)  │ Shell PTY (min h)                           │
└────────────────────────────┴─────────────────────────────────────────────┘
Status Bar (1 row, full width)
```

### Region IDs

| Region | Default Size | Resizable |
|--------|--------------|-----------|
| `left_panel` | 30% width (`app.left_width`) | Horizontal only (future drag) |
| `main_panel` | Remainder | — |
| `bottom_panel` | 25% height (min 5 rows) | Vertical (future) |
| `status_bar` | 1 row | Fixed |

### Tab systems

- **Left nav tabs**: Files | Git | GH | Search — control left content only
- **Main tabs**: Agent | Issues | PRs | Diff | Preview | Logs — independent of left nav

Left nav selection and main tab selection are **orthogonal state**; e.g., Files + Agent, Git + Diff, or **GH + Issues** (issue list left, detail main).

### Focus model

One of: `left`, `main`, `command_palette`, `shell`. Tab within pane retains sub-focus (list index, scroll offset).

## Consequences

### Positive

- Predictable UX matching plan.md wireframe
- Clear widget ownership per region
- Independent nav vs workspace tabs enable powerful combinations

### Negative

- Two tab bars may confuse new users (mitigate with design docs and onboarding)
- PTY in bottom-right requires careful height allocation on small terminals

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Single tab bar for everything | Loses parallel context (file tree while viewing agent) |
| Floating/overlapping panes | Complex focus and mouse hit-testing |
| tmux-style tiling only | Poor fit for structured dev workspace |

## Follow-up Work

- SPEC-002 Layout Engine: constraints, min sizes, resize events
- SPEC-004 Navigation System: tab switching and focus rules
- Persist `left_width` and bottom height in workspace state (ADR-016)
- Design doc: [layout.md](../../design/layout.md)
