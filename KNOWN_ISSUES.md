# Known Issues

Tracked problems, gaps, and workarounds for the Kiwi project.

## Active

_No confirmed defects on `main` documented here._ In-flight M2 PTY work may land on feature branches before merge.

See [docs/development/issue-resolution-log.md](docs/development/issue-resolution-log.md) for recently fixed issues on the agent/shell PTY branch (`#20`–`#24`).

## Resolved (recent)

| Issue | Summary | Log |
|-------|---------|-----|
| Quit / frozen frame | Terminal not restored on exit | [log](docs/development/issue-resolution-log.md) |
| Tab focus | Tab swallowed by agent PTY | [log](docs/development/issue-resolution-log.md) |
| Shell blank / overflow | Prompt and clip rendering | [log](docs/development/issue-resolution-log.md) |

Full symptom → cause → fix detail lives in the resolution log, not here.

## Development Environment

- **Project memory database** — MCP semantic search requires a local PostgreSQL
  database with `pgvector` and a successful run of `./scripts/index-memory.sh`.
  Without it, `search_project_memory` returns empty results.
- **MCP paths** — `.cursor/mcp.json` may need machine-specific paths before
  Cursor can launch the memory servers.

## Reporting

When you find a reproducible bug, add an entry under **Active** with:

1. Symptom
2. Steps to reproduce
3. Expected vs actual behavior
4. Workaround, if any

Prefer filing a GitHub issue for items that belong on the implementation backlog.
When fixed, add a dated entry to [docs/development/issue-resolution-log.md](docs/development/issue-resolution-log.md)
and remove or shorten the **Active** entry here.
