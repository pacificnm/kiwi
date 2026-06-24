# ADR-017: Multi-Agent Future Design

## Status

Accepted (design only — implementation Milestone 7)

## Context

MVP supports a single Cursor Agent PTY in the Agent tab. Users will want multiple concurrent agents, agent history, and orchestration (e.g., one agent on tests, one on implementation).

## Decision

Design for **multi-agent extensibility** now; implement **single agent** in MVP.

### Future architecture

```text
AgentManager
  ├── agents: HashMap<AgentId, AgentSession>
  ├── active_agent: AgentId
  └── each AgentSession:
        ├── pty: PtyState
        ├── label: String
        ├── linked_issue: Option<IssueNumber>
        └── status: Idle | Running | Error
```

### Main tab UX (future)

- Agent tab shows tab bar of agent sessions within main panel
- `Ctrl+Shift+N` spawn new agent
- `Ctrl+Tab` cycle agents
- Status bar shows count: `2 Agents (1 Running)`

### Orchestration (future)

- Queue prompts to idle agent
- Optional: pass context (issue body, selected files) as initial prompt template
- Agent history log persisted per repo (separate from PTY scrollback)

### MVP constraint

Single `AgentPty`; config `[agent] command` only. No `AgentManager` until M7.

## Consequences

### Positive

- Avoids rework of state model and tab system
- Clear user-facing roadmap
- Issue-linked agents align with issue-driven workflow

### Negative

- Some speculative design may change when implementing
- Resource usage with N PTYs needs limits (max 3 default)

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Multiple agents in MVP | Scope explosion |
| No multi-agent plan | Would require state refactor later |
| External orchestrator (docker) | Too heavy |

## Follow-up Work

- SPEC-010 Agent Service: single-agent API with `AgentId` placeholder type
- Document in roadmap M7
- Spike: memory/CPU with 3 concurrent `agent` processes
- Config future: `[[agents]]` table for named profiles
