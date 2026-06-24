# Known Issues

Tracked problems, gaps, and workarounds for the Kiwi project.

## Active

_No confirmed defects documented yet._ Early implementation is in progress on
Milestone 1 (Foundation). See [docs/roadmap/backlog.md](docs/roadmap/backlog.md)
for planned work and [docs/roadmap/milestones.md](docs/roadmap/milestones.md)
for milestone scope.

## Development Environment

- **Project memory database** — MCP semantic search requires a local PostgreSQL
  database with `pgvector` and a successful run of `./scripts/index-memory.sh`.
  Without it, `search_project_memory` returns empty results.
- **MCP paths** — `.cursor/mcp.json` may need machine-specific paths before
  Cursor can launch the memory servers.

## Reporting

When you find a reproducible bug, add an entry here with:

1. Symptom
2. Steps to reproduce
3. Expected vs actual behavior
4. Workaround, if any

Prefer filing a GitHub issue for items that belong on the implementation backlog.
