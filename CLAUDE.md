# Session Memory

Two MCP memory servers are always available. Use them — do not rely on conversation context surviving compaction.

## Always Do

- **MUST load context at session start.** Call `mcp__kiwi-context-memory__list_context_memory` (limit 5) to read recent session summaries before doing any work. If the user references a specific issue or feature, also call `mcp__kiwi-context-memory__search_context_memory` with the relevant keyword.
- **MUST save context after completing each task.** After every issue fix, feature, or significant decision, call `mcp__kiwi-context-memory__save_context_memory` with a `session_key` matching the branch/issue (e.g. `"225-fix"`), a descriptive `title`, relevant `tags`, and a `content` summary covering: what changed, which files, key decisions, and what's next.
- **Use `mcp__kiwi-memory__search_project_memory`** when you need architecture, spec, or design context (searches the `docs/` knowledge base).

## Never Do

- NEVER start coding without first checking session memory — prior context may already exist.
- NEVER end a task (commit + PR) without saving a memory entry summarising the work.

# GitNexus Index Hygiene

The `<!-- gitnexus:start/end -->` block below is auto-managed and will be rewritten by `npx gitnexus analyze`. Keep these rules here, above it.

## Always Do

- **MUST re-index after every commit.** After each `git commit`, run `npx gitnexus analyze` to keep the knowledge graph current. The PostToolUse hook will warn when the index is stale — act on it immediately.
- **MUST verify index freshness before starting any new issue.** Run `npx gitnexus status` or check `gitnexus://repo/kiwi/context`. If stale, re-index before doing any impact analysis.
- **USE gitnexus for exploration** — `gitnexus_query` instead of grep, `gitnexus_context` for full symbol context, `gitnexus_impact` before every edit.

## Never Do

- NEVER ignore a "index is stale" PostToolUse warning — re-index immediately.
- NEVER start work on a new issue without a fresh index.
- NEVER use find-and-replace to rename symbols — use `gitnexus_rename`.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **kiwi** (8706 symbols, 20885 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/kiwi/context` | Codebase overview, check index freshness |
| `gitnexus://repo/kiwi/clusters` | All functional areas |
| `gitnexus://repo/kiwi/processes` | All execution flows |
| `gitnexus://repo/kiwi/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
