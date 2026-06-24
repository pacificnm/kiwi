# Release Plan

Versioning, release cadence, and criteria for Kiwi releases.

## Versioning

Semantic versioning from first public release:

| Version | Meaning |
|---------|---------|
| `0.1.0` | MVP (Milestones 1–5 complete) |
| `0.2.0` | M6 workspace features |
| `0.3.0` | M7 advanced (plugins, multi-agent) |
| `1.0.0` | Stable API, plugin ABI guarantee, documented upgrade path |

Pre-`0.1.0`: internal `0.0.x` tags per milestone completion.

## Release Milestones

### v0.0.1 — M1 Foundation (internal)

- Layout renders
- Config and themes work
- Navigation and quit

### v0.0.2 — M2 Terminal Services (internal)

- Shell + agent PTY
- Command palette

### v0.0.3 — M3 File Management (internal)

- File tree, preview, editor, search

### v0.0.4 — M4 Git (internal)

- Git status, watcher, diff

### v0.1.0 — MVP Public Release

**Criteria:**

- All MVP success criteria from plan.md verified
- README with install, config, dependencies (`gh`, `rg` optional)
- `config.example.toml` shipped
- Known limitations documented
- Linux primary; macOS best-effort tested

**Artifacts:**

- GitHub Release with prebuilt binaries (linux x86_64, aarch64; macOS universal optional)
- Cargo publish optional for `0.1.0`

### v0.2.0 — Workspace Release

- Persistence
- Theme packs documentation
- Performance fixes from MVP feedback

### v0.3.0 — Extensibility Release

- Plugin framework Phase 1
- Multi-agent beta
- Symbol search beta

### v1.0.0 — Stable Release

- Plugin API stable for one major cycle
- Security audit on plugin loading
- Comprehensive user documentation site (optional)

## Dependencies for Release

| Tool | MVP Required | Notes |
|------|--------------|-------|
| Rust toolchain | Yes | edition 2021 |
| Git | Yes | for repo operations |
| `gh` | For GitHub features | Graceful degrade |
| `rg` | For content search | Fallback message |

## Release Checklist

- [ ] All milestone acceptance criteria pass
- [ ] `cargo test` green
- [ ] `cargo clippy -- -D warnings` green
- [ ] Manual workflow test: issue-driven path
- [ ] CHANGELOG.md updated
- [ ] Version bumped in `Cargo.toml`
- [ ] Tag and GitHub Release
- [ ] Binary artifacts attached

## Future Enhancements (Post-1.0)

Grouped by theme:

### Integration

- Native GitHub API (`octocrab`)
- GitLab / Forgejo via plugins
- Jira / Linear plugins

### Editor Experience

- Syntax highlighting in preview (tree-sitter)
- Line/column editor args per editor preset
- Open at symbol (when symbol search lands)

### Performance

- Incremental diff for large files
- Persistent search index
- Parallel directory reads

### UX

- Config hot reload
- Custom keymaps
- Pane drag resize
- Onboarding tutorial overlay

### Platform

- Windows native build
- Nix flake distribution
- Homebrew formula

## Communication

- GitHub Releases for changelog
- Breaking changes called out in CHANGELOG and migration notes
- Plugin authors notified on `kiwi_plugin_api` semver bumps

## Related

- [milestones.md](./milestones.md)
- [backlog.md](./backlog.md)
